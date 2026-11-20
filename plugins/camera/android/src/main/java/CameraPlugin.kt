// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.camera

import android.Manifest
import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.ContentValues
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import android.util.Base64
import androidx.activity.result.ActivityResult
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.FileProvider
import app.tauri.Logger
import app.tauri.PermissionState
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileNotFoundException
import java.io.FileOutputStream
import java.io.IOException
import java.util.UUID
import java.util.concurrent.Executors
import kotlin.math.max
import kotlin.math.min

// the arguments of getPhoto and pickImages. Everything is nullable so a missing or null value
// falls back to the default in CameraSettings
@InvokeArg
class ImageOptions {
  var quality: Int? = null
  var allowEditing: Boolean? = null
  var resultType: String? = null
  var saveToGallery: Boolean? = null
  var width: Int? = null
  var height: Int? = null
  var correctOrientation: Boolean? = null
  var source: String? = null
  var promptLabelHeader: String? = null
  var promptLabelPhoto: String? = null
  var promptLabelPicture: String? = null
  var limit: Int? = null
}

@InvokeArg
class RequestPermissionsArgs {
  var permissions: List<String>? = null
}

enum class CameraSource {
  PROMPT, CAMERA, PHOTOS
}

enum class CameraResultType {
  BASE64, URI, DATA_URL
}

class CameraSettings(options: ImageOptions) {
  val resultType: CameraResultType = when (options.resultType) {
    null, "base64" -> CameraResultType.BASE64
    "uri" -> CameraResultType.URI
    "dataUrl" -> CameraResultType.DATA_URL
    else -> throw IllegalArgumentException("Invalid resultType option")
  }
  var source: CameraSource = when (options.source) {
    null, "prompt" -> CameraSource.PROMPT
    "camera" -> CameraSource.CAMERA
    "photos" -> CameraSource.PHOTOS
    else -> throw IllegalArgumentException("Invalid source option")
  }
  val quality = (options.quality ?: 90).coerceIn(0, 100)
  val width = max(options.width ?: 0, 0)
  val height = max(options.height ?: 0, 0)
  val shouldResize = width > 0 || height > 0
  val shouldCorrectOrientation = options.correctOrientation ?: true
  val saveToGallery = options.saveToGallery ?: false
  val allowEditing = options.allowEditing ?: false
  val limit = max(options.limit ?: 0, 0)
  val promptLabelHeader = options.promptLabelHeader ?: "Photo"
  val promptLabelPhoto = options.promptLabelPhoto ?: "From Photos"
  val promptLabelPicture = options.promptLabelPicture ?: "Take Picture"
}

private const val CAMERA = "camera"
private const val PHOTOS = "photos"

private const val PERMISSION_DENIED_ERROR_CAMERA = "User denied access to camera"
private const val NO_CAMERA_ERROR = "Device doesn't have a camera available"
private const val NO_CAMERA_ACTIVITY_ERROR = "Unable to resolve camera activity"
private const val NO_PHOTO_ACTIVITY_ERROR = "Unable to resolve photo activity"
private const val IMAGE_FILE_SAVE_ERROR = "Unable to create photo on disk"
private const val IMAGE_PROCESS_NO_FILE_ERROR = "Unable to process image, file not found on disk"
private const val UNABLE_TO_PROCESS_IMAGE = "Unable to process image"
private const val IMAGE_EDIT_ERROR = "Unable to edit image"
private const val NO_EDIT_ACTIVITY_ERROR = "Unable to edit image, no app can edit images"
private const val IMAGE_GALLERY_SAVE_ERROR = "Unable to save the image in the gallery"

@TauriPlugin(
  permissions = [
    Permission(strings = [Manifest.permission.CAMERA], alias = CAMERA),
    // only needed to save pictures to the gallery on Android 9 and older
    Permission(strings = [Manifest.permission.WRITE_EXTERNAL_STORAGE], alias = PHOTOS)
  ]
)
class CameraPlugin(private val activity: Activity) : Plugin(activity) {
  // images are decoded and encoded here instead of on the main thread
  private val executor = Executors.newSingleThreadExecutor()

  private var settings = CameraSettings(ImageOptions())

  // the file the camera app writes the picture to
  private var imageFileSavePath: String? = null

  // the file the editor app writes the edited image to
  private var imageEditedFileSavePath: String? = null
  private var imagePickedContentUri: Uri? = null
  private var isEdited = false

