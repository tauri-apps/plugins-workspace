package app.tauri.camera

import android.app.Activity
import android.net.Uri
import android.os.Environment
import androidx.core.content.FileProvider
import app.tauri.Logger
import java.io.File
import java.io.IOException
import java.text.SimpleDateFormat
import java.util.*

object CameraUtils {
  @Throws(IOException::class)
  fun createImageFileUri(activity: Activity, appId: String): Uri {
    val photoFile = createImageFile(activity)
    return FileProvider.getUriForFile(
      activity,
      "$appId.fileprovider", photoFile
    )
  }

  @Throws(IOException::class)
  fun createImageFile(activity: Activity): File {
    // Create an image file name
    val timeStamp: String = SimpleDateFormat("yyyyMMdd_HHmmss").format(Date())
    val imageFileName = "JPEG_" + timeStamp + "_"
    val storageDir =
      activity.getExternalFilesDir(Environment.DIRECTORY_PICTURES)
    return File.createTempFile(
      imageFileName,  /* prefix */
      ".jpg",  /* suffix */
      storageDir /* directory */
    )
  }

  internal val logTag: String
    internal get() = Logger.tags("CameraUtils")
}
