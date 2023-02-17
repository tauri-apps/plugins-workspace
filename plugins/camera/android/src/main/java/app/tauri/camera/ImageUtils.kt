package app.tauri.camera

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Matrix
import android.net.Uri
import androidx.exifinterface.media.ExifInterface
import app.tauri.Logger
import java.io.IOException
import java.io.InputStream
import java.lang.Integer.min
import kotlin.math.roundToInt

object ImageUtils {
  /**
   * Resize an image to the given max width and max height. Constraint can be put
   * on one dimension, or both. Resize will always preserve aspect ratio.
   * @param bitmap
   * @param desiredMaxWidth
   * @param desiredMaxHeight
   * @return a new, scaled Bitmap
   */
  fun resize(bitmap: Bitmap, desiredMaxWidth: Int, desiredMaxHeight: Int): Bitmap {
    return resizePreservingAspectRatio(bitmap, desiredMaxWidth, desiredMaxHeight)
  }

  /**
   * Resize an image to the given max width and max height. Constraint can be put
   * on one dimension, or both. Resize will always preserve aspect ratio.
   * @param bitmap
   * @param desiredMaxWidth
   * @param desiredMaxHeight
   * @return a new, scaled Bitmap
   */
  private fun resizePreservingAspectRatio(
    bitmap: Bitmap,
    desiredMaxWidth: Int,
    desiredMaxHeight: Int
  ): Bitmap {
    val width = bitmap.width
    val height = bitmap.height

    // 0 is treated as 'no restriction'
    val maxHeight = if (desiredMaxHeight == 0) height else desiredMaxHeight
    val maxWidth = if (desiredMaxWidth == 0) width else desiredMaxWidth

    // resize with preserved aspect ratio
    var newWidth = min(width, maxWidth).toFloat()
    var newHeight = height * newWidth / width
    if (newHeight > maxHeight) {
      newWidth = (width * maxHeight / height).toFloat()
      newHeight = maxHeight.toFloat()
    }
    return Bitmap.createScaledBitmap(bitmap, newWidth.roundToInt(), newHeight.roundToInt(), false)
  }

  /**
   * Transform an image with the given matrix
   * @param bitmap
   * @param matrix
   * @return
   */
  private fun transform(bitmap: Bitmap, matrix: Matrix): Bitmap {
    return Bitmap.createBitmap(bitmap, 0, 0, bitmap.width, bitmap.height, matrix, true)
  }

  /**
   * Correct the orientation of an image by reading its exif information and rotating
   * the appropriate amount for portrait mode
   * @param bitmap
   * @param imageUri
   * @param exif
   * @return
   */
  @Throws(IOException::class)
  fun correctOrientation(c: Context, bitmap: Bitmap, imageUri: Uri, exif: ExifWrapper): Bitmap {
    val orientation = getOrientation(c, imageUri)
    return if (orientation != 0) {
      val matrix = Matrix()
      matrix.postRotate(orientation.toFloat())
      exif.resetOrientation()
      transform(bitmap, matrix)
    } else {
      bitmap
    }
  }

  @Throws(IOException::class)
  private fun getOrientation(c: Context, imageUri: Uri): Int {
    var result = 0
    c.getContentResolver().openInputStream(imageUri).use { iStream ->
      val exifInterface = ExifInterface(iStream!!)
      val orientation: Int = exifInterface.getAttributeInt(
        ExifInterface.TAG_ORIENTATION,
        ExifInterface.ORIENTATION_NORMAL
      )
      if (orientation == ExifInterface.ORIENTATION_ROTATE_90) {
        result = 90
      } else if (orientation == ExifInterface.ORIENTATION_ROTATE_180) {
        result = 180
      } else if (orientation == ExifInterface.ORIENTATION_ROTATE_270) {
        result = 270
      }
    }
    return result
  }

  fun getExifData(c: Context, bitmap: Bitmap?, imageUri: Uri): ExifWrapper {
    var stream: InputStream? = null
    try {
      stream = c.getContentResolver().openInputStream(imageUri)
      val exifInterface = ExifInterface(stream!!)
      return ExifWrapper(exifInterface)
    } catch (ex: IOException) {
      Logger.error("Error loading exif data from image", ex)
    } finally {
      if (stream != null) {
        try {
          stream.close()
        } catch (ignored: IOException) {
        }
      }
    }
    return ExifWrapper(null)
  }
}
