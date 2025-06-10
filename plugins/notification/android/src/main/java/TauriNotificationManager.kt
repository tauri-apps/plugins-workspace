// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.annotation.SuppressLint
import android.app.Activity
import android.app.AlarmManager
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.ContentResolver
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.media.AudioAttributes
import android.net.Uri
import android.os.Build
import android.os.Build.VERSION.SDK_INT
import android.os.UserManager
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.RemoteInput
import app.tauri.Logger
import app.tauri.plugin.JSObject
import app.tauri.plugin.PluginManager
import com.fasterxml.jackson.databind.ObjectMapper
import org.json.JSONException
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date

// Action constants
const val NOTIFICATION_INTENT_KEY = "NotificationId"
const val NOTIFICATION_OBJ_INTENT_KEY = "LocalNotficationObject"
const val ACTION_INTENT_KEY = "NotificationUserAction"
const val NOTIFICATION_IS_REMOVABLE_KEY = "NotificationRepeating"
const val REMOTE_INPUT_KEY = "NotificationRemoteInput"
const val DEFAULT_NOTIFICATION_CHANNEL_ID = "default"
const val DEFAULT_PRESS_ACTION = "tap"

class TauriNotificationManager(
  private val storage: NotificationStorage,
  private val activity: Activity?,
  private val context: Context,
  private val config: PluginConfig?
) {
  private var defaultSoundID: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
  private var defaultSmallIconID: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE

  fun handleNotificationActionPerformed(
    data: Intent,
    notificationStorage: NotificationStorage
  ): JSObject? {
    Logger.debug(Logger.tags("Notification"), "Notification received: " + data.dataString)
    val notificationId =
      data.getIntExtra(NOTIFICATION_INTENT_KEY, Int.MIN_VALUE)
    if (notificationId == Int.MIN_VALUE) {
      Logger.debug(Logger.tags("Notification"), "Activity started without notification attached")
      return null
    }
    val isRemovable =
      data.getBooleanExtra(NOTIFICATION_IS_REMOVABLE_KEY, true)
    if (isRemovable) {
      notificationStorage.deleteNotification(notificationId.toString())
    }
    val dataJson = JSObject()
    val results = RemoteInput.getResultsFromIntent(data)
    val input = results?.getCharSequence(REMOTE_INPUT_KEY)
    dataJson.put("inputValue", input?.toString())
    val menuAction = data.getStringExtra(ACTION_INTENT_KEY)
    dismissVisibleNotification(notificationId)
    dataJson.put("actionId", menuAction)
    var request: JSONObject? = null
    try {
      val notificationJsonString =
        data.getStringExtra(NOTIFICATION_OBJ_INTENT_KEY)
      if (notificationJsonString != null) {
        request = JSObject(notificationJsonString)
      }
    } catch (_: JSONException) {
    }
    dataJson.put("notification", request)
    return dataJson
  }

  /**
   * Create notification channel
   */
  fun createNotificationChannel() {
    // Create the NotificationChannel, but only on API 26+ because
    // the NotificationChannel class is new and not in the support library
    if (SDK_INT >= Build.VERSION_CODES.O) {
      val name: CharSequence = "Default"
      val description = "Default"
      val importance = NotificationManager.IMPORTANCE_DEFAULT
      val channel = NotificationChannel(DEFAULT_NOTIFICATION_CHANNEL_ID, name, importance)
      channel.description = description
      val audioAttributes = AudioAttributes.Builder()
        .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
        .setUsage(AudioAttributes.USAGE_ALARM)
        .build()
      val soundUri = getDefaultSoundUrl(context)
      if (soundUri != null) {
        channel.setSound(soundUri, audioAttributes)
      }
      // Register the channel with the system; you can't change the importance
      // or other notification behaviors after this
      val notificationManager = context.getSystemService(
        NotificationManager::class.java
      )
      notificationManager.createNotificationChannel(channel)
    }
  }

  private fun trigger(notificationManager: NotificationManagerCompat, notification: Notification): Int {
    dismissVisibleNotification(notification.id)
    cancelTimerForNotification(notification.id)
    buildNotification(notificationManager, notification)

    return notification.id
  }

  fun schedule(notification: Notification): Int {
    val notificationManager = NotificationManagerCompat.from(context)
    return trigger(notificationManager, notification)
  }

  fun schedule(notifications: List<Notification>): List<Int> {
    val ids = mutableListOf<Int>()
    val notificationManager = NotificationManagerCompat.from(context)

    for (notification in notifications) {
      val id = trigger(notificationManager, notification)
      ids.add(id)
    }

    return ids
  }

  // TODO Progressbar support
  // TODO System categories (DO_NOT_DISTURB etc.)
  // TODO use NotificationCompat.MessagingStyle for latest API
  // TODO expandable notification NotificationCompat.MessagingStyle
  // TODO media style notification support NotificationCompat.MediaStyle
  @SuppressLint("MissingPermission")
  private fun buildNotification(
    notificationManager: NotificationManagerCompat,
    notification: Notification,
  ) {
    val channelId = notification.channelId ?: DEFAULT_NOTIFICATION_CHANNEL_ID
    val mBuilder = NotificationCompat.Builder(
      context, channelId
    )
      .setContentTitle(notification.title)
      .setContentText(notification.body)
      .setAutoCancel(notification.isAutoCancel)
      .setOngoing(notification.isOngoing)
      .setPriority(NotificationCompat.PRIORITY_DEFAULT)
      .setGroupSummary(notification.isGroupSummary)
    if (notification.largeBody != null) {
      // support multiline text
      mBuilder.setStyle(
        NotificationCompat.BigTextStyle()
          .bigText(notification.largeBody)
          .setSummaryText(notification.summary)
      )
    } else if (notification.inboxLines != null) {
      val inboxStyle = NotificationCompat.InboxStyle()
      for (line in notification.inboxLines ?: listOf()) {
        inboxStyle.addLine(line)
      }
      inboxStyle.setBigContentTitle(notification.title)
      inboxStyle.setSummaryText(notification.summary)
      mBuilder.setStyle(inboxStyle)
    }
    val sound = notification.getSound(context, getDefaultSound(context))
    if (sound != null) {
      val soundUri = Uri.parse(sound)
      // Grant permission to use sound
      context.grantUriPermission(
        "com.android.systemui",
        soundUri,
        Intent.FLAG_GRANT_READ_URI_PERMISSION
      )
      mBuilder.setSound(soundUri)
      mBuilder.setDefaults(android.app.Notification.DEFAULT_VIBRATE or android.app.Notification.DEFAULT_LIGHTS)
    } else {
      mBuilder.setDefaults(android.app.Notification.DEFAULT_ALL)
    }
    val group = notification.group
    if (group != null) {
      mBuilder.setGroup(group)
      if (notification.isGroupSummary) {
        mBuilder.setSubText(notification.summary)
      }
    }
    mBuilder.setVisibility(notification.visibility ?: NotificationCompat.VISIBILITY_PRIVATE)
    mBuilder.setOnlyAlertOnce(true)
    mBuilder.setSmallIcon(notification.getSmallIcon(context, getDefaultSmallIcon(context)))
    mBuilder.setLargeIcon(notification.getLargeIcon(context))
    val iconColor = notification.getIconColor(config?.iconColor ?: "")
    if (iconColor.isNotEmpty()) {
      try {
        mBuilder.color = Color.parseColor(iconColor)
      } catch (ex: IllegalArgumentException) {
        throw Exception("Invalid color provided. Must be a hex string (ex: #ff0000")
      }
    }
    createActionIntents(notification, mBuilder)
    // notificationId is a unique int for each notification that you must define
    val buildNotification = mBuilder.build()
    if (notification.schedule != null) {
      triggerScheduledNotification(buildNotification, notification)
    } else {
      notificationManager.notify(notification.id, buildNotification)
      try {
        NotificationPlugin.triggerNotification(notification)
      } catch (_: JSONException) {
      }
    }
  }

  // Create intents for open/dismiss actions
  private fun createActionIntents(
    notification: Notification,
    mBuilder: NotificationCompat.Builder
  ) {
    // Open intent
    val intent = buildIntent(notification, DEFAULT_PRESS_ACTION)
    var flags = PendingIntent.FLAG_CANCEL_CURRENT
    if (SDK_INT >= Build.VERSION_CODES.S) {
      flags = flags or PendingIntent.FLAG_MUTABLE
    }
    val pendingIntent = PendingIntent.getActivity(context, notification.id, intent, flags)
    mBuilder.setContentIntent(pendingIntent)

    // Build action types
    val actionTypeId = notification.actionTypeId
    if (actionTypeId != null) {
      val actionGroup = storage.getActionGroup(actionTypeId)
      for (notificationAction in actionGroup) {
        // TODO Add custom icons to actions
        val actionIntent = buildIntent(notification, notificationAction!!.id)
        val actionPendingIntent = PendingIntent.getActivity(
          context,
          (notification.id) + notificationAction.id.hashCode(),
          actionIntent,
          flags
        )
        val actionBuilder: NotificationCompat.Action.Builder = NotificationCompat.Action.Builder(
          R.drawable.ic_transparent,
          notificationAction.title,
          actionPendingIntent
        )
        if (notificationAction.input == true) {
          val remoteInput = RemoteInput.Builder(REMOTE_INPUT_KEY).setLabel(
            notificationAction.title
          ).build()
          actionBuilder.addRemoteInput(remoteInput)
        }
        mBuilder.addAction(actionBuilder.build())
      }
    }

    // Dismiss intent
    val dissmissIntent = Intent(
      context,
      NotificationDismissReceiver::class.java
    )
    dissmissIntent.flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK
    dissmissIntent.putExtra(NOTIFICATION_INTENT_KEY, notification.id)
    dissmissIntent.putExtra(ACTION_INTENT_KEY, "dismiss")
    val schedule = notification.schedule
    dissmissIntent.putExtra(
      NOTIFICATION_IS_REMOVABLE_KEY,
      schedule == null || schedule.isRemovable()
    )
    flags = 0
    if (SDK_INT >= Build.VERSION_CODES.S) {
      flags = PendingIntent.FLAG_MUTABLE
    }
    val deleteIntent =
      PendingIntent.getBroadcast(context, notification.id, dissmissIntent, flags)
    mBuilder.setDeleteIntent(deleteIntent)
  }

  private fun buildIntent(notification: Notification, action: String?): Intent {
    val intent = if (activity != null) {
      Intent(context, activity.javaClass)
    } else {
      val packageName = context.packageName
      context.packageManager.getLaunchIntentForPackage(packageName)!!
    }
    intent.action = Intent.ACTION_MAIN
    intent.addCategory(Intent.CATEGORY_LAUNCHER)
    intent.flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
    intent.putExtra(NOTIFICATION_INTENT_KEY, notification.id)
    intent.putExtra(ACTION_INTENT_KEY, action)
    intent.putExtra(NOTIFICATION_OBJ_INTENT_KEY, notification.sourceJson)
    val schedule = notification.schedule
    intent.putExtra(NOTIFICATION_IS_REMOVABLE_KEY, schedule == null || schedule.isRemovable())
    return intent
  }

  /**
   * Build a notification trigger, such as triggering each N seconds, or
   * on a certain date "shape" (such as every first of the month)
   */
  // TODO support different AlarmManager.RTC modes depending on priority
  @SuppressLint("SimpleDateFormat")
  private fun triggerScheduledNotification(notification: android.app.Notification, request: Notification) {
    val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
    val schedule = request.schedule
    val notificationIntent = Intent(
      context,
      TimedNotificationPublisher::class.java
    )
    notificationIntent.putExtra(NOTIFICATION_INTENT_KEY, request.id)
    notificationIntent.putExtra(TimedNotificationPublisher.NOTIFICATION_KEY, notification)
    var flags = PendingIntent.FLAG_CANCEL_CURRENT
    if (SDK_INT >= Build.VERSION_CODES.S) {
      flags = flags or PendingIntent.FLAG_MUTABLE
    }
    var pendingIntent =
      PendingIntent.getBroadcast(context, request.id, notificationIntent, flags)

    when (schedule) {
      is NotificationSchedule.At -> {
        if (schedule.date.time < Date().time) {
          Logger.error(Logger.tags("Notification"), "Scheduled time must be *after* current time", null)
          return
        }
        if (schedule.repeating) {
          val interval: Long = schedule.date.time - Date().time
          alarmManager.setRepeating(AlarmManager.RTC, schedule.date.time, interval, pendingIntent)
        } else {
          setExactIfPossible(alarmManager, schedule, schedule.date.time, pendingIntent)
        }
      }
      is NotificationSchedule.Interval -> {
        val trigger = schedule.interval.nextTrigger(Date())
        notificationIntent.putExtra(TimedNotificationPublisher.CRON_KEY, schedule.interval.toMatchString())
        pendingIntent =
          PendingIntent.getBroadcast(context, request.id, notificationIntent, flags)
        setExactIfPossible(alarmManager, schedule, trigger, pendingIntent)
        val sdf = SimpleDateFormat("yyyy/MM/dd HH:mm:ss")
        Logger.debug(
          Logger.tags("Notification"),
          "notification " + request.id + " will next fire at " + sdf.format(Date(trigger))
        )
      }
      is NotificationSchedule.Every -> {
        val everyInterval = getIntervalTime(schedule.interval, schedule.count)
        val startTime: Long = Date().time + everyInterval
        alarmManager.setRepeating(AlarmManager.RTC, startTime, everyInterval, pendingIntent)
      }
      else -> {}
    }
  }

  @SuppressLint("ObsoleteSdkInt", "MissingPermission")
  private fun setExactIfPossible(
    alarmManager: AlarmManager,
    schedule: NotificationSchedule,
    trigger: Long,
    pendingIntent: PendingIntent
  ) {
    if (SDK_INT >= Build.VERSION_CODES.S && !alarmManager.canScheduleExactAlarms()) {
      if (SDK_INT >= Build.VERSION_CODES.M && schedule.allowWhileIdle()) {
        alarmManager.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, trigger, pendingIntent)
      } else {
        alarmManager[AlarmManager.RTC, trigger] = pendingIntent
      }
    } else {
      if (SDK_INT >= Build.VERSION_CODES.M && schedule.allowWhileIdle()) {
        alarmManager.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, trigger, pendingIntent)
      } else {
        alarmManager.setExact(AlarmManager.RTC, trigger, pendingIntent)
      }
    }
  }

  fun cancel(notifications: List<Int>) {
    for (id in notifications) {
      dismissVisibleNotification(id)
      cancelTimerForNotification(id)
      storage.deleteNotification(id.toString())
    }
  }

  private fun cancelTimerForNotification(notificationId: Int) {
    val intent = Intent(context, TimedNotificationPublisher::class.java)
    var flags = 0
    if (SDK_INT >= Build.VERSION_CODES.S) {
      flags = PendingIntent.FLAG_MUTABLE
    }
    val pi = PendingIntent.getBroadcast(context, notificationId, intent, flags)
    if (pi != null) {
      val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
      alarmManager.cancel(pi)
    }
  }

  private fun dismissVisibleNotification(notificationId: Int) {
    val notificationManager = NotificationManagerCompat.from(
      context
    )
    notificationManager.cancel(notificationId)
  }

  fun areNotificationsEnabled(): Boolean {
    val notificationManager = NotificationManagerCompat.from(context)
    return notificationManager.areNotificationsEnabled()
  }

  private fun getDefaultSoundUrl(context: Context): Uri? {
    val soundId = getDefaultSound(context)
    return if (soundId != AssetUtils.RESOURCE_ID_ZERO_VALUE) {
      Uri.parse(ContentResolver.SCHEME_ANDROID_RESOURCE + "://" + context.packageName + "/" + soundId)
    } else null
  }

  private fun getDefaultSound(context: Context): Int {
    if (defaultSoundID != AssetUtils.RESOURCE_ID_ZERO_VALUE) return defaultSoundID
    var resId: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
    val soundConfigResourceName = AssetUtils.getResourceBaseName(config?.sound)
    if (soundConfigResourceName != null) {
      resId = AssetUtils.getResourceID(context, soundConfigResourceName, "raw")
    }
    defaultSoundID = resId
    return resId
  }

  private fun getDefaultSmallIcon(context: Context): Int {
    if (defaultSmallIconID != AssetUtils.RESOURCE_ID_ZERO_VALUE) return defaultSmallIconID
    var resId: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
    val smallIconConfigResourceName = AssetUtils.getResourceBaseName(config?.icon)
    if (smallIconConfigResourceName != null) {
      resId = AssetUtils.getResourceID(context, smallIconConfigResourceName, "drawable")
    }
    if (resId == AssetUtils.RESOURCE_ID_ZERO_VALUE) {
      resId = android.R.drawable.ic_dialog_info
    }
    defaultSmallIconID = resId
    return resId
  }
}

