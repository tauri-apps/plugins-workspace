// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.biometric

import android.annotation.SuppressLint
import android.app.Activity
import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.hardware.biometrics.BiometricManager
import android.os.Build
import android.os.Bundle
import android.os.Handler
import androidx.appcompat.app.AppCompatActivity
import androidx.biometric.BiometricPrompt
import androidx.biometric.BiometricPrompt.CryptoObject
import java.util.concurrent.Executor
import javax.crypto.Cipher
import java.util.Base64

class BiometricActivity : AppCompatActivity() {
    private lateinit var cryptographyManager: CryptographyManager

    @SuppressLint("WrongConstant")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        cryptographyManager = CryptographyManager()

        setContentView(R.layout.auth_activity)

        val executor = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            this.mainExecutor
        } else {
            Executor { command: Runnable? ->
                Handler(this.mainLooper).post(
                    command!!
                )
            }
        }

        val builder = BiometricPrompt.PromptInfo.Builder()
        val intent = intent
        var title = intent.getStringExtra(BiometricPlugin.TITLE)
        val subtitle = intent.getStringExtra(BiometricPlugin.SUBTITLE)
        val description = intent.getStringExtra(BiometricPlugin.REASON)
        val mode = AuthMode.valueOf(intent.getStringExtra(BiometricPlugin.MODE) ?: "PROMPT")
        val cipherKey = intent.getStringExtra(BiometricPlugin.CIPHER_KEY)
        val cipherDataData = intent.getStringExtra(BiometricPlugin.CIPHER_DATA_DATA)
        val cipherDataIV = intent.getStringExtra(BiometricPlugin.CIPHER_DATA_INITIALIZATION_VECTOR)
        allowDeviceCredential = false
        // Android docs say we should check if the device is secure before enabling device credential fallback
        val manager = getSystemService(
            Context.KEYGUARD_SERVICE
        ) as KeyguardManager
        if (manager.isDeviceSecure) {
            allowDeviceCredential =
                intent.getBooleanExtra(BiometricPlugin.DEVICE_CREDENTIAL, false)
        }

        if (title.isNullOrEmpty()) {
            title = "Authenticate"
        }

        builder.setTitle(title).setSubtitle(subtitle).setDescription(description)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            var authenticators = BiometricManager.Authenticators.BIOMETRIC_STRONG
            if (allowDeviceCredential) {
                authenticators = authenticators or BiometricManager.Authenticators.DEVICE_CREDENTIAL
            }
            builder.setAllowedAuthenticators(authenticators)
        } else {
            @Suppress("DEPRECATION")
            builder.setDeviceCredentialAllowed(allowDeviceCredential)
        }

        // From the Android docs:
        //  You can't call setNegativeButtonText() and setAllowedAuthenticators(... or DEVICE_CREDENTIAL)
        //  at the same time on a BiometricPrompt.PromptInfo.Builder instance.
        if (!allowDeviceCredential) {
            val negativeButtonText = intent.getStringExtra(BiometricPlugin.CANCEL_TITLE)
            builder.setNegativeButtonText(
                if (negativeButtonText.isNullOrEmpty()) "Cancel" else negativeButtonText
            )
        }
        builder.setConfirmationRequired(
            intent.getBooleanExtra(BiometricPlugin.CONFIRMATION_REQUIRED, true)
        )
        val promptInfo = builder.build()
        val prompt = BiometricPrompt(
            this,
            executor,
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationError(
                    errorCode: Int,
                    errorMessage: CharSequence
                ) {
                    super.onAuthenticationError(errorCode, errorMessage)
                    finishActivity(
                        null,
                        BiometryResultType.ERROR,
                        errorCode,
                        errorMessage as String
                    )
                }

                override fun onAuthenticationSucceeded(
                    result: BiometricPrompt.AuthenticationResult
                ) {
                    super.onAuthenticationSucceeded(result)
                    
                    finishActivity(processData(mode, cipherDataData, result.cryptoObject))
                }
            }
        )

        if (mode == AuthMode.ENCRYPT) {
            val cipher = cryptographyManager.getInitializedCipherForEncryption(cipherKey!!)
            prompt.authenticate(promptInfo, BiometricPrompt.CryptoObject(cipher))
        } else if (mode == AuthMode.DECRYPT) {
            val cipher = cryptographyManager.getInitializedCipherForDecryption(
                cipherKey!!,
                Base64.getDecoder().decode(cipherDataIV))
            prompt.authenticate(promptInfo, BiometricPrompt.CryptoObject(cipher))
        } else {
            prompt.authenticate(promptInfo)
        }
    }

    private fun processData(mode: AuthMode, cipherDataData: String?, cryptoObject: BiometricPrompt.CryptoObject?): CipherData? {
        if (mode == AuthMode.ENCRYPT) {
            if(cipherDataData == null) {
                throw IllegalStateException("No data to encrypt")
            }
            val encryptedData = cryptographyManager.encryptData(cipherDataData, cryptoObject?.cipher!!)

            return CipherData(
                Base64.getEncoder().encodeToString(encryptedData.ciphertext),
                Base64.getEncoder().encodeToString(encryptedData.initializationVector)
            )
        } else if (mode == AuthMode.DECRYPT) {
            if(cipherDataData == null) {
                throw IllegalStateException("No data to decrypt")
            }
            val decryptedData = cryptographyManager.decryptData(
                Base64.getDecoder().decode(cipherDataData),
                cryptoObject?.cipher!!)

            return CipherData(
                decryptedData,
                null
            )
        }

        return null
    }

    @JvmOverloads
    fun finishActivity(
        cipherData: CipherData?,
        resultType: BiometryResultType = BiometryResultType.SUCCESS,
        errorCode: Int = 0,
        errorMessage: String? = ""
    ) {
        val intent = Intent()
        val prefix = BiometricPlugin.RESULT_EXTRA_PREFIX
        intent
            .putExtra(prefix + BiometricPlugin.RESULT_TYPE, resultType.toString())
            .putExtra(prefix + BiometricPlugin.RESULT_ERROR_CODE, errorCode)
            .putExtra(
                prefix + BiometricPlugin.RESULT_ERROR_MESSAGE,
                errorMessage
            )
        if (cipherData != null) {
            intent
                .putExtra(prefix + BiometricPlugin.RESULT_DATA, cipherData.data)
                .putExtra(prefix + BiometricPlugin.RESULT_IV, cipherData.initializationVector)
        }
        setResult(Activity.RESULT_OK, intent)
        finish()
    }

    companion object {
        var allowDeviceCredential = false
    }
}