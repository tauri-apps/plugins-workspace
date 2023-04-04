// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.dialog

import android.app.Activity
import android.app.AlertDialog
import android.os.Handler
import android.os.Looper
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@TauriPlugin
class DialogPlugin(private val activity: Activity): Plugin(activity) {
  @Command
  fun showMessageDialog(invoke: Invoke) {
    val title = invoke.getString("title")
    val message = invoke.getString("message")
    val okButtonLabel = invoke.getString("okButtonLabel", "OK")
    val cancelButtonLabel = invoke.getString("cancelButtonLabel", "Cancel")
    
    if (message == null) {
      invoke.reject("The `message` argument is required")
      return
    }
    
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
        
        if (title != null) {
          builder.setTitle(title)
        }
        builder
          .setMessage(message)
          .setPositiveButton(
            okButtonLabel
          ) { dialog, _ ->
            dialog.dismiss()
            handler(false, true)
          }
          .setNegativeButton(
            cancelButtonLabel
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