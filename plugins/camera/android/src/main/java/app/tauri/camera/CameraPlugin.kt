package app.tauri.camera

import android.Manifest
import android.app.Activity
import android.content.*
import android.content.pm.PackageManager
import android.content.pm.ResolveInfo
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.os.Parcelable
import android.provider.MediaStore
import android.util.Base64
import androidx.activity.result.ActivityResult
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.FileProvider
import androidx.exifinterface.media.ExifInterface.*
import app.tauri.*
import app.tauri.annotation.*
import app.tauri.plugin.*
import org.json.JSONException
import java.io.*
import java.util.*
import java.util.concurrent.Executor
import java.util.concurrent.Executors

enum class CameraSource(val source: String) {
  PROMPT("PROMPT"), CAMERA("CAMERA"), PHOTOS("PHOTOS");
}

enum class CameraResultType(val type: String) {
  BASE64("base64"), URI("uri"), DATAURL("dataUrl");
}

class CameraSettings {
  var resultType: CameraResultType = CameraResultType.BASE64
  var quality = DEFAULT_QUALITY
  var isShouldResize = false
  var isShouldCorrectOrientation = DEFAULT_CORRECT_ORIENTATION
  var isSaveToGallery = DEFAULT_SAVE_IMAGE_TO_GALLERY
  var isAllowEditing = false
  var width = 0
  var height = 0
  var source: CameraSource = CameraSource.PROMPT

  companion object {
    const val DEFAULT_QUALITY = 90
    const val DEFAULT_SAVE_IMAGE_TO_GALLERY = false
    const val DEFAULT_CORRECT_ORIENTATION = true
  }
}

@TauriPlugin(
  permissions = [
    Permission(strings = [Manifest.permission.CAMERA], alias = "camera"),
    Permission(
    strings = [Manifest.permission.READ_EXTERNAL_STORAGE, Manifest.permission.WRITE_EXTERNAL_STORAGE],
    alias = "photos"
  )]
)
class CameraPlugin(private val activity: Activity): Plugin(activity) {
  // Permission alias constants
  val CAMERA = "camera"
  val PHOTOS = "photos"

  // Message constants
  private val INVALID_RESULT_TYPE_ERROR = "Invalid resultType option"
  private val PERMISSION_DENIED_ERROR_CAMERA = "User denied access to camera"
  private val PERMISSION_DENIED_ERROR_PHOTOS = "User denied access to photos"
  private val NO_CAMERA_ERROR = "Device doesn't have a camera available"
  private val NO_CAMERA_ACTIVITY_ERROR = "Unable to resolve camera activity"
  private val NO_PHOTO_ACTIVITY_ERROR = "Unable to resolve photo activity"
  private val IMAGE_FILE_SAVE_ERROR = "Unable to create photo on disk"
  private val IMAGE_PROCESS_NO_FILE_ERROR = "Unable to process image, file not found on disk"
  private val UNABLE_TO_PROCESS_IMAGE = "Unable to process image"
  private val IMAGE_EDIT_ERROR = "Unable to edit image"
  private val IMAGE_GALLERY_SAVE_ERROR = "Unable to save the image in the gallery"

  private var imageFileSavePath: String? = null
  private var imageEditedFileSavePath: String? = null
  private var imageFileUri: Uri? = null
  private var imagePickedContentUri: Uri? = null
  private var isEdited = false
  private var isFirstRequest = true
  private var isSaved = false

  private var settings: CameraSettings = CameraSettings()

  @PluginMethod
  fun getPhoto(invoke: Invoke) {
    isEdited = false
    settings = getSettings(invoke)
    doShow(invoke)
  }

  @PluginMethod
  fun pickImages(invoke: Invoke) {
    settings = getSettings(invoke)
    openPhotos(invoke, multiple = true, skipPermission = false)
  }

  @PluginMethod
  fun pickLimitedLibraryPhotos(invoke: Invoke) {
    invoke.reject("not supported on android")
  }

  @PluginMethod
  fun getLimitedLibraryPhotos(invoke: Invoke) {
    invoke.reject("not supported on android")
  }

  private fun doShow(invoke: Invoke) {
    when (settings.source) {
      CameraSource.CAMERA -> showCamera(invoke)
      CameraSource.PHOTOS -> showPhotos(invoke)
      else -> showPrompt(invoke)
    }
  }

