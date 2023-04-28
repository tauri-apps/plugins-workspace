package app.tauri.notification

import android.Manifest
import android.annotation.SuppressLint
import android.app.Activity
import android.app.NotificationManager
import android.content.Context
import android.os.Build
import android.webkit.WebView
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.json.JSONException
import org.json.JSONObject

const val LOCAL_NOTIFICATIONS = "permissionState"

@TauriPlugin(
  permissions = [
    Permission(strings = [Manifest.permission.POST_NOTIFICATIONS], alias = "permissionState")
  ]
)
class NotificationPlugin(private val activity: Activity): Plugin(activity) {
  private var webView: WebView? = null
  private var manager: TauriNotificationManager? = null
  var notificationManager: NotificationManager? = null
  private var notificationStorage: NotificationStorage? = null
  private var channelManager = ChannelManager(activity)
  
  override fun load(webView: WebView) {
    super.load(webView)
    this.webView = webView
    notificationStorage = NotificationStorage(activity)
    
    val manager = TauriNotificationManager(
      notificationStorage!!,
      activity,
      activity,
      getConfig()
    )
    manager.createNotificationChannel()
    
    this.manager = manager
    
    notificationManager = activity.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager?
  }

  /*private fun handleOnNewIntent(data: Intent) {
    super.handleOnNewIntent(data)
    if (Intent.ACTION_MAIN != data.action) {
      return
    }
    val dataJson = manager.handleNotificationActionPerformed(data, notificationStorage)
    if (dataJson != null) {
      // notifyListeners("localNotificationActionPerformed", dataJson, true)
    }
  }*/

  @Command
  fun batch(invoke: Invoke) {
    val localNotifications = Notification.buildNotificationList(invoke)
      ?: return
    val ids = manager!!.schedule(invoke, localNotifications)
    if (ids != null) {
      notificationStorage?.appendNotifications(localNotifications)
      val result = JSObject()
      val jsArray = JSArray()
      for (i in 0 until ids.length()) {
        try {
          val notification = JSObject().put("id", ids.getInt(i))
          jsArray.put(notification)
        } catch (_: Exception) {
        }
      }
      result.put("notifications", jsArray)
      invoke.resolve(result)
    }
  }

  @Command
  fun cancel(invoke: Invoke) {
    val notifications = invoke.getArray("notifications")
    if (notifications == null) {
      manager?.cancel(invoke)
    } else {
      try {
        for (o in notifications.toList<Any>()) {
          if (o is JSONObject) {
            val notification = JSObject.fromJSONObject((o))
            val tag = notification.getString("tag")
            val id = notification.getInteger("id")
            if (tag.isEmpty()) {
              notificationManager!!.cancel(id!!)
            } else {
              notificationManager!!.cancel(tag, id!!)
            }
          } else {
            invoke.reject("Unexpected notification type")
          }
        }
      } catch (e: JSONException) {
        invoke.reject(e.message)
      }
      invoke.resolve()
    }
  }

  @Command
  fun getPending(invoke: Invoke) {
    val notifications= notificationStorage!!.getSavedNotifications()
    val result = Notification.buildNotificationPendingList(notifications)
    invoke.resolve(result)
  }

  @Command
  fun registerActionTypes(invoke: Invoke) {
    val types = invoke.getArray("types", JSArray())
    val typesArray = NotificationAction.buildTypes(types)
    notificationStorage?.writeActionGroup(typesArray)
    invoke.resolve()
  }

  @SuppressLint("ObsoleteSdkInt")
  @Command
  fun getDeliveredNotifications(invoke: Invoke) {
    val notifications = JSArray()
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
      val activeNotifications = notificationManager!!.activeNotifications
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
    val result = JSObject()
    result.put("notifications", notifications)
    invoke.resolve(result)
  }

  @Command
  fun cancelAll(invoke: Invoke) {
    notificationManager!!.cancelAll()
    invoke.resolve()
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
    permissionsResultJSON.put("display", getPermissionState())
    invoke.resolve(permissionsResultJSON)
  }

  private fun getPermissionState(): String {
    return if (manager!!.areNotificationsEnabled()) {
      "granted"
    } else {
      "denied"
    }
  }
}
