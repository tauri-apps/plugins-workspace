// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.camera

import android.app.Activity
import android.os.Environment
import java.io.File
import java.io.IOException
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

object CameraUtils {
  /** Creates an empty file in the app's pictures directory for the camera app to write to. */
  @Throws(IOException::class)
  fun createImageFile(activity: Activity): File {
    val timeStamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(Date())
    val storageDir = activity.getExternalFilesDir(Environment.DIRECTORY_PICTURES)
    return File.createTempFile("JPEG_" + timeStamp + "_", ".jpg", storageDir)
  }
}