class NotificationDismissReceiver : BroadcastReceiver() {
  override fun onReceive(context: Context, intent: Intent) {
    val intExtra =
      intent.getIntExtra(NOTIFICATION_INTENT_KEY, Int.MIN_VALUE)
    if (intExtra == Int.MIN_VALUE) {
      Logger.error(Logger.tags("Notification"), "Invalid notification dismiss operation", null)
      return
    }
    val isRemovable =
      intent.getBooleanExtra(NOTIFICATION_IS_REMOVABLE_KEY, true)
    if (isRemovable) {
      val notificationStorage = NotificationStorage(context, ObjectMapper())
      notificationStorage.deleteNotification(intExtra.toString())
    }
  }
}

class TimedNotificationPublisher : BroadcastReceiver() {
  /**
   * Restore and present notification
   */
  override fun onReceive(context: Context, intent: Intent) {
    val notificationManager =
      context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    val notification = if (SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      intent.getParcelableExtra(
        NOTIFICATION_KEY,
        android.app.Notification::class.java
      )
    } else {
      getParcelableExtraLegacy(intent, NOTIFICATION_KEY)
    }
    notification?.`when` = System.currentTimeMillis()
    val id = intent.getIntExtra(NOTIFICATION_INTENT_KEY, Int.MIN_VALUE)
    if (id == Int.MIN_VALUE) {
      Logger.error(Logger.tags("Notification"), "No valid id supplied", null)
    }
    val storage = NotificationStorage(context, ObjectMapper())

    val savedNotification = storage.getSavedNotification(id.toString())
    if (savedNotification != null) {
      NotificationPlugin.triggerNotification(savedNotification)
    }

    notificationManager.notify(id, notification)
    if (!rescheduleNotificationIfNeeded(context, intent, id)) {
      storage.deleteNotification(id.toString())
    }
  }

