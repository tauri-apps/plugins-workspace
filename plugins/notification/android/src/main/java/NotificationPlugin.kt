// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.webkit.WebView
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.json.JSONArray
import org.json.JSONObject

const val LOCAL_NOTIFICATIONS = "permissionState"
private const val PREFS_NAME = "tauri_notification_plugin"
private const val PREF_KEY_PENDING_ACTION_EVENTS = "pending_action_events"
private const val PENDING_ACTION_EVENT_TTL_MS = 24 * 60 * 60 * 1000L

@InvokeArg
class PluginConfig {
  var icon: String? = null
  var sound: String? = null
  var iconColor: String? = null
}

@InvokeArg
class BatchArgs {
  lateinit var notifications: List<Notification>
}

@InvokeArg
class CancelArgs {
  lateinit var notifications: List<Int>
}

@InvokeArg
class NotificationAction {
  lateinit var id: String
  var title: String? = null
  var input: Boolean? = null
}

@InvokeArg
class ActionType {
  lateinit var id: String
  lateinit var actions: List<NotificationAction>
}

@InvokeArg
class RegisterActionTypesArgs {
  lateinit var types: List<ActionType>
}

@InvokeArg
class ActiveNotification {
  var id: Int = 0
  var tag: String? = null
}

@InvokeArg
class RemoveActiveArgs {
  var notifications: List<ActiveNotification> = listOf()
}

@TauriPlugin(
  permissions = [
    Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = "permissionState")
  ]
)
class NotificationPlugin(private val activity: Activity): Plugin(activity) {
  private var webView: WebView? = null
  private lateinit var manager: TauriNotificationManager
  private lateinit var notificationManager: NotificationManager
  private lateinit var notificationStorage: NotificationStorage
  private var channelManager = ChannelManager(activity)
  private data class PendingActionEvent(
    val key: String,
    val payload: JSObject,
    val timestampMs: Long
  )

  private val pendingActionEvents = mutableListOf<PendingActionEvent>()
  private val pendingActionEventKeys = mutableSetOf<String>()
  private var isActionListenerReady = false

  private fun nowMs(): Long = System.currentTimeMillis()

  private fun isEventExpired(timestampMs: Long): Boolean {
    return nowMs() - timestampMs > PENDING_ACTION_EVENT_TTL_MS
  }

  private fun buildActionEventKey(payload: JSObject): String {
    val notification = payload.optJSONObject("notification")
    val notificationId = notification?.opt("id") ?: payload.opt("id")
    val actionId = payload.optString("actionId")
    val inputValue = payload.optString("inputValue")

    if (notificationId != null && actionId.isNotEmpty()) {
      return "$notificationId|$actionId|$inputValue"
    }

    // Fallback for malformed payloads so we can still dedupe identical events.
    return "payload:${payload.toString()}"
  }

  private fun rebuildPendingActionEventKeysLocked() {
    pendingActionEventKeys.clear()
    for (event in pendingActionEvents) {
      pendingActionEventKeys.add(event.key)
    }
  }

  private fun persistPendingActionEventsLocked() {
    val iterator = pendingActionEvents.iterator()
    var droppedExpired = 0
    while (iterator.hasNext()) {
      val event = iterator.next()
      if (isEventExpired(event.timestampMs)) {
        iterator.remove()
        droppedExpired += 1
      }
    }
    if (droppedExpired > 0) {
      rebuildPendingActionEventKeysLocked()
      Logger.debug(
        Logger.tags("Notification"),
        "Dropped expired pending actionPerformed events=$droppedExpired"
      )
    }

    val events = JSONArray()
    for (event in pendingActionEvents) {
      try {
        val wrappedEvent = JSONObject()
        wrappedEvent.put("key", event.key)
        wrappedEvent.put("timestampMs", event.timestampMs)
        wrappedEvent.put("payload", JSONObject(event.payload.toString()))
        events.put(wrappedEvent)
      } catch (_: Throwable) {
        events.put(event.payload)
      }
    }
    activity
      .getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
      .edit()
      .putString(PREF_KEY_PENDING_ACTION_EVENTS, events.toString())
      .apply()
  }

