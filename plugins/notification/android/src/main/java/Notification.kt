package app.tauri.notification

import android.content.ContentResolver
import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import org.json.JSONException
import org.json.JSONObject

class Notification {
  var title: String? = null
  var body: String? = null
  var largeBody: String? = null
  var summary: String? = null
  var id: Int = 0
  private var sound: String? = null
  private var smallIcon: String? = null
  private var largeIcon: String? = null
  var iconColor: String? = null
  var actionTypeId: String? = null
  var group: String? = null
  var inboxLines: List<String>? = null
  var isGroupSummary = false
  var isOngoing = false
  var isAutoCancel = false
  var extra: JSObject? = null
  var attachments: List<NotificationAttachment>? = null
  var schedule: NotificationSchedule? = null
  var channelId: String? = null
  var source: JSObject? = null
  var visibility: Int? = null
  var number: Int? = null

  fun getSound(context: Context, defaultSound: Int): String? {
    var soundPath: String? = null
    var resId: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
    val name = AssetUtils.getResourceBaseName(sound)
    if (name != null) {
      resId = AssetUtils.getResourceID(context, name, "raw")
    }
    if (resId == AssetUtils.RESOURCE_ID_ZERO_VALUE) {
      resId = defaultSound
    }
    if (resId != AssetUtils.RESOURCE_ID_ZERO_VALUE) {
      soundPath =
        ContentResolver.SCHEME_ANDROID_RESOURCE + "://" + context.packageName + "/" + resId
    }
    return soundPath
  }

  fun setSound(sound: String?) {
    this.sound = sound
  }

  fun setSmallIcon(smallIcon: String?) {
    this.smallIcon = AssetUtils.getResourceBaseName(smallIcon)
  }

  fun setLargeIcon(largeIcon: String?) {
    this.largeIcon = AssetUtils.getResourceBaseName(largeIcon)
  }

  fun getIconColor(globalColor: String): String {
    // use the one defined local before trying for a globally defined color
    return iconColor ?: globalColor
  }

  fun getSmallIcon(context: Context, defaultIcon: Int): Int {
    var resId: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
    if (smallIcon != null) {
      resId = AssetUtils.getResourceID(context, smallIcon, "drawable")
    }
    if (resId == AssetUtils.RESOURCE_ID_ZERO_VALUE) {
      resId = defaultIcon
    }
    return resId
  }

  fun getLargeIcon(context: Context): Bitmap? {
    if (largeIcon != null) {
      val resId: Int = AssetUtils.getResourceID(context, largeIcon, "drawable")
      return BitmapFactory.decodeResource(context.resources, resId)
    }
    return null
  }

  val isScheduled = schedule != null

  companion object {
    fun fromJson(jsonNotification: JSONObject): Notification {
      val notification: JSObject = try {
        val identifier = jsonNotification.getLong("id")
        if (identifier > Int.MAX_VALUE || identifier < Int.MIN_VALUE) {
          throw Exception("The notification identifier should be a 32-bit integer")
        }
        JSObject.fromJSONObject(jsonNotification)
      } catch (e: JSONException) {
        throw Exception("Invalid notification JSON object", e)
      }
      return fromJSObject(notification)
    }

    fun fromJSObject(jsonObject: JSObject): Notification {
      val notification = Notification()
      notification.source = jsonObject
      notification.id = jsonObject.getInteger("id") ?: throw Exception("Missing notification identifier")
      notification.body = jsonObject.getString("body", null)
      notification.largeBody = jsonObject.getString("largeBody", null)
      notification.summary = jsonObject.getString("summary", null)
      notification.actionTypeId = jsonObject.getString("actionTypeId", null)
      notification.group = jsonObject.getString("group", null)
      notification.setSound(jsonObject.getString("sound", null))
      notification.title = jsonObject.getString("title", null)
      notification.setSmallIcon(jsonObject.getString("icon", null))
      notification.setLargeIcon(jsonObject.getString("largeIcon", null))
      notification.iconColor = jsonObject.getString("iconColor", null)
      notification.attachments = NotificationAttachment.getAttachments(jsonObject)
      notification.isGroupSummary = jsonObject.getBoolean("groupSummary", false)
      notification.channelId = jsonObject.getString("channelId", null)
      val schedule = jsonObject.getJSObject("schedule")
      if (schedule != null) {
        notification.schedule = NotificationSchedule(schedule)
      }
      notification.extra = jsonObject.getJSObject("extra")
      notification.isOngoing = jsonObject.getBoolean("ongoing", false)
      notification.isAutoCancel = jsonObject.getBoolean("autoCancel", true)
      notification.visibility = jsonObject.getInteger("visibility")
      notification.number = jsonObject.getInteger("number")
      try {
        val inboxLines = jsonObject.getJSONArray("inboxLines")
        val inboxStringList: MutableList<String> = ArrayList()
        for (i in 0 until inboxLines.length()) {
          inboxStringList.add(inboxLines.getString(i))
        }
        notification.inboxLines = inboxStringList
      } catch (_: Exception) {
      }
      return notification
    }

    fun buildNotificationPendingList(notifications: List<Notification>): JSObject {
      val result = JSObject()
      val jsArray = JSArray()
      for (notification in notifications) {
        val jsNotification = JSObject()
        jsNotification.put("id", notification.id)
        jsNotification.put("title", notification.title)
        jsNotification.put("body", notification.body)
        val schedule = notification.schedule
        if (schedule != null) {
          val jsSchedule = JSObject()
          jsSchedule.put("kind", schedule.scheduleObj.getString("kind", null))
          jsSchedule.put("data", schedule.scheduleObj.getJSObject("data"))
          jsNotification.put("schedule", jsSchedule)
        }
        jsNotification.put("extra", notification.extra)
        jsArray.put(jsNotification)
      }
      result.put("notifications", jsArray)
      return result
    }
  }
}