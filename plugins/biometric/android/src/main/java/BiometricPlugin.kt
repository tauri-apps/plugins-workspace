// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.biometric

import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.util.EnumMap
import java.util.HashMap
import kotlin.math.max

enum class BiometryResultType {
    SUCCESS, FAILURE, ERROR
}

private const val MAX_ATTEMPTS = "maxAttemps"
private const val BIOMETRIC_FAILURE = "authenticationFailed"
private const val INVALID_CONTEXT_ERROR = "invalidContext"

@InvokeArg
class AuthOptions {
    lateinit var reason: String
    var allowDeviceCredential: Boolean = false
    var title: String? = null
    var subtitle: String? = null
    var cancelTitle: String? = null
    var confirmationRequired: Boolean? = null
    var maxAttemps: Int = 3
}

@TauriPlugin
class BiometricPlugin(private val activity: Activity): Plugin(activity) {
    private var biometryTypes: ArrayList<BiometryType> = arrayListOf()

    companion object {
        var RESULT_EXTRA_PREFIX = ""
        const val TITLE = "title"
        const val SUBTITLE = "subtitle"
        const val REASON = "reason"
        const val CANCEL_TITLE = "cancelTitle"
        const val RESULT_TYPE = "type"
        const val RESULT_ERROR_CODE = "errorCode"
        const val RESULT_ERROR_MESSAGE = "errorMessage"
        const val DEVICE_CREDENTIAL = "allowDeviceCredential"
        const val CONFIRMATION_REQUIRED = "confirmationRequired"

        // Maps biometry error numbers to string error codes
        private var biometryErrorCodeMap: MutableMap<Int, String> = HashMap()
        private var biometryNameMap: MutableMap<BiometryType, String> = EnumMap(BiometryType::class.java)

       init {
           biometryErrorCodeMap[BiometricManager.BIOMETRIC_SUCCESS] = ""
           biometryErrorCodeMap[BiometricManager.BIOMETRIC_SUCCESS] = ""
           biometryErrorCodeMap[BiometricPrompt.ERROR_CANCELED] = "systemCancel"
           biometryErrorCodeMap[BiometricPrompt.ERROR_HW_NOT_PRESENT] = "biometryNotAvailable"
           biometryErrorCodeMap[BiometricPrompt.ERROR_HW_UNAVAILABLE] = "biometryNotAvailable"
           biometryErrorCodeMap[BiometricPrompt.ERROR_LOCKOUT] = "biometryLockout"
           biometryErrorCodeMap[BiometricPrompt.ERROR_LOCKOUT_PERMANENT] = "biometryLockout"
           biometryErrorCodeMap[BiometricPrompt.ERROR_NEGATIVE_BUTTON] = "userCancel"
           biometryErrorCodeMap[BiometricPrompt.ERROR_NO_BIOMETRICS] = "biometryNotEnrolled"
           biometryErrorCodeMap[BiometricPrompt.ERROR_NO_DEVICE_CREDENTIAL] = "noDeviceCredential"
           biometryErrorCodeMap[BiometricPrompt.ERROR_NO_SPACE] = "systemCancel"
           biometryErrorCodeMap[BiometricPrompt.ERROR_TIMEOUT] = "systemCancel"
           biometryErrorCodeMap[BiometricPrompt.ERROR_UNABLE_TO_PROCESS] = "systemCancel"
           biometryErrorCodeMap[BiometricPrompt.ERROR_USER_CANCELED] = "userCancel"
           biometryErrorCodeMap[BiometricPrompt.ERROR_VENDOR] = "systemCancel"

           biometryNameMap[BiometryType.NONE] = "No Authentication"
           biometryNameMap[BiometryType.FINGERPRINT] = "Fingerprint Authentication"
           biometryNameMap[BiometryType.FACE] = "Face Authentication"
           biometryNameMap[BiometryType.IRIS] = "Iris Authentication"
       }
    }

    override fun load(webView: WebView) {
        super.load(webView)

        biometryTypes = ArrayList()
        val manager = activity.packageManager
        if (manager.hasSystemFeature(PackageManager.FEATURE_FINGERPRINT)) {
            biometryTypes.add(BiometryType.FINGERPRINT)
        }
        if (manager.hasSystemFeature(PackageManager.FEATURE_FACE)) {
            biometryTypes.add(BiometryType.FACE)
        }
        if (manager.hasSystemFeature(PackageManager.FEATURE_IRIS)) {
            biometryTypes.add(BiometryType.IRIS)
        }
        if (biometryTypes.size == 0) {
            biometryTypes.add(BiometryType.NONE)
        }
    }

