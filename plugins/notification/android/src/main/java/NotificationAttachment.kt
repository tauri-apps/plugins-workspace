// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import app.tauri.plugin.JSObject
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

class NotificationAttachment {
  var id: String? = null
  var url: String? = null
  var options: JSONObject? = null

  companion object {
    fun getAttachments(notification: JSObject): List<NotificationAttachment> {
      val attachmentsList: MutableList<NotificationAttachment> = ArrayList()
      var attachments: JSONArray? = null
      try {
        attachments = notification.getJSONArray("attachments")
      } catch (_: Exception) {
      }
      if (attachments != null) {
        for (i in 0 until attachments.length()) {
          val newAttachment = NotificationAttachment()
          var jsonObject: JSONObject? = null
          try {
            jsonObject = attachments.getJSONObject(i)
          } catch (e: JSONException) {
          }
          if (jsonObject != null) {
            var jsObject: JSObject? = null
            try {
              jsObject = JSObject.fromJSONObject(jsonObject)
            } catch (_: JSONException) {
            }
            newAttachment.id = jsObject!!.getString("id")
            newAttachment.url = jsObject.getString("url")
            try {
              newAttachment.options = jsObject.getJSONObject("options")
            } catch (_: JSONException) {
            }
            attachmentsList.add(newAttachment)
          }
        }
      }
      return attachmentsList
    }
  }
}