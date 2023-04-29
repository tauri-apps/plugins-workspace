package app.tauri.notification

import android.Manifest
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
import android.content.pm.PackageManager
import android.graphics.Color
import android.media.AudioAttributes
import android.net.Uri
import android.os.Build
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.app.RemoteInput
import app.tauri.Logger
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import org.json.JSONArray
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
  private val activity: Activity,
  private val context: Context,
  private val config: JSObject
) {
  private var defaultSoundID: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
  private var defaultSmallIconID: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE

  /**
   * Method extecuted when notification is launched by user from the notification bar.
   */
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
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
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
  // TODO control visibility by flag Notification.VISIBILITY_PRIVATE
  // TODO Group notifications (setGroup, setGroupSummary, setNumber)
  // TODO use NotificationCompat.MessagingStyle for latest API
  // TODO expandable notification NotificationCompat.MessagingStyle
  // TODO media style notification support NotificationCompat.MediaStyle
  // TODO custom small/large icons
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
    mBuilder.setVisibility(NotificationCompat.VISIBILITY_PRIVATE)
    mBuilder.setOnlyAlertOnce(true)
    mBuilder.setSmallIcon(notification.getSmallIcon(context, getDefaultSmallIcon(context)))
    mBuilder.setLargeIcon(notification.getLargeIcon(context))
    val iconColor = notification.getIconColor(config.getString("iconColor"))
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
    if (notification.isScheduled) {
      triggerScheduledNotification(buildNotification, notification)
    } else {
      try {
        // TODO notify
        // val notificationJson = JSObject(notification.source ?: "")
      } catch (_: JSONException) {
      }
      if (ActivityCompat.checkSelfPermission(
          activity,
          Manifest.permission.POST_NOTIFICATIONS
        ) != PackageManager.PERMISSION_GRANTED
      ) {
        return
      }
      notificationManager.notify(notification.id ?: 0, buildNotification)
    }
  }

  // Create intents for open/dissmis actions
  private fun createActionIntents(
    notification: Notification,
    mBuilder: NotificationCompat.Builder
  ) {
    // Open intent
    val intent = buildIntent(notification, DEFAULT_PRESS_ACTION)
    var flags = PendingIntent.FLAG_CANCEL_CURRENT
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
      flags = flags or PendingIntent.FLAG_MUTABLE
    }
    val pendingIntent = PendingIntent.getActivity(context, notification.id ?: 0, intent, flags)
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
          (notification.id ?: 0) + notificationAction.id.hashCode(),
          actionIntent,
          flags
        )
        val actionBuilder: NotificationCompat.Action.Builder = NotificationCompat.Action.Builder(
          R.drawable.ic_transparent,
          notificationAction.title,
          actionPendingIntent
        )
        if (notificationAction.input) {
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
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
      flags = PendingIntent.FLAG_MUTABLE
    }
    val deleteIntent =
      PendingIntent.getBroadcast(context, notification.id ?: 0, dissmissIntent, flags)
    mBuilder.setDeleteIntent(deleteIntent)
  }

  private fun buildIntent(notification: Notification, action: String?): Intent {
    val intent = Intent(context, activity.javaClass)
    intent.action = Intent.ACTION_MAIN
    intent.addCategory(Intent.CATEGORY_LAUNCHER)
    intent.flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
    intent.putExtra(NOTIFICATION_INTENT_KEY, notification.id)
    intent.putExtra(ACTION_INTENT_KEY, action)
    intent.putExtra(NOTIFICATION_OBJ_INTENT_KEY, notification.source)
    val schedule = notification.schedule
    intent.putExtra(NOTIFICATION_IS_REMOVABLE_KEY, schedule == null || schedule.isRemovable())
    return intent
  }

  /**
   * Build a notification trigger, such as triggering each N seconds, or
   * on a certain date "shape" (such as every first of the month)
   */
  // TODO support different AlarmManager.RTC modes depending on priority
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
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
      flags = flags or PendingIntent.FLAG_MUTABLE
    }
    var pendingIntent =
      PendingIntent.getBroadcast(context, request.id ?: 0, notificationIntent, flags)

    when (val scheduleKind = schedule?.kind) {
      is ScheduleKind.At -> {
        val at = scheduleKind.date
        if (at.time < Date().time) {
          Logger.error(Logger.tags("Notification"), "Scheduled time must be *after* current time", null)
          return
        }
        if (scheduleKind.repeating) {
          val interval: Long = at.time - Date().time
          alarmManager.setRepeating(AlarmManager.RTC, at.time, interval, pendingIntent)
        } else {
          setExactIfPossible(alarmManager, schedule, at.time, pendingIntent)
        }
      }
      is ScheduleKind.Interval -> {
        val trigger = scheduleKind.interval.nextTrigger(Date())
        notificationIntent.putExtra(TimedNotificationPublisher.CRON_KEY, scheduleKind.interval.toMatchString())
        pendingIntent =
          PendingIntent.getBroadcast(context, request.id ?: 0, notificationIntent, flags)
        setExactIfPossible(alarmManager, schedule, trigger, pendingIntent)
        val sdf = SimpleDateFormat("yyyy/MM/dd HH:mm:ss")
        Logger.debug(
          Logger.tags("Notification"),
          "notification " + request.id + " will next fire at " + sdf.format(Date(trigger))
        )
      }
      is ScheduleKind.Every -> {
        val everyInterval = getIntervalTime(scheduleKind.interval, scheduleKind.count)
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
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && !alarmManager.canScheduleExactAlarms()) {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M && schedule.whileIdle == true) {
        alarmManager.setAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, trigger, pendingIntent)
      } else {
        alarmManager[AlarmManager.RTC, trigger] = pendingIntent
      }
    } else {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M && schedule.whileIdle == true) {
        alarmManager.setExactAndAllowWhileIdle(AlarmManager.RTC_WAKEUP, trigger, pendingIntent)
      } else {
        alarmManager.setExact(AlarmManager.RTC, trigger, pendingIntent)
      }
    }
  }

  fun cancel(invoke: Invoke) {
    val notificationsToCancel = Notification.getNotificationPendingList(invoke)
    if (notificationsToCancel != null) {
      for (id in notificationsToCancel) {
        dismissVisibleNotification(id)
        cancelTimerForNotification(id)
        storage.deleteNotification(id.toString())
      }
    }
    invoke.resolve()
  }

  private fun cancelTimerForNotification(notificationId: Int) {
    val intent = Intent(context, TimedNotificationPublisher::class.java)
    var flags = 0
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
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
    val soundConfigResourceName = AssetUtils.getResourceBaseName(config.getString("sound"))
    if (soundConfigResourceName != null) {
      resId = AssetUtils.getResourceID(context, soundConfigResourceName, "raw")
    }
    defaultSoundID = resId
    return resId
  }

  private fun getDefaultSmallIcon(context: Context): Int {
    if (defaultSmallIconID != AssetUtils.RESOURCE_ID_ZERO_VALUE) return defaultSmallIconID
    var resId: Int = AssetUtils.RESOURCE_ID_ZERO_VALUE
    val smallIconConfigResourceName = AssetUtils.getResourceBaseName(config.getString("icon"))
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
      val notificationStorage = NotificationStorage(context)
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
    val notification = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
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
    val storage = NotificationStorage(context)
    // TODO notify
    // val notificationJson = storage.getSavedNotificationAsJSObject(id.toString())
    notificationManager.notify(id, notification)
    if (!rescheduleNotificationIfNeeded(context, intent, id)) {
      storage.deleteNotification(id.toString())
    }
  }

  @Suppress("DEPRECATION")
  private fun getParcelableExtraLegacy(intent: Intent, string: String): android.app.Notification? {
    return intent.getParcelableExtra(string)
  }

  @SuppressLint("MissingPermission")
  private fun rescheduleNotificationIfNeeded(context: Context, intent: Intent, id: Int): Boolean {
    val dateString = intent.getStringExtra(CRON_KEY)
    if (dateString != null) {
      val date = DateMatch.fromMatchString(dateString)
      val alarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
      val trigger = date.nextTrigger(Date())
      val clone = intent.clone() as Intent
      var flags = PendingIntent.FLAG_CANCEL_CURRENT
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        flags = flags or PendingIntent.FLAG_MUTABLE
      }
      val pendingIntent = PendingIntent.getBroadcast(context, id, clone, flags)
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && !alarmManager.canScheduleExactAlarms()) {
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