// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import app.tauri.Logger
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import org.json.JSONObject

class NotificationAction() {
  var id: String? = null
  var title: String? = null
  var input = false

  constructor(id: String?, title: String?, input: Boolean): this() {
    this.id = id
    this.title = title
    this.input = input
  }

  companion object {
    fun buildTypes(types: JSArray): Map<String, List<NotificationAction>> {
      val actionTypeMap: MutableMap<String, List<NotificationAction>> = HashMap()
      try {
        val objects: List<JSONObject> = types.toList()
        for (obj in objects) {
          val jsObject = JSObject.fromJSONObject(
            obj
          )
          val actionGroupId = jsObject.getString("id")
          val actions = jsObject.getJSONArray("actions")
          val typesArray = mutableListOf<NotificationAction>()
          for (i in 0 until actions.length()) {
            val notificationAction = NotificationAction()
            val action = JSObject.fromJSONObject(actions.getJSONObject(i))
            notificationAction.id = action.getString("id")
            notificationAction.title = action.getString("title")
            notificationAction.input = action.getBoolean("input")
            typesArray.add(notificationAction)
          }
          actionTypeMap[actionGroupId] = typesArray.toList()
        }
      } catch (e: Exception) {
        Logger.error(Logger.tags("Notification"), "Error when building action types", e)
      }
      return actionTypeMap
    }
  }
}