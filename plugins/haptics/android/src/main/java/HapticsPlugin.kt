// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.haptics

import android.Manifest
import android.app.Activity
import android.content.Context
import android.location.Location
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class HapticsOptions {
  var duration: Long = 300
}

@TauriPlugin
class HapticsPlugin(private val activity: Activity): Plugin(activity) {
    private val vibrator: Vibrator = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
        val vibManager = activity.applicationContext.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager
        vibManager.defaultVibrator
    } else {
        @Suppress("DEPRECATION")
        activity.applicationContext.getSystemService(Context.VIBRATOR_SERVICE) as Vibrator
    }
    
    @Command
    fun vibrate(invoke: Invoke) {
        val args = invoke.parseArgs(HapticsOptions::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createOneShot(args.duration, VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            vibrator.vibrate(argsduration)
        }
    }
}
