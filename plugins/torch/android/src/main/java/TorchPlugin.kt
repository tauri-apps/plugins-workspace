// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.torch

import android.app.Activity
import android.content.Context
import android.hardware.camera2.CameraAccessException
import android.hardware.camera2.CameraCharacteristics
import android.hardware.camera2.CameraManager
import android.os.Handler
import android.os.Looper
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

private const val ERROR_NOT_AVAILABLE = "The torch is not available on this device"
private const val SCANNER_PLUGIN_CLASS = "app.tauri.barcodescanner.BarcodeScannerPlugin"

@TauriPlugin
class TorchPlugin(private val activity: Activity) : Plugin(activity) {
    private val cameraManager =
        activity.getSystemService(Context.CAMERA_SERVICE) as CameraManager

    // the camera whose flash unit is used as the torch, or null if the device has none
    private val torchCameraId: String? = findTorchCameraId()

    // the system-reported state of the torch, kept in sync by torchCallback so changes made by
    // the system or other apps are reflected
    @Volatile
    private var torchEnabled = false

    private val torchCallback = object : CameraManager.TorchCallback() {
        override fun onTorchModeChanged(cameraId: String, enabled: Boolean) {
            if (cameraId == torchCameraId) torchEnabled = enabled
        }

        override fun onTorchModeUnavailable(cameraId: String) {
            if (cameraId == torchCameraId) torchEnabled = false
        }
    }

    init {
        if (torchCameraId != null) {
            cameraManager.registerTorchCallback(torchCallback, Handler(Looper.getMainLooper()))
        }
    }

    override fun onDestroy(activity: AppCompatActivity) {
        cameraManager.unregisterTorchCallback(torchCallback)
    }

    private fun findTorchCameraId(): String? {
        return try {
            val withFlash = cameraManager.cameraIdList.filter {
                cameraManager.getCameraCharacteristics(it)
                    .get(CameraCharacteristics.FLASH_INFO_AVAILABLE) == true
            }
            withFlash.firstOrNull {
                cameraManager.getCameraCharacteristics(it)
                    .get(CameraCharacteristics.LENS_FACING) == CameraCharacteristics.LENS_FACING_BACK
            } ?: withFlash.firstOrNull()
        } catch (e: Exception) {
            Logger.error("Failed to look up the camera flash unit: ${e.message}")
            null
        }
    }

    // The camera of an active barcode scan, if the barcode scanner plugin is installed and scanning.
    private fun findScannerCamera(): ScannerCamera? {
        val camera = try {
            Class.forName(SCANNER_PLUGIN_CLASS).getMethod("getActiveCamera").invoke(null)
        } catch (e: Exception) {
            null
        } ?: return null
        return ScannerCamera(camera)
    }

    private fun isTorchOn(): Boolean = findScannerCamera()?.torchEnabled ?: torchEnabled

    private fun setTorch(invoke: Invoke, enabled: Boolean, onSuccess: () -> Unit) {
        val action = if (enabled) "enable" else "disable"

        val scannerCamera = findScannerCamera()
        if (scannerCamera != null) {
            if (!scannerCamera.hasFlashUnit) {
                invoke.reject("The camera used by the barcode scanner has no flash unit")
                return
            }
            scannerCamera.setTorch(enabled, ContextCompat.getMainExecutor(activity)) { error ->
                if (error == null) {
                    onSuccess()
                } else {
                    invoke.reject("Failed to $action the torch: ${error.message}")
                }
            }
            return
        }

        val cameraId = torchCameraId
        if (cameraId == null) {
            invoke.reject(ERROR_NOT_AVAILABLE)
            return
        }
        try {
            cameraManager.setTorchMode(cameraId, enabled)
            torchEnabled = enabled
            onSuccess()
        } catch (e: CameraAccessException) {
            when (e.reason) {
                CameraAccessException.CAMERA_IN_USE, CameraAccessException.MAX_CAMERAS_IN_USE ->
                    invoke.reject("Failed to $action the torch: the camera is in use by another app")
                else -> invoke.reject("Failed to $action the torch: ${e.message}")
            }
        } catch (e: Exception) {
            invoke.reject("Failed to $action the torch: ${e.message}")
        }
    }

    @Command
    fun enable(invoke: Invoke) {
        setTorch(invoke, true) { invoke.resolve() }
    }

    @Command
    fun disable(invoke: Invoke) {
        setTorch(invoke, false) { invoke.resolve() }
    }

    @Command
    fun toggle(invoke: Invoke) {
        val enabled = !isTorchOn()
        setTorch(invoke, enabled) {
            val result = JSObject()
            result.put("enabled", enabled)
            invoke.resolve(result)
        }
    }

    @Command
    fun isAvailable(invoke: Invoke) {
        val result = JSObject()
        result.put("available", torchCameraId != null)
        invoke.resolve(result)
    }

    @Command
    fun isEnabled(invoke: Invoke) {
        val result = JSObject()
        result.put("enabled", isTorchOn())
        invoke.resolve(result)
    }
}
