package app.tauri.notification

import android.content.ContentResolver
import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import app.tauri.Logger
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import org.json.JSONException
import org.json.JSONObject
import java.text.ParseException

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
  var source: String? = null

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

  override fun toString(): String {
    return "Notification{" +
      "title='" +
      title +
      '\'' +
      ", body='" +
      body +
      '\'' +
      ", id=" +
      id +
      ", sound='" +
      sound +
      '\'' +
      ", smallIcon='" +
      smallIcon +
      '\'' +
      ", iconColor='" +
      iconColor +
      '\'' +
      ", actionTypeId='" +
      actionTypeId +
      '\'' +
      ", group='" +
      group +
      '\'' +
      ", extra=" +
      extra +
      ", attachments=" +
      attachments +
      ", schedule=" +
      schedule +
      ", groupSummary=" +
      isGroupSummary +
      ", ongoing=" +
      isOngoing +
      ", autoCancel=" +
      isAutoCancel +
      '}'
  }

  override fun equals(other: Any?): Boolean {
    if (this === other) return true
    if (other == null || javaClass != other.javaClass) return false
    val that = other as Notification
    if (if (title != null) title != that.title else that.title != null) return false
    if (if (body != null) body != that.body else that.body != null) return false
    if (if (largeBody != null) largeBody != that.largeBody else that.largeBody != null) return false
    if (id != that.id) return false
    if (if (sound != null) sound != that.sound else that.sound != null) return false
    if (if (smallIcon != null) smallIcon != that.smallIcon else that.smallIcon != null) return false
    if (if (largeIcon != null) largeIcon != that.largeIcon else that.largeIcon != null) return false
    if (if (iconColor != null) iconColor != that.iconColor else that.iconColor != null) return false
    if (if (actionTypeId != null) actionTypeId != that.actionTypeId else that.actionTypeId != null) return false
    if (if (group != null) group != that.group else that.group != null) return false
    if (if (extra != null) extra != that.extra else that.extra != null) return false
    if (if (attachments != null) attachments != that.attachments else that.attachments != null) return false
    if (if (inboxLines != null) inboxLines != that.inboxLines else that.inboxLines != null) return false
    if (isGroupSummary != that.isGroupSummary) return false
    if (isOngoing != that.isOngoing) return false
    if (isAutoCancel != that.isAutoCancel) return false
    return if (schedule != null) schedule?.equals(that.schedule) ?: false else that.schedule == null
  }

  override fun hashCode(): Int {
    var result = if (title != null) title.hashCode() else 0
    result = 31 * result + if (body != null) body.hashCode() else 0
    result = 31 * result + id.hashCode()
    result = 31 * result + if (sound != null) sound.hashCode() else 0
    result = 31 * result + if (smallIcon != null) smallIcon.hashCode() else 0
    result = 31 * result + if (iconColor != null) iconColor.hashCode() else 0
    result = 31 * result + if (actionTypeId != null) actionTypeId.hashCode() else 0
    result = 31 * result + if (group != null) group.hashCode() else 0
    result = 31 * result + java.lang.Boolean.hashCode(isGroupSummary)
    result = 31 * result + java.lang.Boolean.hashCode(isOngoing)
    result = 31 * result + java.lang.Boolean.hashCode(isAutoCancel)
    result = 31 * result + if (extra != null) extra.hashCode() else 0
    result = 31 * result + if (attachments != null) attachments.hashCode() else 0
    result = 31 * result + if (schedule != null) schedule.hashCode() else 0
    return result
  }

  fun setExtraFromString(extraFromString: String) {
    try {
      val jsonObject = JSONObject(extraFromString)
      extra = JSObject.fromJSONObject(jsonObject)
    } catch (e: JSONException) {
      Logger.error(Logger.tags("Notification"), "Cannot rebuild extra data", e)
    }
  }

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
      notification.source = jsonObject.toString()
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

    fun getNotificationPendingList(invoke: Invoke): List<Int>? {
      var notifications: List<JSONObject>? = null
      try {
        notifications = invoke.getArray("notifications", JSArray()).toList()
      } catch (_: JSONException) {
      }
      if (notifications.isNullOrEmpty()) {
        invoke.reject("Must provide notifications array as notifications option")
        return null
      }
      val notificationsList: MutableList<Int> = ArrayList(notifications.size)
      for (notificationToCancel in notifications) {
        try {
          notificationsList.add(notificationToCancel.getInt("id"))
        } catch (_: JSONException) {
        }
      }
      return notificationsList
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