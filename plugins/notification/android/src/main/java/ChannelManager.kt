package app.tauri.notification

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.ContentResolver
import android.content.Context
import android.graphics.Color
import android.media.AudioAttributes
import android.net.Uri
import android.os.Build
import androidx.core.app.NotificationCompat
import app.tauri.Logger
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject

private const val CHANNEL_ID = "id"
private const val CHANNEL_NAME = "name"
private const val CHANNEL_DESCRIPTION = "description"
private const val CHANNEL_IMPORTANCE = "importance"
private const val CHANNEL_VISIBILITY = "visibility"
private const val CHANNEL_SOUND = "sound"
private const val CHANNEL_VIBRATE = "vibration"
private const val CHANNEL_USE_LIGHTS = "lights"
private const val CHANNEL_LIGHT_COLOR = "lightColor"

class ChannelManager(private var context: Context) {
  private var notificationManager: NotificationManager? = null

  init {
    notificationManager =
      context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager?
  }

  fun createChannel(invoke: Invoke) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val channel = JSObject()
      if (invoke.getString(CHANNEL_ID) != null) {
        channel.put(CHANNEL_ID, invoke.getString(CHANNEL_ID))
      } else {
        invoke.reject("Channel missing identifier")
        return
      }
      if (invoke.getString(CHANNEL_NAME) != null) {
        channel.put(CHANNEL_NAME, invoke.getString(CHANNEL_NAME))
      } else {
        invoke.reject("Channel missing name")
        return
      }
      channel.put(
        CHANNEL_IMPORTANCE,
        invoke.getInt(CHANNEL_IMPORTANCE, NotificationManager.IMPORTANCE_DEFAULT)
      )
      channel.put(CHANNEL_DESCRIPTION, invoke.getString(CHANNEL_DESCRIPTION, ""))
      channel.put(
        CHANNEL_VISIBILITY,
        invoke.getInt(CHANNEL_VISIBILITY, NotificationCompat.VISIBILITY_PUBLIC)
      )
      channel.put(CHANNEL_SOUND, invoke.getString(CHANNEL_SOUND))
      channel.put(CHANNEL_VIBRATE, invoke.getBoolean(CHANNEL_VIBRATE, false))
      channel.put(CHANNEL_USE_LIGHTS, invoke.getBoolean(CHANNEL_USE_LIGHTS, false))
      channel.put(CHANNEL_LIGHT_COLOR, invoke.getString(CHANNEL_LIGHT_COLOR))
      createChannel(channel)
      invoke.resolve()
    } else {
      invoke.reject("channel not available")
    }
  }

  private fun createChannel(channel: JSObject) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val notificationChannel = NotificationChannel(
        channel.getString(CHANNEL_ID),
        channel.getString(CHANNEL_NAME),
        channel.getInteger(CHANNEL_IMPORTANCE)!!
      )
      notificationChannel.description = channel.getString(CHANNEL_DESCRIPTION)
      notificationChannel.lockscreenVisibility = channel.getInteger(CHANNEL_VISIBILITY, android.app.Notification.VISIBILITY_PRIVATE)
      notificationChannel.enableVibration(channel.getBoolean(CHANNEL_VIBRATE, false))
      notificationChannel.enableLights(channel.getBoolean(CHANNEL_USE_LIGHTS, false))
      val lightColor = channel.getString(CHANNEL_LIGHT_COLOR)
      if (lightColor.isNotEmpty()) {
        try {
          notificationChannel.lightColor = Color.parseColor(lightColor)
        } catch (ex: IllegalArgumentException) {
          Logger.error(
            Logger.tags("NotificationChannel"),
            "Invalid color provided for light color.",
            null
          )
        }
      }
      var sound = channel.getString(CHANNEL_SOUND)
      if (sound.isNotEmpty()) {
        if (sound.contains(".")) {
          sound = sound.substring(0, sound.lastIndexOf('.'))
        }
        val audioAttributes = AudioAttributes.Builder()
          .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
          .setUsage(AudioAttributes.USAGE_NOTIFICATION)
          .build()
        val soundUri =
          Uri.parse(ContentResolver.SCHEME_ANDROID_RESOURCE + "://" + context.packageName + "/raw/" + sound)
        notificationChannel.setSound(soundUri, audioAttributes)
      }
      notificationManager?.createNotificationChannel(notificationChannel)
    }
  }

  fun deleteChannel(invoke: Invoke) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val channelId = invoke.getString("id")
      notificationManager?.deleteNotificationChannel(channelId)
      invoke.resolve()
    } else {
      invoke.reject("channel not available")
    }
  }

  fun listChannels(invoke: Invoke) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val notificationChannels: List<NotificationChannel> =
        notificationManager?.notificationChannels ?: listOf()
      val channels = JSArray()
      for (notificationChannel in notificationChannels) {
        val channel = JSObject()
        channel.put(CHANNEL_ID, notificationChannel.id)
        channel.put(CHANNEL_NAME, notificationChannel.name)
        channel.put(CHANNEL_DESCRIPTION, notificationChannel.description)
        channel.put(CHANNEL_IMPORTANCE, notificationChannel.importance)
        channel.put(CHANNEL_VISIBILITY, notificationChannel.lockscreenVisibility)
        channel.put(CHANNEL_SOUND, notificationChannel.sound)
        channel.put(CHANNEL_VIBRATE, notificationChannel.shouldVibrate())
        channel.put(CHANNEL_USE_LIGHTS, notificationChannel.shouldShowLights())
        channel.put(
          CHANNEL_LIGHT_COLOR, String.format(
            "#%06X",
            0xFFFFFF and notificationChannel.lightColor
          )
        )
        channels.put(channel)
      }
      val result = JSObject()
      result.put("channels", channels)
      invoke.resolve(result)
    } else {
      invoke.reject("channel not available")
    }
  }
}