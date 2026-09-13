// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.clipboard

import android.app.Activity
import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import android.text.Html
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.core.JsonProcessingException
import com.fasterxml.jackson.databind.DeserializationContext
import com.fasterxml.jackson.databind.JsonDeserializer
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.SerializerProvider
import com.fasterxml.jackson.databind.annotation.JsonDeserialize
import com.fasterxml.jackson.databind.annotation.JsonSerialize
import com.fasterxml.jackson.databind.ser.std.StdSerializer
import java.io.IOException

@InvokeArg
@JsonDeserialize(using = WriteOptionsDeserializer::class)
sealed class WriteOptions {
  @JsonDeserialize
  class PlainText: WriteOptions() {
    lateinit var text: String
    var label: String? = null
  }
}

@JsonSerialize(using = ReadClipDataSerializer::class)
sealed class ReadClipData {
  class PlainText: ReadClipData() {
    lateinit var text: String
  }
}

internal class ReadClipDataSerializer @JvmOverloads constructor(t: Class<ReadClipData>? = null) :
  StdSerializer<ReadClipData>(t) {
  @Throws(IOException::class, JsonProcessingException::class)
  override fun serialize(
    value: ReadClipData, jgen: JsonGenerator, provider: SerializerProvider
  ) {
    jgen.writeStartObject()
    when (value) {
      is ReadClipData.PlainText -> {
        jgen.writeObjectFieldStart("plainText")

        jgen.writeStringField("text", value.text)

        jgen.writeEndObject()
      }
      else -> {
        throw Exception("unimplemented ReadClipData")
      }
    }

    jgen.writeEndObject()
  }
}

internal class WriteOptionsDeserializer: JsonDeserializer<WriteOptions>() {
  override fun deserialize(
    jsonParser: JsonParser,
    deserializationContext: DeserializationContext
  ): WriteOptions {
    val node: JsonNode = jsonParser.codec.readTree(jsonParser)
    node.get("plainText")?.let {
      return jsonParser.codec.treeToValue(it, WriteOptions.PlainText::class.java)
    } ?: run {
      throw Error("unknown write options $node")
    }
  }
}

@TauriPlugin
class ClipboardPlugin(private val activity: Activity) : Plugin(activity) {
  private val manager: ClipboardManager =
    activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager

  @Command
  @Suppress("MoveVariableDeclarationIntoWhen")
  fun writeText(invoke: Invoke) {
    val args = invoke.parseArgs(WriteOptions::class.java)

    val clipData = when (args) {
      is WriteOptions.PlainText -> {
        ClipData.newPlainText(args.label, args.text)
      } else -> {
        invoke.reject("unimplemented WriteOptions")
        return
      }

    }

    manager.setPrimaryClip(clipData)

    invoke.resolve()
  }


  @Command
  fun readText(invoke: Invoke) {
    // Use .let for safely handling nullable primaryClip
    val data = manager.primaryClip?.let { clip ->
      // Ensure the clipboard actually has something
      if (clip.itemCount == 0) {
        invoke.reject("Clipboard is empty")
        return
      }

      val item = clip.getItemAt(0)
      val description = clip.description

      // Use a when statement to gracefully handle different MIME types.
      // HTML is prioritized as it often contains richer information.
      when {
        // Case 1: Clipboard contains HTML text
        description.hasMimeType(ClipDescription.MIMETYPE_TEXT_HTML) -> {
          val htmlText = item.htmlText
          // Convert HTML to plain text, also handling the case where htmlText might be null
          val plainText = if (htmlText != null) {
            // Use different fromHtml methods depending on the Android version, which is a best practice
            Html.fromHtml(htmlText, Html.FROM_HTML_MODE_COMPACT).toString()
          } else {
            // If htmlText is null, fall back to plain text
            item.text?.toString() ?: ""
          }
          ReadClipData.PlainText().apply { text = plainText }
        }

        // Case 2: Clipboard contains only plain text
        description.hasMimeType(ClipDescription.MIMETYPE_TEXT_PLAIN) -> {
          val plainText = item.text?.toString() ?: ""
          ReadClipData.PlainText().apply { text = plainText }
        }

        // Case 3: The content is not a text type we can handle (e.g., an image or a URI)
        else -> {
          invoke.reject("Clipboard content is not plain text or HTML")
          return
        }
      }
    }

    // The final call is determined by whether data is null or not
    if (data != null) {
      invoke.resolveObject(data)
    } else {
      // if manager.primaryClip is null, data will be null as well
      invoke.reject("Clipboard is empty")
    }
  }

  @Command
  fun clear(invoke: Invoke) {
      if (manager.hasPrimaryClip()) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
          manager.clearPrimaryClip()
        } else {
          manager.setPrimaryClip(ClipData.newPlainText("", ""))
        }
      }
      invoke.resolve()
  }
}
