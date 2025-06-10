// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.ContentResolver
import android.content.Context
import android.graphics.Color
import android.media.AudioAttributes
import android.net.Uri
import android.os.Build
import app.tauri.Logger
import app.tauri.annotation.InvokeArg
import app.tauri.plugin.Invoke
import com.fasterxml.jackson.annotation.JsonValue

enum class Importance(@JsonValue val value: Int) {
  None(0),
  Min(1),
  Low(2),
  Default(3),
  High(4);
}

enum class Visibility(@JsonValue val value: Int) {
  Secret(-1),
  Private(0),
  Public(1);
}

@InvokeArg
class Channel {
  lateinit var id: String
  lateinit var name: String
  var description: String? = null
  var sound: String? = null
  var lights: Boolean? = null
  var lightsColor: String? = null
  var vibration: Boolean? = null
  var importance: Importance? = null
  var visibility: Visibility? = null
}

@InvokeArg
class DeleteChannelArgs {
  lateinit var id: String
}

class ChannelManager(private var context: Context) {
  private var notificationManager: NotificationManager? = null

  init {
    notificationManager =
      context.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager?
  }

  fun createChannel(invoke: Invoke) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val channel = invoke.parseArgs(Channel::class.java)
      createChannel(channel)
      invoke.resolve()
    } else {
      invoke.reject("channel not available")
    }
  }

  private fun createChannel(channel: Channel) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val notificationChannel = NotificationChannel(
        channel.id,
        channel.name,
        (channel.importance ?: Importance.Default).value
      )
      notificationChannel.description = channel.description
      notificationChannel.lockscreenVisibility = (channel.visibility ?: Visibility.Private).value
      notificationChannel.enableVibration(channel.vibration ?: false)
      notificationChannel.enableLights(channel.lights ?: false)
      val lightColor = channel.lightsColor ?: ""
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
      var sound = channel.sound ?: ""
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
      val args = invoke.parseArgs(DeleteChannelArgs::class.java)
      notificationManager?.deleteNotificationChannel(args.id)
      invoke.resolve()
    } else {
      invoke.reject("channel not available")
    }
  }

  fun listChannels(invoke: Invoke) {
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      val notificationChannels: List<NotificationChannel> =
        notificationManager?.notificationChannels ?: listOf()

      val channels = mutableListOf<Channel>()

      for (notificationChannel in notificationChannels) {
        val channel = Channel()
        channel.id = notificationChannel.id
        channel.name = notificationChannel.name.toString()
        channel.description = notificationChannel.description
        channel.sound = notificationChannel.sound.toString()
        channel.lights = notificationChannel.shouldShowLights()
        String.format(
          "#%06X",
          0xFFFFFF and notificationChannel.lightColor
        )
        channel.vibration = notificationChannel.shouldVibrate()
        channel.importance = Importance.values().firstOrNull { it.value == notificationChannel.importance }
        channel.visibility = Visibility.values().firstOrNull { it.value == notificationChannel.lockscreenVisibility }

        channels.add(channel)
      }

      invoke.resolveObject(channels)

    } else {
      invoke.reject("channel not available")
    }
  }
}