  private fun showPrompt(invoke: Invoke) {
    // We have all necessary permissions, open the camera
    val options: MutableList<String> = ArrayList()
    options.add(invoke.getString("promptLabelPhoto", "From Photos"))
    options.add(invoke.getString("promptLabelPicture", "Take Picture"))
    val fragment = CameraBottomSheetDialogFragment()
    fragment.setTitle(invoke.getString("promptLabelHeader", "Photo"))
    fragment.setOptions(
      options,
      { index: Int ->
        if (index == 0) {
          settings.source = CameraSource.PHOTOS
          openPhotos(invoke)
        } else if (index == 1) {
          settings.source = CameraSource.CAMERA
          openCamera(invoke)
        }
      },
      { invoke.reject("User cancelled photos app") })
    fragment.show((activity as AppCompatActivity).supportFragmentManager, "capacitorModalsActionSheet")
  }

  private fun showCamera(invoke: Invoke) {
    if (!activity.packageManager.hasSystemFeature(PackageManager.FEATURE_CAMERA_ANY)) {
      invoke.reject(NO_CAMERA_ERROR)
      return
    }
    openCamera(invoke)
  }

  private fun showPhotos(invoke: Invoke) {
    openPhotos(invoke)
  }

  private fun checkCameraPermissions(invoke: Invoke): Boolean {
    // if the manifest does not contain the camera permissions key, we don't need to ask the user
    val needCameraPerms = isPermissionDeclared(CAMERA)
    val hasCameraPerms = !needCameraPerms || getPermissionState(CAMERA) === PermissionState.GRANTED
    val hasPhotoPerms = getPermissionState(PHOTOS) === PermissionState.GRANTED

    // If we want to save to the gallery, we need two permissions
    if (settings.isSaveToGallery && !(hasCameraPerms && hasPhotoPerms) && isFirstRequest) {
      isFirstRequest = false
      val aliases = if (needCameraPerms) {
        arrayOf(CAMERA, PHOTOS)
      } else {
        arrayOf(PHOTOS)
      }
      requestPermissionForAliases(aliases, invoke, "cameraPermissionsCallback")
      return false
    } else if (!hasCameraPerms) {
      requestPermissionForAlias(CAMERA, invoke, "cameraPermissionsCallback")
      return false
    }
    return true
  }

  private fun checkPhotosPermissions(invoke: Invoke): Boolean {
    if (getPermissionState(PHOTOS) !== PermissionState.GRANTED) {
      requestPermissionForAlias(PHOTOS, invoke, "cameraPermissionsCallback")
      return false
    }
    return true
  }

  /**
   * Completes the plugin invoke after a camera permission request
   *
   * @see .getPhoto
   * @param invoke the plugin invoke
   */
  @PermissionCallback
  private fun cameraPermissionsCallback(invoke: Invoke) {
    if (invoke.command == "pickImages") {
      openPhotos(invoke, multiple = true, skipPermission = true)
    } else {
      if (settings.source === CameraSource.CAMERA && getPermissionState(CAMERA) !== PermissionState.GRANTED) {
        Logger.debug(
          getLogTag(),
          "User denied camera permission: " + getPermissionState(CAMERA).toString()
        )
        invoke.reject(PERMISSION_DENIED_ERROR_CAMERA)
        return
      } else if (settings.source === CameraSource.PHOTOS && getPermissionState(PHOTOS) !== PermissionState.GRANTED) {
        Logger.debug(
          getLogTag(),
          "User denied photos permission: " + getPermissionState(PHOTOS).toString()
        )
        invoke.reject(PERMISSION_DENIED_ERROR_PHOTOS)
        return
      }
      doShow(invoke)
    }
  }