  private fun restorePendingActionEventsLocked() {
    val prefs = activity.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
    val serializedEvents = prefs.getString(PREF_KEY_PENDING_ACTION_EVENTS, null) ?: return

    try {
      val events = JSONArray(serializedEvents)
      for (index in 0 until events.length()) {
        val event = events.optJSONObject(index) ?: continue
        val wrappedPayload = event.optJSONObject("payload")
        val payloadObject = wrappedPayload ?: event
        val payload = JSObject(payloadObject.toString())

        val timestampMs =
          if (wrappedPayload != null) event.optLong("timestampMs", nowMs()) else nowMs()
        if (isEventExpired(timestampMs)) {
          continue
        }

        val key = event.optString("key").ifEmpty { buildActionEventKey(payload) }
        if (pendingActionEventKeys.contains(key)) {
          Logger.debug(
            Logger.tags("Notification"),
            "Skipping duplicate restored actionPerformed event key=$key"
          )
          continue
        }

        pendingActionEvents.add(PendingActionEvent(key, payload, timestampMs))
        pendingActionEventKeys.add(key)
      }
      Logger.debug(
        Logger.tags("Notification"),
        "Restored pending actionPerformed events=${pendingActionEvents.size}"
      )
    } catch (error: Throwable) {
      Logger.error(
        Logger.tags("Notification"),
        "Failed to restore pending actionPerformed events",
        error
      )
      pendingActionEvents.clear()
      pendingActionEventKeys.clear()
      persistPendingActionEventsLocked()
    }
  }

  companion object {
    var instance: NotificationPlugin? = null

    fun triggerNotification(notification: Notification) {
      instance?.triggerObject("notification", notification)
    }
  }

  override fun load(webView: WebView) {
    instance = this

    super.load(webView)
    this.webView = webView
    synchronized(this) {
      pendingActionEvents.clear()
      pendingActionEventKeys.clear()
      isActionListenerReady = false
      restorePendingActionEventsLocked()
    }
    notificationStorage = NotificationStorage(activity, jsonMapper())
    
    val manager = TauriNotificationManager(
      notificationStorage,
      activity,
      activity,
      getConfig(PluginConfig::class.java)
    )
    manager.createNotificationChannel()
    
    this.manager = manager
    
    notificationManager = activity.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

    val intent = activity.intent
    intent?.let {
      onIntent(it)
    }
  }

  override fun onNewIntent(intent: Intent) {
    super.onNewIntent(intent)
    onIntent(intent)
  }

  fun onIntent(intent: Intent) {
    if (Intent.ACTION_MAIN != intent.action) {
      return
    }
    val dataJson = manager.handleNotificationActionPerformed(intent, notificationStorage)
    if (dataJson != null) {
      dispatchActionPerformed(dataJson)
    }
  }

  private fun dispatchActionPerformed(payload: JSObject) {
    synchronized(this) {
      if (!isActionListenerReady) {
        val key = buildActionEventKey(payload)
        // `load()` restores persisted pending events before processing the current activity intent.
        // Without this key check, the same action can be enqueued twice across reload boundaries.
        if (pendingActionEventKeys.contains(key)) {
          Logger.debug(
            Logger.tags("Notification"),
            "Skipping duplicate queued actionPerformed event key=$key"
          )
          return
        }
        pendingActionEvents.add(PendingActionEvent(key, payload, nowMs()))
        pendingActionEventKeys.add(key)
        persistPendingActionEventsLocked()
        Logger.debug(
          Logger.tags("Notification"),
          "Queued actionPerformed event; listener not ready (pending=${pendingActionEvents.size})"
        )
        return
      }
    }
    trigger("actionPerformed", payload)
  }

  @Command
  fun show(invoke: Invoke) {
    val notification = invoke.parseArgs(Notification::class.java)
    val id = manager.schedule(notification)

    invoke.resolveObject(id)
  }

  @Command
  fun batch(invoke: Invoke) {
    val args = invoke.parseArgs(BatchArgs::class.java)

    val ids = manager.schedule(args.notifications)
    notificationStorage.appendNotifications(args.notifications)

    invoke.resolveObject(ids)
  }

  @Command
  fun cancel(invoke: Invoke) {
    val args = invoke.parseArgs(CancelArgs::class.java)
    manager.cancel(args.notifications)
    invoke.resolve()
  }