  @Suppress("DEPRECATION")
  private fun getParcelableExtraLegacy(intent: Intent, string: String): android.app.Notification? {
    return intent.getParcelableExtra(string)
  }

  @SuppressLint("MissingPermission", "SimpleDateFormat")
  private fun rescheduleNotificationIfNeeded(context: Context, intent: Intent, id: Int): Boolean {
    val dateString = intent.getStringExtra(CRON_KEY)
    if (dateString != null) {
      val date = DateMatch.fromMatchString(dateString)
      val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
      val trigger = date.nextTrigger(Date())
      val clone = intent.clone() as Intent
      var flags = PendingIntent.FLAG_CANCEL_CURRENT
      if (SDK_INT >= Build.VERSION_CODES.S) {
        flags = flags or PendingIntent.FLAG_MUTABLE
      }
      val pendingIntent = PendingIntent.getBroadcast(context, id, clone, flags)
      if (SDK_INT >= Build.VERSION_CODES.S && !alarmManager.canScheduleExactAlarms()) {
        alarmManager[AlarmManager.RTC, trigger] = pendingIntent
      } else {
        alarmManager.setExact(AlarmManager.RTC, trigger, pendingIntent)
      }
      val sdf = SimpleDateFormat("yyyy/MM/dd HH:mm:ss")
      Logger.debug(
        Logger.tags("Notification"),
        "notification " + id + " will next fire at " + sdf.format(Date(trigger))
      )
      return true
    }
    return false
  }

