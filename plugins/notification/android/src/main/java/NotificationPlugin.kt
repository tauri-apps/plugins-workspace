package app.tauri.notification

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@TauriPlugin
class NotificationPlugin(private val activity: Activity): Plugin(activity) {
    @Command
    fun requestPermission(invoke: Invoke) {
        val ret = JSObject()
        ret.put("permissionState", "granted")
        invoke.resolve(ret)
    }

    @Command
    fun permissionState(invoke: Invoke) {
        val ret = JSObject()
        ret.put("permissionState", "granted")
        invoke.resolve(ret)
    }

    @Command
    fun notify(invoke: Invoke) {
        // TODO
        invoke.resolve()
    }
}
