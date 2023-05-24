// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.clipboard

import android.R.attr.value
import android.app.Activity
import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin


@TauriPlugin
class ClipboardPlugin(private val activity: Activity) : Plugin(activity) {
  private val manager: ClipboardManager =
    activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager

  @Command
  @Suppress("MoveVariableDeclarationIntoWhen")
  fun write(invoke: Invoke) {
    val options = invoke.getObject("options")
    if (options == null) {
      invoke.reject("Missing `options` input")
      return
    }
    val kind = invoke.getString("kind", "")

    val clipData = when (kind) {
      "PlainText" -> {
        val label = options.getString("label", "")
        val text = options.getString("text", "")
        ClipData.newPlainText(label, text)
      }

      else -> {
        invoke.reject("Unknown kind $kind")
        return
      }
    }

    manager.setPrimaryClip(clipData)

    invoke.resolve()
  }

  @Command
  fun read(invoke: Invoke) {
    val (kind, options) = if (manager.hasPrimaryClip()) {
      if (manager.primaryClipDescription?.hasMimeType(ClipDescription.MIMETYPE_TEXT_PLAIN) == true) {
        val item: ClipData.Item = manager.primaryClip!!.getItemAt(0)
        Pair("PlainText", item.text)
      } else {
        // TODO
        invoke.reject("Clipboard content reader not implemented")
        return
      }
    } else {
      invoke.reject("Clipboard is empty")
        return
    }

    val response = JSObject()
    response.put("kind", kind)
    response.put("options", options)
    invoke.resolve(response)
  }
}