  private fun getSettings(invoke: Invoke): CameraSettings {
    val settings = CameraSettings()
    val resultType = getResultType(invoke.getString("resultType"))
    if (resultType != null) {
      settings.resultType = resultType
    }
    settings.isSaveToGallery =
      invoke.getBoolean(
        "saveToGallery",
        CameraSettings.DEFAULT_SAVE_IMAGE_TO_GALLERY
      )
    settings.isAllowEditing = invoke.getBoolean("allowEditing", false)
    settings.quality = invoke.getInt("quality", CameraSettings.DEFAULT_QUALITY)
    settings.width = invoke.getInt("width", 0)
    settings.height = invoke.getInt("height", 0)
    settings.isShouldResize = settings.width > 0 || settings.height > 0
    settings.isShouldCorrectOrientation =
      invoke.getBoolean(
        "correctOrientation",
        CameraSettings.DEFAULT_CORRECT_ORIENTATION
      )

    try {
      settings.source =
        CameraSource.valueOf(
          invoke.getString(
            "source",
            CameraSource.PROMPT.source
          )
        )

    } catch (ex: IllegalArgumentException) {
      settings.source = CameraSource.PROMPT
    }
    return settings
  }

  private fun getResultType(resultType: String?): CameraResultType? {
    return if (resultType == null) {
      null
    } else try {
      CameraResultType.valueOf(resultType.uppercase(Locale.ROOT))
    } catch (ex: IllegalArgumentException) {
      Logger.debug(getLogTag(), "Invalid result type \"$resultType\", defaulting to base64")
      CameraResultType.BASE64
    }
  }

  private fun openCamera(invoke: Invoke) {
    if (checkCameraPermissions(invoke)) {
      val takePictureIntent = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
      if (takePictureIntent.resolveActivity(activity.packageManager) != null) {
        // If we will be saving the photo, send the target file along
        try {
          val appId: String = activity.packageName
          val photoFile: File = CameraUtils.createImageFile(activity)
          imageFileSavePath = photoFile.absolutePath
          // TODO: Verify provider config exists
          imageFileUri = FileProvider.getUriForFile(
            activity,
            "$appId.fileprovider", photoFile
          )
          takePictureIntent.putExtra(MediaStore.EXTRA_OUTPUT, imageFileUri)
        } catch (ex: Exception) {
          invoke.reject(IMAGE_FILE_SAVE_ERROR, ex)
          return
        }
        startActivityForResult(invoke, takePictureIntent, "processCameraImage")
      } else {
        invoke.reject(NO_CAMERA_ACTIVITY_ERROR)
      }
    }
  }

  private fun openPhotos(invoke: Invoke) {
    openPhotos(invoke, multiple = false, skipPermission = false)
  }