  @Command
  fun getPhoto(invoke: Invoke) {
    if (!start(invoke)) {
      return
    }
    when (settings.source) {
      CameraSource.CAMERA -> openCamera(invoke)
      CameraSource.PHOTOS -> openPhotos(invoke, false)
      CameraSource.PROMPT -> showPrompt(invoke)
    }
  }

  @Command
  fun pickImages(invoke: Invoke) {
    if (!start(invoke)) {
      return
    }
    openPhotos(invoke, true)
  }

  @Command
  override fun requestPermissions(invoke: Invoke) {
    val requested =
      invoke.parseArgs(RequestPermissionsArgs::class.java).permissions ?: listOf(CAMERA, PHOTOS)
    // permissions that are reported as granted because they are not needed (see
    // getPermissionStates) must not be requested: the system would deny them right away
    val aliases = requested.filter {
      (it == CAMERA || it == PHOTOS) && getPermissionState(it) != PermissionState.GRANTED
    }
    if (aliases.isEmpty()) {
      checkPermissions(invoke)
    } else {
      requestPermissionForAliases(aliases.toTypedArray(), invoke, "permissionsCallback")
    }
  }

  @PermissionCallback
  private fun permissionsCallback(invoke: Invoke) {
    checkPermissions(invoke)
  }

