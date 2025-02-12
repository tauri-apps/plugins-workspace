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
import java.util.concurrent.Executor
import android.util.Base64
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import javax.crypto.spec.SecretKeySpec
//import javax.crypto.spec.IvParameterSpec
import java.nio.charset.Charset
import app.tauri.biometric.CipherType

class BiometricActivity : AppCompatActivity() {
    private var ENCRYPTION_ALGORITHM = KeyProperties.KEY_ALGORITHM_AES
    private val ENCRYPTION_BLOCK_MODE = KeyProperties.BLOCK_MODE_GCM
    private val ENCRYPTION_PADDING = KeyProperties.ENCRYPTION_PADDING_NONE
    private val KEYSTORE_NAME = "AndroidKeyStore"
    private val KEY_ALIAS = "tauri-plugin-biometric-key"

    @SuppressLint("WrongConstant")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.auth_activity)

        val promptInfo = createPromptInfo()
        val prompt = createBiometricPrompt()
        
        cipherOperation = intent.hasExtra(BiometricPlugin.ENCRYPT_DECRYPT_OPERATION)
        if (!cipherOperation) {
            prompt.authenticate(promptInfo)
            return
        }

        intent.getStringExtra(BiometricPlugin.ENCRYPT_DECRYPT_DATA)?.let {
            encryptDecryptData = it
        }

        try {
            val type = CipherType.values()[intent.getIntExtra(BiometricPlugin.CIPHER_OPERATION_TYPE, CipherType.ENCRYPT.ordinal)]
            cipherType = type
        } catch (e: ArrayIndexOutOfBoundsException) {
            finishActivity(
                BiometryResultType.ERROR,
                0,
                "Couldn't identify the cipher operation type (encrypt/decrypt)!"
            )
            return
        }

        val cipher = getCipher()
        prompt.authenticate(promptInfo, BiometricPrompt.CryptoObject(cipher))
    }

    @JvmOverloads
    fun finishActivity(
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
            .putExtra(
                prefix + BiometricPlugin.ENCRYPT_DECRYPT_OPERATION, 
                cipherOperation
            )

        if (cipherOperation) {
            val cryptoObject = requireNotNull(authenticationResult?.cryptoObject)
            val cipher = requireNotNull(cryptoObject.cipher)
            var dataToProcess = if (cipherType == CipherType.ENCRYPT) {
                encryptDecryptData.toByteArray()
            } else {
                val (_, encryptedData) = decodeEncryptedData(encryptDecryptData)
                encryptedData
            }
            var processedData: ByteArray = cipher.doFinal(dataToProcess)

            if (cipherType == CipherType.ENCRYPT) {
                // Converts the encrypted data to Base64 string
                val encodedString = encodeEncryptedData(cipher, processedData)
                intent.putExtra(
                    prefix + BiometricPlugin.RESULT_ENCRYPT_DECRYPT_DATA,
                    encodedString
                )
            } else {
                // For decryption, return the decrypted string
                intent.putExtra(
                    prefix + BiometricPlugin.RESULT_ENCRYPT_DECRYPT_DATA,
                    String(processedData, Charset.forName("UTF-8"))
                )
            }
        }
        setResult(Activity.RESULT_OK, intent)
        finish()
    }

    private fun createPromptInfo(): BiometricPrompt.PromptInfo {
        val builder = BiometricPrompt.PromptInfo.Builder()
        val intent = intent
        var title = intent.getStringExtra(BiometricPlugin.TITLE)
        val subtitle = intent.getStringExtra(BiometricPlugin.SUBTITLE)
        val description = intent.getStringExtra(BiometricPlugin.REASON)
        
        allowDeviceCredential = false
        cipherOperation = intent.hasExtra(BiometricPlugin.ENCRYPT_DECRYPT_OPERATION)

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
            var authenticators = if (cipherOperation) {
                BiometricManager.Authenticators.BIOMETRIC_STRONG
            } else {
                BiometricManager.Authenticators.BIOMETRIC_WEAK
            }
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
        
        return builder.build()
    }
    
    private fun createBiometricPrompt(): BiometricPrompt {
        val executor = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            this.mainExecutor
        } else {
            Executor { command: Runnable? ->
                Handler(this.mainLooper).post(
                    command!!
                )
            }
        }

        val callback = object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationError(
                errorCode: Int,
                errorMessage: CharSequence
            ) {
                super.onAuthenticationError(errorCode, errorMessage)
                finishActivity(
                    BiometryResultType.ERROR,
                    errorCode,
                    errorMessage as String
                )
            }

            override fun onAuthenticationSucceeded(
                result: BiometricPrompt.AuthenticationResult
            ) {
                super.onAuthenticationSucceeded(result)
                authenticationResult = result
                finishActivity()
            }
        }

        return BiometricPrompt(this, executor, callback)
    }

    /**
     * Get the key if exists, or create a new one if not
     */
    private fun getEncryptionKey(): SecretKey {
        val keyStore = KeyStore.getInstance(KEYSTORE_NAME)
        keyStore.load(null)
        keyStore.getKey(KEY_ALIAS, null)?.let { return it as SecretKey }

        // from here ahead, we will move only if there is not a key with the provided alias
        val keyGenerator = KeyGenerator.getInstance(ENCRYPTION_ALGORITHM, KEYSTORE_NAME)

        val builder = KeyGenParameterSpec.Builder(
                KEY_ALIAS, 
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            )
            .setBlockModes(ENCRYPTION_BLOCK_MODE)
            .setEncryptionPaddings(ENCRYPTION_PADDING)
            .setKeySize(256)
            .setRandomizedEncryptionRequired(true)
            .setUserAuthenticationRequired(true) // Forces to use the biometric authentication to create/retrieve the key

        keyGenerator.init(builder.build())
        return keyGenerator.generateKey()
    }

    private fun getCipher(): Cipher {
        val biometricKey = getEncryptionKey()

        val cipher = Cipher.getInstance("$ENCRYPTION_ALGORITHM/$ENCRYPTION_BLOCK_MODE/$ENCRYPTION_PADDING")

        if (cipherType == CipherType.ENCRYPT) {
            cipher.init(Cipher.ENCRYPT_MODE, biometricKey)
        } else {
            // Decodes the Base64 string to get the encrypted data and IV
            val (iv, _) = decodeEncryptedData(encryptDecryptData)
            cipher.init(Cipher.DECRYPT_MODE, biometricKey, GCMParameterSpec(128, iv))
        }
        return cipher
    }

    private fun encodeEncryptedData(cipher: Cipher, rawEncryptedData: ByteArray): String {
        val encodedData = Base64.encodeToString(rawEncryptedData, Base64.NO_WRAP)
        val encodedIv = Base64.encodeToString(cipher.iv, Base64.NO_WRAP)

        val encodedString = encodedData + ";" + encodedIv
        return encodedString
    }

    private fun decodeEncryptedData(encryptedDataEncoded: String): List<ByteArray> {
        val (data, iv) = encryptedDataEncoded.split(";")
        val decodedData = Base64.decode(data, Base64.NO_WRAP)
        val decodedIv = Base64.decode(iv, Base64.NO_WRAP)

        val ret = listOf(decodedIv, decodedData)
        return ret
    }

    companion object {
        var allowDeviceCredential = false
        var cipherOperation = false
        var encryptDecryptData: String = ""
        var authenticationResult: BiometricPrompt.AuthenticationResult? = null
        var cipherType: CipherType = CipherType.ENCRYPT
    }
}