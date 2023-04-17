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
    val data = invoke.getObject("options")
    if (data == null) {
      invoke.reject("Missing `options` input")
      return
    }
    val kind = invoke.getString("kind", "")

    val clipData = when (kind) {
      "PlainText" -> {
        val label = data.getString("label", "")
        val text = data.getString("text", "")
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
        return
      }
    } else {
      Pair("PlainText", "")
    }

    val response = JSObject()
    response.put("kind", kind)
    response.put("options", options)
    invoke.resolve(response)
  }
}
