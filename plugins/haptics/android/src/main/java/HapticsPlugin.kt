// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.haptics

import android.app.Activity
import android.content.Context
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.haptics.patterns.ImpactPatternHeavy
import app.tauri.haptics.patterns.ImpactPatternLight
import app.tauri.haptics.patterns.ImpactPatternMedium
import app.tauri.haptics.patterns.ImpactPatternRigid
import app.tauri.haptics.patterns.ImpactPatternSoft
import app.tauri.haptics.patterns.NotificationPatternError
import app.tauri.haptics.patterns.NotificationPatternSuccess
import app.tauri.haptics.patterns.NotificationPatternWarning
import app.tauri.haptics.patterns.Pattern
import app.tauri.haptics.patterns.SelectionPattern
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import com.fasterxml.jackson.annotation.JsonProperty

@InvokeArg
class HapticsOptions {
  var duration: Long = 300
}

@InvokeArg
class NotificationFeedbackArgs {
    val type: NotificationFeedbackType = NotificationFeedbackType.Success
}

@InvokeArg
enum class NotificationFeedbackType {
    @JsonProperty("success")
    Success,
    @JsonProperty("warning")
    Warning,
    @JsonProperty("error")
    Error;

    fun into(): Pattern {
        return when(this) {
            Success -> NotificationPatternSuccess
            Warning -> NotificationPatternWarning
            Error -> NotificationPatternError
        }
    }
}

@InvokeArg
class ImpactFeedbackArgs {
    val style: ImpactFeedbackStyle = ImpactFeedbackStyle.Medium
}

@InvokeArg
enum class ImpactFeedbackStyle {
    @JsonProperty("light")
    Light,
    @JsonProperty("medium")
    Medium,
    @JsonProperty("heavy")
    Heavy,
    @JsonProperty("soft")
    Soft,
    @JsonProperty("rigid")
    Rigid;

    fun into(): Pattern {
        return when(this) {
            Light -> ImpactPatternLight
            Medium -> ImpactPatternMedium
            Heavy -> ImpactPatternHeavy
            Soft -> ImpactPatternSoft
            Rigid -> ImpactPatternRigid
        }
    }
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

    //
    // TAURI COMMANDS
    //

    @Command
    fun vibrate(invoke: Invoke) {
        val args = invoke.parseArgs(HapticsOptions::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createOneShot(args.duration, VibrationEffect.DEFAULT_AMPLITUDE))
        } else {
            vibrator.vibrate(args.duration)
        }
        invoke.resolve()
    }

    @Command
    fun impactFeedback(invoke: Invoke) {
        val args = invoke.parseArgs(ImpactFeedbackArgs::class.java)
        vibratePattern(args.style.into())
        invoke.resolve()
    }

    @Command
    fun notificationFeedback(invoke: Invoke) {
        val args = invoke.parseArgs(NotificationFeedbackArgs::class.java)
        vibratePattern(args.type.into())
        invoke.resolve()
    }

    // TODO: Consider breaking this up into Start,Change,End like capacitor
    @Command
    fun selectionFeedback(invoke: Invoke) {
        vibratePattern(SelectionPattern)
        invoke.resolve()
    }

    // INTERNAL FUNCTIONS

    private fun vibratePattern(pattern: Pattern) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibrator.vibrate(VibrationEffect.createWaveform(pattern.timings, pattern.amplitudes, -1))
        } else {
            vibrator.vibrate(pattern.oldSDKPattern, -1)
        }
    }
}