  @Command
  fun removeActive(invoke: Invoke) {
    val args = invoke.parseArgs(RemoveActiveArgs::class.java)

    if (args.notifications.isEmpty()) {
      notificationManager.cancelAll()
      invoke.resolve()
    } else {
      for (notification in args.notifications) {
        if (notification.tag == null) {
          notificationManager.cancel(notification.id)
        } else {
          notificationManager.cancel(notification.tag, notification.id)
        }
      }
      invoke.resolve()
    }
  }

  @Command
  fun getPending(invoke: Invoke) {
    val notifications= notificationStorage.getSavedNotifications()
    val result = Notification.buildNotificationPendingList(notifications)
    invoke.resolveObject(result)
  }

  @Command
  fun registerActionTypes(invoke: Invoke) {
    val args = invoke.parseArgs(RegisterActionTypesArgs::class.java)
    notificationStorage.writeActionGroup(args.types)
    invoke.resolve()
  }

  @Command
  fun registerActionListenerReady(invoke: Invoke) {
    val pending = JSArray()
    synchronized(this) {
      isActionListenerReady = true
      for (event in pendingActionEvents) {
        pending.put(event.payload)
      }
      pendingActionEvents.clear()
      pendingActionEventKeys.clear()
      persistPendingActionEventsLocked()
    }
    invoke.resolveObject(pending)
  }

  @SuppressLint("ObsoleteSdkInt")
  @Command
  fun getActive(invoke: Invoke) {
    val notifications = JSArray()
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      val activeNotifications = notificationManager.activeNotifications
      for (activeNotification in activeNotifications) {
        val jsNotification = JSObject()
        jsNotification.put("id", activeNotification.id)
        jsNotification.put("tag", activeNotification.tag)
        val notification = activeNotification.notification
        if (notification != null) {
          jsNotification.put("title", notification.extras.getCharSequence(android.app.Notification.EXTRA_TITLE))
          jsNotification.put("body", notification.extras.getCharSequence(android.app.Notification.EXTRA_TEXT))
          jsNotification.put("group", notification.group)
          jsNotification.put(
            "groupSummary",
            0 != notification.flags and android.app.Notification.FLAG_GROUP_SUMMARY
          )
          val extras = JSObject()
          for (key in notification.extras.keySet()) {
            extras.put(key!!, notification.extras.getString(key))
          }
          jsNotification.put("data", extras)
        }
        notifications.put(jsNotification)
      }
    }
    
    invoke.resolveObject(notifications)
  }

  @Command
  fun createChannel(invoke: Invoke) {
    channelManager.createChannel(invoke)
  }

  @Command
  fun deleteChannel(invoke: Invoke) {
    channelManager.deleteChannel(invoke)
  }

  @Command
  fun listChannels(invoke: Invoke) {
    channelManager.listChannels(invoke)
  }

  @Command
  override fun checkPermissions(invoke: Invoke) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
      val permissionsResultJSON = JSObject()
      permissionsResultJSON.put("permissionState", getPermissionState())
      invoke.resolve(permissionsResultJSON)
    } else {
      super.checkPermissions(invoke)
    }
  }

  @Command
  override fun requestPermissions(invoke: Invoke) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
      permissionState(invoke)
    } else {
      if (getPermissionState(LOCAL_NOTIFICATIONS) !== PermissionState.GRANTED) {
        requestPermissionForAlias(LOCAL_NOTIFICATIONS, invoke, "permissionsCallback")
      }
    }
  }

  @Command
  fun permissionState(invoke: Invoke) {
    val permissionsResultJSON = JSObject()
    permissionsResultJSON.put("permissionState", getPermissionState())
    invoke.resolve(permissionsResultJSON)
  }

  @PermissionCallback
  private fun permissionsCallback(invoke: Invoke) {
    val permissionsResultJSON = JSObject()
    permissionsResultJSON.put("permissionState", getPermissionState())
    invoke.resolve(permissionsResultJSON)
  }

  private fun getPermissionState(): String {
    return if (manager.areNotificationsEnabled()) {
      "granted"
    } else {
      "denied"
    }
  }
}
