// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.os

import android.annotation.SuppressLint
import android.app.Activity
import android.app.ActivityManager
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.BatteryManager
import android.os.Build
import android.provider.Settings
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@TauriPlugin
class OsPlugin(private val activity: Activity) : Plugin(activity) {
  @Command
  fun getDeviceInfo(invoke: Invoke) {
    val ret = JSObject()
    ret.put("model", Build.MODEL)
    ret.put("manufacturer", Build.MANUFACTURER)
    ret.put("isVirtual", isEmulator())
    totalMemory()?.let { ret.put("totalMemory", it) }
    ret.put("webviewVersion", webviewVersion())
    invoke.resolve(ret)
  }

  @Command
  fun getDeviceName(invoke: Invoke) {
    val ret = JSObject()
    // Settings.Global.DEVICE_NAME was added in API 25. Fields left unset reach Rust as `None`.
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N_MR1) {
      ret.put("name", Settings.Global.getString(activity.contentResolver, Settings.Global.DEVICE_NAME))
    }
    invoke.resolve(ret)
  }

  @Command
  fun getBatteryInfo(invoke: Invoke) {
    val ret = JSObject()
    // ACTION_BATTERY_CHANGED is sticky: registering without a receiver returns the last broadcast
    val intent = activity.registerReceiver(null, IntentFilter(Intent.ACTION_BATTERY_CHANGED))
    if (intent != null) {
      val level = intent.getIntExtra(BatteryManager.EXTRA_LEVEL, -1)
      val scale = intent.getIntExtra(BatteryManager.EXTRA_SCALE, -1)
      if (level >= 0 && scale > 0) {
        ret.put("level", level.toDouble() / scale)
      }
      val status = intent.getIntExtra(BatteryManager.EXTRA_STATUS, -1)
      if (status != -1 && status != BatteryManager.BATTERY_STATUS_UNKNOWN) {
        ret.put(
          "isCharging",
          status == BatteryManager.BATTERY_STATUS_CHARGING || status == BatteryManager.BATTERY_STATUS_FULL
        )
      }
    }
    invoke.resolve(ret)
  }

  private fun totalMemory(): Long? {
    val activityManager = activity.getSystemService(Context.ACTIVITY_SERVICE) as? ActivityManager ?: return null
    val memoryInfo = ActivityManager.MemoryInfo()
    activityManager.getMemoryInfo(memoryInfo)
    return memoryInfo.totalMem
  }

  // same lookup as wry's WryActivity, which backs tauri::webview_version()
  @SuppressLint("WebViewApiAvailability")
  private fun webviewVersion(): String? {
    // WebView.getCurrentWebViewPackage() was added in API 26
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      return try {
        WebView.getCurrentWebViewPackage()?.versionName
      } catch (ex: Exception) {
        Logger.warn("failed to get the WebView package: $ex")
        null
      }
    }
    return null
  }

  // A heuristic: it recognizes the Android SDK emulator (goldfish/ranchu, generic or sdk builds)
  // but not necessarily every third-party emulator.
  private fun isEmulator(): Boolean {
    return Build.FINGERPRINT.startsWith("generic") ||
      Build.FINGERPRINT.contains("emulator") ||
      Build.HARDWARE == "goldfish" ||
      Build.HARDWARE == "ranchu" ||
      Build.PRODUCT.contains("sdk") ||
      Build.PRODUCT.contains("emulator") ||
      Build.MODEL.contains("Emulator") ||
      Build.MODEL.contains("Android SDK built for")
  }
}
