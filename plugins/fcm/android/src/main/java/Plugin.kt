package app.tauri.fcm

import android.app.Activity
import android.content.Intent
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

import com.google.firebase.FirebaseApp
import com.google.firebase.FirebaseOptions
import com.google.firebase.installations.FirebaseInstallations
import com.google.firebase.messaging.FirebaseMessaging

@InvokeArg
class PluginConfig {
  lateinit var androidAppId: String
  lateinit var apiKey: String
  lateinit var projectId: String
}

@InvokeArg
class SubscribeToTopicArgs {
    lateinit var topic: String
}

@TauriPlugin
class FCMPlugin(private val activity: Activity) : Plugin(activity) {
    private var latestData = JSObject()

    override fun load(webView: WebView) {
        val config = this.getConfig(PluginConfig::class.java)
        val options = FirebaseOptions.Builder().setApiKey(config.apiKey)
            .setProjectId(config.projectId)
            .setApplicationId(config.androidAppId).build()

        FirebaseApp.initializeApp(activity, options)

        activity.intent?.let {
            handleIntent(it)
        }
    }

    override fun onNewIntent(newIntent: Intent) {
        handleIntent(newIntent)
    }

    private fun handleIntent(newIntent: Intent) {
        newIntent.extras?.let {
            val data = it.getString("data")
            val sentAt = it.getLong("sent_at")
            val openedAt = it.getLong("opened_at")
            if (data != null) {
                val result = JSObject().apply {
                    put("data", JSObject(data))
                    put("sentAt", sentAt)
                    put("openedAt", openedAt)
                }

                this.latestData = result
                trigger("pushNotificationOpened", result)
            }
        }
    }

    @Command
    fun getLatestNotificationData(invoke: Invoke) {
        invoke.resolve(this.latestData)
    }

    @Command
    fun getToken(invoke: Invoke) {
        FirebaseInstallations.getInstance().getToken(true).addOnSuccessListener { result ->
            invoke.resolve(JSObject().put("token", result.token))
        }.addOnFailureListener { e ->
            invoke.reject("Cannot get token", e)
        }
    }

    @Command
    fun subscribeToTopic(invoke: Invoke) {
        val args = invoke.parseArgs(SubscribeToTopicArgs::class.java)

        FirebaseMessaging.getInstance().subscribeToTopic(args.topic).addOnSuccessListener {
            invoke.resolve(JSObject())
        }.addOnFailureListener { e ->
            invoke.reject("Cannot subscribe to topic", e)
        }
    }

}
