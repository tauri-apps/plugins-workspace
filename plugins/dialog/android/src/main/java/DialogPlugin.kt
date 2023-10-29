// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.dialog

import android.app.Activity
import android.app.AlertDialog
import android.content.Intent
import android.net.Uri
import android.os.Handler
import android.os.Looper
import androidx.activity.result.ActivityResult
import app.tauri.Logger
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class Filter {
  lateinit var extensions: Array<String>
}

@InvokeArg
class FilePickerOptions {
  lateinit var filters: Array<Filter>
  var multiple: Boolean? = null
  var readData: Boolean? = null
}

@InvokeArg
class MessageOptions {
  var title: String? = null
  lateinit var message: String
  var okButtonLabel: String? = null
  var cancelButtonLabel: String? = null
}

@TauriPlugin
class DialogPlugin(private val activity: Activity): Plugin(activity) {
  var filePickerOptions: FilePickerOptions? = null

  @Command
  fun showFilePicker(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(FilePickerOptions::class.java)
      val parsedTypes = parseFiltersOption(args.filters)
      
      val intent = if (parsedTypes.isNotEmpty()) {
        val intent = Intent(Intent.ACTION_PICK)
        intent.putExtra(Intent.EXTRA_MIME_TYPES, parsedTypes)
        
        var uniqueMimeType = true
        var mimeKind: String? = null
        for (mime in parsedTypes) {
          val kind = mime.split("/")[0]
          if (mimeKind == null) {
            mimeKind = kind
          } else if (mimeKind != kind) {
            uniqueMimeType = false
          }
        }
        
        intent.type = if (uniqueMimeType) Intent.normalizeMimeType("$mimeKind/*") else "*/*"
        intent
      } else {
        val intent = Intent(Intent.ACTION_GET_CONTENT)
        intent.addCategory(Intent.CATEGORY_OPENABLE)
        intent.type = "*/*"
        intent
      }

      intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, args.multiple ?: false)
      
      startActivityForResult(invoke, intent, "filePickerResult")
    } catch (ex: Exception) {
      val message = ex.message ?: "Failed to pick file"
      Logger.error(message)
      invoke.reject(message)
    }
  }

  @ActivityCallback
  fun filePickerResult(invoke: Invoke, result: ActivityResult) {
    try {
      when (result.resultCode) {
        Activity.RESULT_OK -> {
          val callResult = createPickFilesResult(result.data, filePickerOptions?.readData ?: false)
          invoke.resolve(callResult)
        }
        Activity.RESULT_CANCELED -> invoke.reject("File picker cancelled")
        else -> invoke.reject("Failed to pick files")
      }
    } catch (ex: java.lang.Exception) {
      val message = ex.message ?: "Failed to read file pick result"
      Logger.error(message)
      invoke.reject(message)
    }
  }

  private fun createPickFilesResult(data: Intent?, readData: Boolean): JSObject {
    val callResult = JSObject()
    val filesResultList: MutableList<JSObject> = ArrayList()
    if (data == null) {
      callResult.put("files", JSArray.from(filesResultList))
      return callResult
    }
    val uris: MutableList<Uri?> = ArrayList()
    if (data.clipData == null) {
      val uri: Uri? = data.data
      uris.add(uri)
    } else {
      for (i in 0 until data.clipData!!.itemCount) {
        val uri: Uri = data.clipData!!.getItemAt(i).uri
        uris.add(uri)
      }
    }
    for (i in uris.indices) {
      val uri = uris[i] ?: continue
      val fileResult = JSObject()
      if (readData) {
        fileResult.put("base64Data", FilePickerUtils.getDataFromUri(activity, uri))
      }
      val duration = FilePickerUtils.getDurationFromUri(activity, uri)
      if (duration != null) {
        fileResult.put("duration", duration)
      }
      val resolution = FilePickerUtils.getHeightAndWidthFromUri(activity, uri)
      if (resolution != null) {
        fileResult.put("height", resolution.height)
        fileResult.put("width", resolution.width)
      }
      fileResult.put("mimeType", FilePickerUtils.getMimeTypeFromUri(activity, uri))
      val modifiedAt = FilePickerUtils.getModifiedAtFromUri(activity, uri)
      if (modifiedAt != null) {
        fileResult.put("modifiedAt", modifiedAt)
      }
      fileResult.put("name", FilePickerUtils.getNameFromUri(activity, uri))
      fileResult.put("path", FilePickerUtils.getPathFromUri(uri))
      fileResult.put("size", FilePickerUtils.getSizeFromUri(activity, uri))
      filesResultList.add(fileResult)
    }
    callResult.put("files", JSArray.from(filesResultList.toTypedArray()))
    return callResult
  }
  
  private fun parseFiltersOption(filters: Array<Filter>): Array<String> {
    val mimeTypes = mutableListOf<String>()
    for (filter in filters) {
      for (mime in filter.extensions) {
        mimeTypes.add(if (mime == "text/csv") "text/comma-separated-values" else mime)
      }
    }
    return mimeTypes.toTypedArray()
  }
  
  @Command
  fun showMessageDialog(invoke: Invoke) {
    val args = invoke.parseArgs(MessageOptions::class.java)
    
    if (activity.isFinishing) {
      invoke.reject("App is finishing")
      return
    }

    val handler = { cancelled: Boolean, value: Boolean ->
      val ret = JSObject()
      ret.put("cancelled", cancelled)
      ret.put("value", value)
      invoke.resolve(ret)
    }

    Handler(Looper.getMainLooper())
      .post {
        val builder = AlertDialog.Builder(activity)
        
        if (args.title != null) {
          builder.setTitle(args.title)
        }
        builder
          .setMessage(args.message)
          .setPositiveButton(
            args.okButtonLabel ?: "OK"
          ) { dialog, _ ->
            dialog.dismiss()
            handler(false, true)
          }
          .setNegativeButton(
            args.cancelButtonLabel ?: "Cancel"
          ) { dialog, _ ->
            dialog.dismiss()
            handler(false, false)
          }
          .setOnCancelListener { dialog ->
            dialog.dismiss()
            handler(true, false)
          }
        val dialog = builder.create()
        dialog.show()
      }
  }
}