  companion object {
    var NOTIFICATION_KEY = "NotificationPublisher.notification"
    var CRON_KEY = "NotificationPublisher.cron"
  }
}

class LocalNotificationRestoreReceiver : BroadcastReceiver() {
  @SuppressLint("ObsoleteSdkInt")
  override fun onReceive(context: Context, intent: Intent) {
    if (SDK_INT >= Build.VERSION_CODES.N) {
      val um = context.getSystemService(
        UserManager::class.java
      )
      if (um == null || !um.isUserUnlocked) return
    }
    val storage = NotificationStorage(context, ObjectMapper())
    val ids = storage.getSavedNotificationIds()
    val notifications = mutableListOf<Notification>()
    val updatedNotifications = mutableListOf<Notification>()
    for (id in ids) {
      val notification = storage.getSavedNotification(id) ?: continue
      val schedule = notification.schedule
      if (schedule != null && schedule is NotificationSchedule.At) {
        val at: Date = schedule.date
        if (at.before(Date())) {
          // modify the scheduled date in order to show notifications that would have been delivered while device was off.
          val newDateTime = Date().time + 15 * 1000
          schedule.date = Date(newDateTime)
          updatedNotifications.add(notification)
        }
      }
      notifications.add(notification)
    }
    if (updatedNotifications.size > 0) {
      storage.appendNotifications(updatedNotifications)
    }

    var config: PluginConfig? = null
    try {
      config = PluginManager.loadConfig(context, "notification", PluginConfig::class.java)
    } catch (ex: Exception) {
      ex.printStackTrace()
    }
    val notificationManager = TauriNotificationManager(storage, null, context, config)
    notificationManager.schedule(notifications)
  }
}
