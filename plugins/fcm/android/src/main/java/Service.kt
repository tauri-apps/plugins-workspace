package app.tauri.fcm

import android.annotation.SuppressLint
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import androidx.core.app.NotificationCompat
import com.google.firebase.messaging.FirebaseMessagingService
import com.google.firebase.messaging.RemoteMessage
import com.google.gson.Gson
import java.time.Instant
import kotlin.random.Random
import kotlin.time.Duration.Companion.milliseconds

class FCMService : FirebaseMessagingService() {
    @SuppressLint("LaunchActivityFromNotification")
    override fun onMessageReceived(remoteMessage: RemoteMessage) {
        val gson = Gson() // should we keep a reference ?
        val dataPayload = gson.toJson(remoteMessage.data)
        val intent = Intent(this, NotificationHandler::class.java).apply {
            putExtra("data", dataPayload)
            putExtra("sent_at", remoteMessage.sentTime)
        }

        val requestCode = Random.nextInt()
        val pendingIntent = PendingIntent.getBroadcast(
            this,
            requestCode,
            intent,
            PendingIntent.FLAG_CANCEL_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val channelId = "default_channel_id"
        val notificationBuilder = NotificationCompat.Builder(this, channelId).apply {
            setSmallIcon(getAppIconResourceId())  // TODO handle other icons ?
            setContentTitle(remoteMessage.notification?.title)
            setContentText(remoteMessage.notification?.body)
            setAutoCancel(true)
            setContentIntent(pendingIntent)
        }

        val notificationManager =
            getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

        // Since Android Oreo, notification channels are required
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                channelId, "Default Channel", NotificationManager.IMPORTANCE_DEFAULT
            )
            notificationManager.createNotificationChannel(channel)
        }

        notificationManager.notify(Random.nextInt(), notificationBuilder.build()
        )
    }

    private fun getAppIconResourceId(): Int {
        val packageManager = packageManager
        val packageName = applicationContext.packageName
        try {
            val appInfo = packageManager.getApplicationInfo(packageName, 0)
            return appInfo.icon
        } catch (e: PackageManager.NameNotFoundException) {
            e.printStackTrace()
        }
        return android.R.drawable.sym_def_app_icon // fallback icon
    }
}
