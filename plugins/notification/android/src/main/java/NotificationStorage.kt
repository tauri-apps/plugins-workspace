// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.content.Context
import android.content.SharedPreferences
import com.fasterxml.jackson.databind.ObjectMapper
import org.json.JSONException
import java.lang.Exception

// Key for private preferences
private const val NOTIFICATION_STORE_ID = "NOTIFICATION_STORE"
// Key used to save action types
private const val ACTION_TYPES_ID = "ACTION_TYPE_STORE"

class NotificationStorage(private val context: Context, private val jsonMapper: ObjectMapper) {
  fun appendNotifications(localNotifications: List<Notification>) {
    val storage = getStorage(NOTIFICATION_STORE_ID)
    val editor = storage.edit()
    for (request in localNotifications) {
      if (request.schedule != null) {
        val key: String = request.id.toString()
        editor.putString(key, request.sourceJson.toString())
      }
    }
    editor.apply()
  }

  fun getSavedNotificationIds(): List<String> {
    val storage = getStorage(NOTIFICATION_STORE_ID)
    val all = storage.all
    return if (all != null) {
      ArrayList(all.keys)
    } else ArrayList()
  }

  fun getSavedNotifications(): List<Notification> {
    val storage = getStorage(NOTIFICATION_STORE_ID)
    val all = storage.all
    if (all != null) {
      val notifications = ArrayList<Notification>()
      for (key in all.keys) {
        val notificationString = all[key] as String?
        try {
          val notification = jsonMapper.readValue(notificationString, Notification::class.java)
          notifications.add(notification)
        } catch (_: Exception) { }
      }
      return notifications
    }
    return ArrayList()
  }

  fun getSavedNotification(key: String): Notification? {
    val storage = getStorage(NOTIFICATION_STORE_ID)
    val notificationString = try {
      storage.getString(key, null)
    } catch (ex: ClassCastException) {
      return null
    } ?: return null

    return try {
      jsonMapper.readValue(notificationString, Notification::class.java)
    } catch (ex: JSONException) {
      null
    }
  }

  fun deleteNotification(id: String?) {
    val editor = getStorage(NOTIFICATION_STORE_ID).edit()
    editor.remove(id)
    editor.apply()
  }

  private fun getStorage(key: String): SharedPreferences {
    return context.getSharedPreferences(key, Context.MODE_PRIVATE)
  }

  fun writeActionGroup(actions: List<ActionType>) {
    for (type in actions) {
      val i = type.id
      val editor = getStorage(ACTION_TYPES_ID + type.id).edit()
      editor.clear()
      editor.putInt("count", type.actions.size)
      for (action in type.actions) {
        editor.putString("id$i", action.id)
        editor.putString("title$i", action.title)
        editor.putBoolean("input$i", action.input ?: false)
      }
      editor.apply()
    }
  }

  fun getActionGroup(forId: String): Array<NotificationAction?> {
    val storage = getStorage(ACTION_TYPES_ID + forId)
    val count = storage.getInt("count", 0)
    val actions: Array<NotificationAction?> = arrayOfNulls(count)
    for (i in 0 until count) {
      val id = storage.getString("id$i", "")
      val title = storage.getString("title$i", "")
      val input = storage.getBoolean("input$i", false)

      val action = NotificationAction()
      action.id = id ?: ""
      action.title = title
      action.input = input
      actions[i] = action
    }
    return actions
  }
}