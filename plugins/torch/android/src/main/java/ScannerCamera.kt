// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.torch

import androidx.camera.core.Camera
import androidx.camera.core.TorchState
import java.util.concurrent.ExecutionException
import java.util.concurrent.Executor

/**
 * The camera held by an active barcode scan.
 *
 * While CameraX holds the camera, `CameraManager.setTorchMode` fails with `CAMERA_IN_USE`, so the
 * torch has to go through the camera's own `CameraControl`. This is the only class referencing
 * CameraX, which is only on the classpath when the barcode scanner plugin is; only construct it
 * from a camera returned by [TorchPlugin.findScannerCamera].
 */
internal class ScannerCamera(camera: Any) {
    private val camera = camera as Camera

    val hasFlashUnit: Boolean
        get() = camera.cameraInfo.hasFlashUnit()

    val torchEnabled: Boolean
        get() = camera.cameraInfo.torchState.value == TorchState.ON

    /** Calls [callback] with `null` once the torch changed, or with the error that prevented it. */
    fun setTorch(enabled: Boolean, executor: Executor, callback: (Throwable?) -> Unit) {
        val future = camera.cameraControl.enableTorch(enabled)
        future.addListener({
            try {
                future.get()
                callback(null)
            } catch (e: ExecutionException) {
                callback(e.cause ?: e)
            } catch (e: Exception) {
                callback(e)
            }
        }, executor)
    }
}