    /**
     * Check the device's availability and type of biometric authentication.
     */
    @Command
    fun status(invoke: Invoke) {
        val manager = BiometricManager.from(activity)
        val biometryResult = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            manager.canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_WEAK)
        } else {
            @Suppress("DEPRECATION")
            manager.canAuthenticate()
        }
        val ret = JSObject()

        val available = biometryResult == BiometricManager.BIOMETRIC_SUCCESS
        ret.put(
            "isAvailable",
            available
        )

        ret.put("biometryType", biometryTypes[0].type)

        if (!available) {
            var reason = ""
            when (biometryResult) {
                BiometricManager.BIOMETRIC_ERROR_HW_UNAVAILABLE -> reason =
                    "Biometry unavailable."
                BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED -> reason =
                    "Biometrics not enrolled."
                BiometricManager.BIOMETRIC_ERROR_NO_HARDWARE -> reason =
                    "No biometric on this device."
                BiometricManager.BIOMETRIC_ERROR_SECURITY_UPDATE_REQUIRED -> reason =
                    "A security update is required."
                BiometricManager.BIOMETRIC_ERROR_UNSUPPORTED -> reason =
                    "Unsupported biometry."
                BiometricManager.BIOMETRIC_STATUS_UNKNOWN -> reason =
                    "Unknown biometry state."
            }

            var errorCode = biometryErrorCodeMap[biometryResult]
            if (errorCode == null) {
                errorCode = "biometryNotAvailable"
            }
            ret.put("error", reason)
            ret.put("errorCode", errorCode)
        }

        invoke.resolve(ret)
    }

    /**
     * Prompt the user for biometric authentication.
     */
    @Command
    fun authenticate(invoke: Invoke) {
        // The result of an intent is supposed to have the package name as a prefix
        RESULT_EXTRA_PREFIX = activity.packageName + "."
        val intent = Intent(
            activity,
            BiometricActivity::class.java
        )
        
        val args = invoke.parseArgs(AuthOptions::class.java)

        // Pass the options to the activity
        intent.putExtra(
            TITLE,
            args.title ?: (biometryNameMap[biometryTypes[0]] ?: "")
        )
        intent.putExtra(SUBTITLE, args.subtitle)
        intent.putExtra(REASON, args.reason)
        intent.putExtra(CANCEL_TITLE, args.cancelTitle)
        intent.putExtra(DEVICE_CREDENTIAL, args.allowDeviceCredential)
        args.confirmationRequired?.let {
            intent.putExtra(CONFIRMATION_REQUIRED, it)
        }

        val maxAttemptsConfig = args.maxAttemps
        val maxAttempts = max(maxAttemptsConfig, 1)
        intent.putExtra(MAX_ATTEMPTS, maxAttempts)
        startActivityForResult(invoke, intent, "authenticateResult")
    }

    @ActivityCallback
    private fun authenticateResult(invoke: Invoke, result: ActivityResult) {
        val resultCode = result.resultCode

        // If the system canceled the activity, we might get RESULT_CANCELED in resultCode.
        // In that case return that immediately, because there won't be any data.
        if (resultCode == Activity.RESULT_CANCELED) {
            invoke.reject(
                "The system canceled authentication",
                biometryErrorCodeMap[BiometricPrompt.ERROR_CANCELED]
            )
            return
        }

        // Convert the string result type to an enum
        val data = result.data
        val resultTypeName = data?.getStringExtra(
            RESULT_EXTRA_PREFIX + RESULT_TYPE
        )
        if (resultTypeName == null) {
            invoke.reject(
                "Missing data in the result of the activity",
                INVALID_CONTEXT_ERROR
            )
            return
        }
        val resultType = try {
            BiometryResultType.valueOf(resultTypeName)
        } catch (e: IllegalArgumentException) {
            invoke.reject(
                "Invalid data in the result of the activity",
                INVALID_CONTEXT_ERROR
            )
            return
        }
        val errorCode = data.getIntExtra(
            RESULT_EXTRA_PREFIX + RESULT_ERROR_CODE,
            0
        )
        var errorMessage = data.getStringExtra(
            RESULT_EXTRA_PREFIX + RESULT_ERROR_MESSAGE
        )
        when (resultType) {
            BiometryResultType.SUCCESS -> invoke.resolve()
            BiometryResultType.FAILURE ->         // Biometry was successfully presented but was not recognized
                invoke.reject(errorMessage, BIOMETRIC_FAILURE)

            BiometryResultType.ERROR -> {
                // The user cancelled, the system cancelled, or some error occurred.
                // If the user cancelled, errorMessage is the text of the "negative" button,
                // which is not especially descriptive.
                if (errorCode == BiometricPrompt.ERROR_NEGATIVE_BUTTON) {
                    errorMessage = "Cancel button was pressed"
                }
                invoke.reject(errorMessage, biometryErrorCodeMap[errorCode])
            }
        }
    }

    internal enum class BiometryType(val type: Int) {
        NONE(0), FINGERPRINT(1), FACE(2), IRIS(3);
    }
}