  override fun getPermissionStates(): Map<String, PermissionState> {
    val states = super.getPermissionStates().toMutableMap()
    // The camera app takes the picture, so the app itself does not need the camera permission.
    // But if the app declares it in its manifest (e.g. for the barcode scanner plugin), the
    // system refuses to open the camera app until the permission is granted.
    if (!isPermissionDeclared(CAMERA)) {
      states[CAMERA] = PermissionState.GRANTED
    }
    // since Android 10 apps can add images to the gallery without a permission
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
      states[PHOTOS] = PermissionState.GRANTED
    }
    return states
  }

  /** Reads the options of a new request. Returns false if they are invalid, rejecting the invoke. */
  private fun start(invoke: Invoke): Boolean {
    try {
      settings = CameraSettings(invoke.parseArgs(ImageOptions::class.java))
    } catch (ex: Exception) {
      invoke.reject(ex.message ?: "Invalid options")
      return false
    }
    clearState()
    return true
  }

  private fun clearState() {
    imageFileSavePath = null
    imageEditedFileSavePath = null
    imagePickedContentUri = null
    isEdited = false
  }

  private fun showPrompt(invoke: Invoke) {
    activity.runOnUiThread {
      val fragment = CameraBottomSheetDialogFragment()
      fragment.setTitle(settings.promptLabelHeader)
      fragment.setOptions(
        listOf(settings.promptLabelPhoto, settings.promptLabelPicture),
        { index: Int ->
          if (index == 0) {
            settings.source = CameraSource.PHOTOS
            openPhotos(invoke, false)
          } else {
            settings.source = CameraSource.CAMERA
            openCamera(invoke)
          }
        },
        { invoke.resolve() })
      fragment.show((activity as AppCompatActivity).supportFragmentManager, "cameraPrompt")
    }
  }

  private fun openCamera(invoke: Invoke) {
    if (!activity.packageManager.hasSystemFeature(PackageManager.FEATURE_CAMERA_ANY)) {
      invoke.reject(NO_CAMERA_ERROR)
      return
    }
    val missing = ArrayList<String>()
    if (getPermissionState(CAMERA) != PermissionState.GRANTED) {
      missing.add(CAMERA)
    }
    if (settings.saveToGallery && getPermissionState(PHOTOS) != PermissionState.GRANTED) {
      missing.add(PHOTOS)
    }
    if (missing.isEmpty()) {
      launchCamera(invoke)
    } else {
      requestPermissionForAliases(missing.toTypedArray(), invoke, "cameraPermissionsCallback")
    }
  }

  @PermissionCallback
  private fun cameraPermissionsCallback(invoke: Invoke) {
    if (getPermissionState(CAMERA) != PermissionState.GRANTED) {
      invoke.reject(PERMISSION_DENIED_ERROR_CAMERA)
      return
    }
    // without the storage permission the picture is still returned, it just is not saved to the
    // gallery, which is reported in the `saved` field
    launchCamera(invoke)
  }

  private fun launchCamera(invoke: Invoke) {
    val takePictureIntent = Intent(MediaStore.ACTION_IMAGE_CAPTURE)
    if (takePictureIntent.resolveActivity(activity.packageManager) == null) {
      invoke.reject(NO_CAMERA_ACTIVITY_ERROR)
      return
    }
    try {
      // the camera app writes the picture to this file
      val photoFile = CameraUtils.createImageFile(activity)
      imageFileSavePath = photoFile.absolutePath
      takePictureIntent.putExtra(MediaStore.EXTRA_OUTPUT, fileProviderUri(photoFile))
    } catch (ex: Exception) {
      invoke.reject(IMAGE_FILE_SAVE_ERROR, ex)
      return
    }
    try {
      startActivityForResult(invoke, takePictureIntent, "processCameraImage")
    } catch (ex: ActivityNotFoundException) {
      invoke.reject(NO_CAMERA_ACTIVITY_ERROR)
    }
  }

  // the provider is declared by the manifest of Tauri apps
  private fun fileProviderUri(file: File): Uri {
    return FileProvider.getUriForFile(activity, activity.packageName + ".fileprovider", file)
  }

  private fun openPhotos(invoke: Invoke, multiple: Boolean) {
    // neither picker needs a permission: the app is only granted access to the selected images
    val intent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      // the system photo picker
      Intent(MediaStore.ACTION_PICK_IMAGES).apply {
        type = "image/*"
        if (multiple) {
          val maxLimit = MediaStore.getPickImagesMaxLimit()
          val limit = if (settings.limit == 0) maxLimit else min(settings.limit, maxLimit)
          // without the extra a single image is picked. It must be greater than 1
          if (limit > 1) {
            putExtra(MediaStore.EXTRA_PICK_IMAGES_MAX, limit)
          }
        }
      }
    } else {
      Intent(Intent.ACTION_GET_CONTENT).apply {
        type = "image/*"
        addCategory(Intent.CATEGORY_OPENABLE)
        putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple)
      }
    }
    try {
      startActivityForResult(
        invoke,
        intent,
        if (multiple) "processPickedImages" else "processPickedImage"
      )
    } catch (ex: ActivityNotFoundException) {
      invoke.reject(NO_PHOTO_ACTIVITY_ERROR)
    }
  }

  /** Runs [task] on the background thread, rejecting the invoke if it throws. */
  private fun runInBackground(invoke: Invoke, task: () -> Unit) {
    executor.execute {
      try {
        task()
      } catch (ex: Exception) {
        invoke.reject(ex.message ?: UNABLE_TO_PROCESS_IMAGE, ex)
      } catch (err: OutOfMemoryError) {
        invoke.reject("Out of memory")
      }
    }
  }

  @ActivityCallback
  private fun processCameraImage(invoke: Invoke, result: ActivityResult) {
    val path = imageFileSavePath
    if (path == null) {
      invoke.reject(IMAGE_PROCESS_NO_FILE_ERROR)
      return
    }
    if (result.resultCode != Activity.RESULT_OK) {
      // the user left the camera app without taking a picture
      File(path).delete()
      clearState()
      invoke.resolve()
      return
    }
    runInBackground(invoke) { processCameraFile(invoke, path) }
  }

  private fun processCameraFile(invoke: Invoke, path: String) {
    val bitmap = BitmapFactory.decodeFile(path) ?: throw IOException(UNABLE_TO_PROCESS_IMAGE)
    returnResult(invoke, bitmap, Uri.fromFile(File(path)))
  }

  @ActivityCallback
  private fun processPickedImage(invoke: Invoke, result: ActivityResult) {
    val uri = result.data?.data
    if (result.resultCode != Activity.RESULT_OK || uri == null) {
      invoke.resolve()
      return
    }
    imagePickedContentUri = uri
    runInBackground(invoke) { returnResult(invoke, decodeBitmap(uri), uri) }
  }

  @ActivityCallback
  private fun processPickedImages(invoke: Invoke, result: ActivityResult) {
    val uris = ArrayList<Uri>()
    val data = result.data
    if (result.resultCode == Activity.RESULT_OK && data != null) {
      val clipData = data.clipData
      if (clipData != null) {
        for (i in 0 until clipData.itemCount) {
          clipData.getItemAt(i).uri?.let { uris.add(it) }
        }
      } else {
        data.data?.let { uris.add(it) }
      }
    }
    runInBackground(invoke) {
      val images = JSArray()
      for (uri in uris) {
        images.put(processGalleryImage(uri))
      }
      val ret = JSObject()
      ret.put("images", images)
      invoke.resolve(ret)
    }
  }

  @ActivityCallback
  private fun processEditedImage(invoke: Invoke, result: ActivityResult) {
    isEdited = true
    runInBackground(invoke) {
      if (result.resultCode != Activity.RESULT_OK) {
        // the user cancelled the edit, return the image that was picked or taken
        val pickedUri = imagePickedContentUri
        val cameraPath = imageFileSavePath
        imageEditedFileSavePath = null
        if (pickedUri != null) {
          returnResult(invoke, decodeBitmap(pickedUri), pickedUri)
        } else if (cameraPath != null) {
          processCameraFile(invoke, cameraPath)
        } else {
          throw IOException(IMAGE_PROCESS_NO_FILE_ERROR)
        }
      } else {
        // editors either return the URI of the edited image or write it to the file we asked for
        val uri = result.data?.data
          ?: imageEditedFileSavePath?.let { Uri.fromFile(File(it)) }
          ?: throw IOException(IMAGE_PROCESS_NO_FILE_ERROR)
        returnResult(invoke, decodeBitmap(uri), uri)
      }
    }
  }

  @Throws(IOException::class)
  private fun decodeBitmap(uri: Uri): Bitmap {
    val bitmap = activity.contentResolver.openInputStream(uri)?.use {
      BitmapFactory.decodeStream(it)
    }
    return bitmap ?: throw IOException(UNABLE_TO_PROCESS_IMAGE)
  }

  /** Processes an image selected with pickImages and writes it to a file in the cache directory. */
  @Throws(IOException::class)
  private fun processGalleryImage(uri: Uri): JSObject {
    val exif = ImageUtils.getExifData(activity, uri)
    val bitmap = prepareBitmap(decodeBitmap(uri), uri, exif)
    val bitmapOutputStream = ByteArrayOutputStream()
    bitmap.compress(Bitmap.CompressFormat.JPEG, settings.quality, bitmapOutputStream)
    bitmap.recycle()
    val newUri = writeImage(uri, bitmapOutputStream)
    exif.copyExif(newUri.path)
    return fileResult(exif, newUri, false)
  }

  /** After the image is taken or picked, process it and return it to the webview. */
  @Throws(IOException::class)
  private fun returnResult(invoke: Invoke, bitmap: Bitmap, u: Uri) {
    val exif = ImageUtils.getExifData(activity, u)
    val preparedBitmap = prepareBitmap(bitmap, u, exif)
    // compress the final image and prepare for output to client
    val bitmapOutputStream = ByteArrayOutputStream()
    preparedBitmap.compress(Bitmap.CompressFormat.JPEG, settings.quality, bitmapOutputStream)

    if (settings.allowEditing && !isEdited) {
      preparedBitmap.recycle()
      // continues in processEditedImage
      editImage(invoke, u, bitmapOutputStream)
      return
    }

    // images picked from the gallery are already there, unless they were edited
    val saved = settings.saveToGallery
      && (imageEditedFileSavePath != null || imageFileSavePath != null)
      && saveToGallery(preparedBitmap)
    preparedBitmap.recycle()

    when (settings.resultType) {
      CameraResultType.URI -> {
        val newUri = writeImage(u, bitmapOutputStream)
        exif.copyExif(newUri.path)
        invoke.resolve(fileResult(exif, newUri, saved))
      }
      CameraResultType.BASE64, CameraResultType.DATA_URL -> {
        var data = Base64.encodeToString(bitmapOutputStream.toByteArray(), Base64.NO_WRAP)
        if (settings.resultType == CameraResultType.DATA_URL) {
          data = "data:image/jpeg;base64,$data"
        }
        val ret = JSObject()
        ret.put("format", "jpeg")
        ret.put("data", data)
        ret.put("exif", exif.toJson())
        ret.put("saved", saved)
        invoke.resolve(ret)
        // the picture file is not part of the result
        imageFileSavePath?.let { File(it).delete() }
      }
    }
    clearState()
  }

  private fun fileResult(exif: ExifWrapper, uri: Uri, saved: Boolean): JSObject {
    val ret = JSObject()
    ret.put("format", "jpeg")
    ret.put("data", uri.toString())
    ret.put("assetUrl", assetUrl(uri))
    ret.put("exif", exif.toJson())
    ret.put("saved", saved)
    return ret
  }

  /** Adds the image to the device's gallery. Returns whether that worked. */
  @Suppress("DEPRECATION")
  private fun saveToGallery(bitmap: Bitmap): Boolean {
    val name = File(imageEditedFileSavePath ?: imageFileSavePath ?: return false).name
    try {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        val resolver = activity.contentResolver
        val values = ContentValues()
        values.put(MediaStore.MediaColumns.DISPLAY_NAME, name)
        values.put(MediaStore.MediaColumns.MIME_TYPE, "image/jpeg")
        values.put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DCIM)
        val uri = resolver.insert(MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values)
          ?: throw IOException("Failed to create new MediaStore record.")
        val stream = resolver.openOutputStream(uri)
          ?: throw IOException("Failed to open output stream.")
        return stream.use { bitmap.compress(Bitmap.CompressFormat.JPEG, settings.quality, it) }
      } else {
        if (getPermissionState(PHOTOS) != PermissionState.GRANTED) {
          return false
        }
        return MediaStore.Images.Media.insertImage(
          activity.contentResolver,
          bitmap,
          name,
          ""
        ) != null
      }
    } catch (ex: Exception) {
      Logger.error(getLogTag(), IMAGE_GALLERY_SAVE_ERROR, ex)
      return false
    }
  }

  /**
   * Writes the processed image to a file and returns its URI.
   *
   * Pictures taken with the camera replace the file the camera app wrote. Other images are written
   * to a new file in the cache directory.
   */
  @Throws(IOException::class)
  private fun writeImage(uri: Uri, bitmapOutputStream: ByteArrayOutputStream): Uri {
    val bytes = bitmapOutputStream.toByteArray()
    var outFile = uri.path?.takeIf { uri.scheme == "file" }?.let { File(it) } ?: newCacheFile()
    try {
      FileOutputStream(outFile).use { it.write(bytes) }
    } catch (ex: FileNotFoundException) {
      // the file is read only
      outFile = newCacheFile()
      FileOutputStream(outFile).use { it.write(bytes) }
    }
    return Uri.fromFile(outFile)
  }

  private fun newCacheFile(): File {
    return File(activity.cacheDir, UUID.randomUUID().toString() + ".jpeg")
  }

  /**
   * Apply our standard processing of the bitmap, returning a new one and
   * recycling the old one in the process
   */
  @Throws(IOException::class)
  private fun prepareBitmap(bitmap: Bitmap, imageUri: Uri, exif: ExifWrapper): Bitmap {
    var preparedBitmap: Bitmap = bitmap
    if (settings.shouldCorrectOrientation) {
      val newBitmap = ImageUtils.correctOrientation(activity, preparedBitmap, imageUri, exif)
      preparedBitmap = replaceBitmap(preparedBitmap, newBitmap)
    }
    if (settings.shouldResize) {
      val newBitmap = ImageUtils.resize(preparedBitmap, settings.width, settings.height)
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

  /** Opens the image in an editor app. Continues in [processEditedImage]. */
  @Throws(IOException::class)
  private fun editImage(invoke: Invoke, uri: Uri, bitmapOutputStream: ByteArrayOutputStream) {
    val editFile = File(writeImage(uri, bitmapOutputStream).path!!)
    val editUri = fileProviderUri(editFile)
    val flags = Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
    val editIntent = Intent(Intent.ACTION_EDIT)
    editIntent.setDataAndType(editUri, "image/*")
    editIntent.addFlags(flags)
    editIntent.putExtra(MediaStore.EXTRA_OUTPUT, editUri)
    val editors = activity.packageManager
      .queryIntentActivities(editIntent, PackageManager.MATCH_DEFAULT_ONLY)
    if (editors.isEmpty()) {
      invoke.reject(NO_EDIT_ACTIVITY_ERROR)
      return
    }
    for (editor in editors) {
      activity.grantUriPermission(editor.activityInfo.packageName, editUri, flags)
    }
    imageEditedFileSavePath = editFile.absolutePath
    activity.runOnUiThread {
      try {
        startActivityForResult(invoke, editIntent, "processEditedImage")
      } catch (ex: Exception) {
        invoke.reject(IMAGE_EDIT_ERROR, ex)
      }
    }
  }
}