  private fun openPhotos(invoke: Invoke, multiple: Boolean, skipPermission: Boolean) {
    if (skipPermission || checkPhotosPermissions(invoke)) {
      val intent = Intent(Intent.ACTION_PICK)
      intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple)
      intent.setType("image/*")
      try {
        if (multiple) {
          intent.putExtra("multi-pick", multiple)
          intent.putExtra(Intent.EXTRA_MIME_TYPES, arrayOf("image/*"))
          startActivityForResult(invoke, intent, "processPickedImages")
        } else {
          startActivityForResult(invoke, intent, "processPickedImage")
        }
      } catch (ex: ActivityNotFoundException) {
        invoke.reject(NO_PHOTO_ACTIVITY_ERROR)
      }
    }
  }

  @ActivityCallback
  fun processCameraImage(invoke: Invoke, result: ActivityResult?) {
    settings = getSettings(invoke)
    if (imageFileSavePath == null) {
      invoke.reject(IMAGE_PROCESS_NO_FILE_ERROR)
      return
    }
    // Load the image as a Bitmap
    val f = File(imageFileSavePath!!)
    val bmOptions: BitmapFactory.Options = BitmapFactory.Options()
    val contentUri: Uri = Uri.fromFile(f)
    val bitmap: Bitmap = BitmapFactory.decodeFile(imageFileSavePath, bmOptions)
    returnResult(invoke, bitmap, contentUri)
  }

  @ActivityCallback
  fun processPickedImage(invoke: Invoke, result: ActivityResult) {
    settings = getSettings(invoke)
    val data: Intent? = result.data
    if (data == null) {
      invoke.reject("No image picked")
      return
    }
    val u: Uri = data.data!!
    imagePickedContentUri = u
    processPickedImage(u, invoke)
  }

  @ActivityCallback
  fun processPickedImages(invoke: Invoke, result: ActivityResult) {
    val data: Intent? = result.data
    if (data != null) {
      val executor: Executor = Executors.newSingleThreadExecutor()
      executor.execute {
        val ret = JSObject()
        val photos = JSArray()
        if (data.clipData != null) {
          val count: Int = data.clipData!!.itemCount
          for (i in 0 until count) {
            val imageUri: Uri = data.clipData!!.getItemAt(i).uri
            val processResult = processPickedImages(imageUri)
            if (processResult.getString("error").isNotEmpty()
            ) {
              invoke.reject(processResult.getString("error"))
              return@execute
            } else {
              photos.put(processResult)
            }
          }
        } else if (data.data != null) {
          val imageUri: Uri = data.data!!
          val processResult = processPickedImages(imageUri)
          if (processResult.getString("error").isNotEmpty()
          ) {
            invoke.reject(processResult.getString("error"))
            return@execute
          } else {
            photos.put(processResult)
          }
        } else if (data.extras != null) {
          val bundle: Bundle = data.extras!!
          if (bundle.keySet().contains("selectedItems")) {
            val fileUris: ArrayList<Parcelable>? = bundle.getParcelableArrayList("selectedItems")
            if (fileUris != null) {
              for (fileUri in fileUris) {
                if (fileUri is Uri) {
                  val imageUri: Uri = fileUri
                  try {
                    val processResult = processPickedImages(imageUri)
                    if (processResult.getString("error").isNotEmpty()
                    ) {
                      invoke.reject(processResult.getString("error"))
                      return@execute
                    } else {
                      photos.put(processResult)
                    }
                  } catch (ex: SecurityException) {
                    invoke.reject("SecurityException")
                  }
                }
              }
            }
          }
        }
        ret.put("photos", photos)
        invoke.resolve(ret)
      }
    } else {
      invoke.reject("No images picked")
    }
  }

  private fun processPickedImage(imageUri: Uri, invoke: Invoke) {
    var imageStream: InputStream? = null
    try {
      imageStream = activity.contentResolver.openInputStream(imageUri)
      val bitmap = BitmapFactory.decodeStream(imageStream)
      if (bitmap == null) {
        invoke.reject("Unable to process bitmap")
        return
      }
      returnResult(invoke, bitmap, imageUri)
    } catch (err: OutOfMemoryError) {
      invoke.reject("Out of memory")
    } catch (ex: FileNotFoundException) {
      invoke.reject("No such image found", ex)
    } finally {
      if (imageStream != null) {
        try {
          imageStream.close()
        } catch (e: IOException) {
          Logger.error(getLogTag(), UNABLE_TO_PROCESS_IMAGE, e)
        }
      }
    }
  }

  private fun processPickedImages(imageUri: Uri): JSObject {
    var imageStream: InputStream? = null
    val ret = JSObject()
    try {
      imageStream = activity.contentResolver.openInputStream(imageUri)
      var bitmap = BitmapFactory.decodeStream(imageStream)
      if (bitmap == null) {
        ret.put("error", "Unable to process bitmap")
        return ret
      }
      val exif: ExifWrapper = ImageUtils.getExifData(activity, bitmap, imageUri)
      bitmap = try {
        prepareBitmap(bitmap, imageUri, exif)
      } catch (e: IOException) {
        ret.put("error", UNABLE_TO_PROCESS_IMAGE)
        return ret
      }
      // Compress the final image and prepare for output to client
      val bitmapOutputStream = ByteArrayOutputStream()
      bitmap.compress(Bitmap.CompressFormat.JPEG, settings.quality, bitmapOutputStream)
      val newUri: Uri? = getTempImage(imageUri, bitmapOutputStream)
      exif.copyExif(newUri?.path)
      if (newUri != null) {
        ret.put("format", "jpeg")
        ret.put("exif", exif.toJson())
        ret.put("data", newUri.toString())
        ret.put("assetUrl", assetUrl(newUri))
      } else {
        ret.put("error", UNABLE_TO_PROCESS_IMAGE)
      }
      return ret
    } catch (err: OutOfMemoryError) {
      ret.put("error", "Out of memory")
    } catch (ex: FileNotFoundException) {
      ret.put("error", "No such image found")
      Logger.error(getLogTag(), "No such image found", ex)
    } finally {
      if (imageStream != null) {
        try {
          imageStream.close()
        } catch (e: IOException) {
          Logger.error(getLogTag(), UNABLE_TO_PROCESS_IMAGE, e)
        }
      }
    }
    return ret
  }

  @ActivityCallback
  private fun processEditedImage(invoke: Invoke, result: ActivityResult) {
    isEdited = true
    settings = getSettings(invoke)
    if (result.resultCode == Activity.RESULT_CANCELED) {
      // User cancelled the edit operation, if this file was picked from photos,
      // process the original picked image, otherwise process it as a camera photo
      if (imagePickedContentUri != null) {
        processPickedImage(imagePickedContentUri!!, invoke)
      } else {
        processCameraImage(invoke, result)
      }
    } else {
      processPickedImage(invoke, result)
    }
  }

  /**
   * Save the modified image on the same path,
   * or on a temporary location if it's a content url
   * @param uri
   * @param is
   * @return
   * @throws IOException
   */
  @Throws(IOException::class)
  private fun saveImage(uri: Uri, input: InputStream): Uri? {
    var outFile = if (uri.scheme.equals("content")) {
      getTempFile(uri)
    } else {
      uri.path?.let { File(it) }
    }
    try {
      writePhoto(outFile!!, input)
    } catch (ex: FileNotFoundException) {
      // Some gallery apps return read only file url, create a temporary file for modifications
      outFile = getTempFile(uri)
      writePhoto(outFile, input)
    }
    return Uri.fromFile(outFile)
  }

  @Throws(IOException::class)
  private fun writePhoto(outFile: File, input: InputStream) {
    val fos = FileOutputStream(outFile)
    val buffer = ByteArray(1024)
    var len: Int
    while (input.read(buffer).also { len = it } != -1) {
      fos.write(buffer, 0, len)
    }
    fos.close()
  }

  private fun getTempFile(uri: Uri): File {
    var filename: String = Uri.parse(Uri.decode(uri.toString())).lastPathSegment!!
    if (!filename.contains(".jpg") && !filename.contains(".jpeg")) {
      filename += "." + Date().time + ".jpeg"
    }
    val cacheDir: File = activity.getCacheDir()
    return File(cacheDir, filename)
  }

  /**
   * After processing the image, return the final result back to the invokeer.
   * @param invoke
   * @param bitmap
   * @param u
   */
  private fun returnResult(invoke: Invoke, bitmap: Bitmap, u: Uri) {
    val exif: ExifWrapper = ImageUtils.getExifData(activity, bitmap, u)
    val preparedBitmap = try {
      prepareBitmap(bitmap, u, exif)
    } catch (e: IOException) {
      invoke.reject(UNABLE_TO_PROCESS_IMAGE)
      return
    }
    // Compress the final image and prepare for output to client
    val bitmapOutputStream = ByteArrayOutputStream()
    preparedBitmap.compress(Bitmap.CompressFormat.JPEG, settings.quality, bitmapOutputStream)
    if (settings.isAllowEditing && !isEdited) {
      editImage(invoke, u, bitmapOutputStream)
      return
    }
    val saveToGallery: Boolean =
      invoke.getBoolean("saveToGallery", CameraSettings.DEFAULT_SAVE_IMAGE_TO_GALLERY)
    if (saveToGallery && (imageEditedFileSavePath != null || imageFileSavePath != null)) {
      isSaved = true
      try {
        val fileToSavePath =
          if (imageEditedFileSavePath != null) imageEditedFileSavePath!! else imageFileSavePath!!
        val fileToSave = File(fileToSavePath)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
          val resolver: ContentResolver = activity.contentResolver
          val values = ContentValues()
          values.put(MediaStore.MediaColumns.DISPLAY_NAME, fileToSave.name)
          values.put(MediaStore.MediaColumns.MIME_TYPE, "image/jpeg")
          values.put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DCIM)
          val contentUri: Uri = MediaStore.Images.Media.EXTERNAL_CONTENT_URI
          val uri: Uri = resolver.insert(contentUri, values)
            ?: throw IOException("Failed to create new MediaStore record.")
          val stream: OutputStream = resolver.openOutputStream(uri)
            ?: throw IOException("Failed to open output stream.")
          val inserted: Boolean =
            preparedBitmap.compress(Bitmap.CompressFormat.JPEG, settings.quality, stream)
          if (!inserted) {
            isSaved = false
          }
        } else {
          val inserted = MediaStore.Images.Media.insertImage(
            activity.contentResolver,
            fileToSavePath,
            fileToSave.name,
            ""
          )
          if (inserted == null) {
            isSaved = false
          }
        }
      } catch (e: FileNotFoundException) {
        isSaved = false
        Logger.error(getLogTag(), IMAGE_GALLERY_SAVE_ERROR, e)
      } catch (e: IOException) {
        isSaved = false
        Logger.error(getLogTag(), IMAGE_GALLERY_SAVE_ERROR, e)
      }
    }
    if (settings.resultType === CameraResultType.BASE64) {
      returnBase64(invoke, exif, bitmapOutputStream)
    } else if (settings.resultType === CameraResultType.URI) {
      returnFileURI(invoke, exif, bitmap, u, bitmapOutputStream)
    } else if (settings.resultType === CameraResultType.DATAURL) {
      returnDataUrl(invoke, exif, bitmapOutputStream)
    } else {
      invoke.reject(INVALID_RESULT_TYPE_ERROR)
    }
    // Result returned, clear stored paths and images
    if (settings.resultType !== CameraResultType.URI) {
      deleteImageFile()
    }
    imageFileSavePath = null
    imageFileUri = null
    imagePickedContentUri = null
    imageEditedFileSavePath = null
  }

  private fun deleteImageFile() {
    if (imageFileSavePath != null && !settings.isSaveToGallery) {
      val photoFile = File(imageFileSavePath!!)
      if (photoFile.exists()) {
        photoFile.delete()
      }
    }
  }

  private fun returnFileURI(
    invoke: Invoke,
    exif: ExifWrapper,
    bitmap: Bitmap,
    u: Uri,
    bitmapOutputStream: ByteArrayOutputStream
  ) {
    val newUri: Uri? = getTempImage(u, bitmapOutputStream)
    exif.copyExif(newUri?.path)
    if (newUri != null) {
      val ret = JSObject()
      ret.put("format", "jpeg")
      ret.put("exif", exif.toJson())
      ret.put("data", newUri.toString())
      ret.put("assetUrl", assetUrl(newUri))
      ret.put("saved", isSaved)
      invoke.resolve(ret)
    } else {
      invoke.reject(UNABLE_TO_PROCESS_IMAGE)
    }
  }

  private fun getTempImage(u: Uri, bitmapOutputStream: ByteArrayOutputStream): Uri? {
    var bis: ByteArrayInputStream? = null
    var newUri: Uri? = null
    try {
      bis = ByteArrayInputStream(bitmapOutputStream.toByteArray())
      newUri = saveImage(u, bis)
    } catch (_: IOException) {
    } finally {
      if (bis != null) {
        try {
          bis.close()
        } catch (e: IOException) {
          Logger.error(getLogTag(), UNABLE_TO_PROCESS_IMAGE, e)
        }
      }
    }
    return newUri
  }

  /**
   * Apply our standard processing of the bitmap, returning a new one and
   * recycling the old one in the process
   * @param bitmap
   * @param imageUri
   * @param exif
   * @return
   */
  @Throws(IOException::class)
  private fun prepareBitmap(bitmap: Bitmap, imageUri: Uri, exif: ExifWrapper): Bitmap {
    var preparedBitmap: Bitmap = bitmap
    if (settings.isShouldCorrectOrientation) {
      val newBitmap: Bitmap = ImageUtils.correctOrientation(activity, preparedBitmap, imageUri, exif)
      preparedBitmap = replaceBitmap(preparedBitmap, newBitmap)
    }
    if (settings.isShouldResize) {
      val newBitmap: Bitmap = ImageUtils.resize(preparedBitmap, settings.width, settings.height)
      preparedBitmap = replaceBitmap(preparedBitmap, newBitmap)
    }
    return preparedBitmap
  }

  private fun replaceBitmap(bitmap: Bitmap, newBitmap: Bitmap): Bitmap {
    if (bitmap !== newBitmap) {
      bitmap.recycle()
    }
    return newBitmap
  }

  private fun returnDataUrl(
    invoke: Invoke,
    exif: ExifWrapper,
    bitmapOutputStream: ByteArrayOutputStream
  ) {
    val byteArray: ByteArray = bitmapOutputStream.toByteArray()
    val encoded: String = Base64.encodeToString(byteArray, Base64.NO_WRAP)
    val data = JSObject()
    data.put("format", "jpeg")
    data.put("data", "data:image/jpeg;base64,$encoded")
    data.put("exif", exif.toJson())
    invoke.resolve(data)
  }

  private fun returnBase64(
    invoke: Invoke,
    exif: ExifWrapper,
    bitmapOutputStream: ByteArrayOutputStream
  ) {
    val byteArray: ByteArray = bitmapOutputStream.toByteArray()
    val encoded: String = Base64.encodeToString(byteArray, Base64.NO_WRAP)
    val data = JSObject()
    data.put("format", "jpeg")
    data.put("data", encoded)
    data.put("exif", exif.toJson())
    invoke.resolve(data)
  }

  @PluginMethod
  override fun requestPermissions(invoke: Invoke) {
    // If the camera permission is defined in the manifest, then we have to prompt the user
    // or else we will get a security exception when trying to present the camera. If, however,
    // it is not defined in the manifest then we don't need to prompt and it will just work.
    if (isPermissionDeclared(CAMERA)) {
      // just request normally
      super.requestPermissions(invoke)
    } else {
      // the manifest does not define camera permissions, so we need to decide what to do
      // first, extract the permissions being requested
      val providedPerms = invoke.getArray("permissions", JSArray())
      var permsList: List<String>? = null
      try {
        permsList = providedPerms.toList()
      } catch (_: JSONException) {
      }
      if (permsList != null && permsList.size == 1 && permsList.contains(CAMERA)) {
        // the only thing being asked for was the camera so we can just return the current state
        checkPermissions(invoke)
      } else {
        // we need to ask about photos so request storage permissions
        requestPermissionForAlias(PHOTOS, invoke, "checkPermissions")
      }
    }
  }

  override fun getPermissionStates(): Map<String, PermissionState> {
    val permissionStates = super.getPermissionStates() as MutableMap

    // If Camera is not in the manifest and therefore not required, say the permission is granted
    if (!isPermissionDeclared(CAMERA)) {
      permissionStates[CAMERA] = PermissionState.GRANTED
    }
    return permissionStates
  }

  private fun editImage(invoke: Invoke, uri: Uri, bitmapOutputStream: ByteArrayOutputStream) {
    try {
      val tempImage = getTempImage(uri, bitmapOutputStream)
      val editIntent = createEditIntent(tempImage)
      if (editIntent != null) {
        startActivityForResult(invoke, editIntent, "processEditedImage")
      } else {
        invoke.reject(IMAGE_EDIT_ERROR)
      }
    } catch (ex: Exception) {
      invoke.reject(IMAGE_EDIT_ERROR, ex)
    }
  }

  private fun createEditIntent(origPhotoUri: Uri?): Intent? {
    return try {
      val editFile = origPhotoUri?.path?.let { File(it) }
      val editUri: Uri = FileProvider.getUriForFile(
        activity,
        activity.packageName + ".fileprovider",
        editFile!!
      )
      val editIntent = Intent(Intent.ACTION_EDIT)
      editIntent.setDataAndType(editUri, "image/*")
      imageEditedFileSavePath = editFile.absolutePath
      val flags: Int =
        Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
      editIntent.addFlags(flags)
      editIntent.putExtra(MediaStore.EXTRA_OUTPUT, editUri)
      val resInfoList: List<ResolveInfo> = activity
        .packageManager
        .queryIntentActivities(editIntent, PackageManager.MATCH_DEFAULT_ONLY)
      for (resolveInfo in resInfoList) {
        val packageName: String = resolveInfo.activityInfo.packageName
        activity.grantUriPermission(packageName, editUri, flags)
      }
      editIntent
    } catch (ex: Exception) {
      null
    }
  }

  /*protected fun saveInstanceState(): Bundle? {
    val bundle: Bundle = super.saveInstanceState()
    if (bundle != null) {
      bundle.putString("cameraImageFileSavePath", imageFileSavePath)
    }
    return bundle
  }

  protected fun restoreState(state: Bundle) {
    val storedImageFileSavePath: String = state.getString("cameraImageFileSavePath")
    if (storedImageFileSavePath != null) {
      imageFileSavePath = storedImageFileSavePath
    }
  }*/
}
