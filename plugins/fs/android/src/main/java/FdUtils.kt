// Copyright 2025 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.plugin.fs

import android.content.ContentResolver
import android.content.Context
import android.content.res.AssetFileDescriptor.UNKNOWN_LENGTH
import android.net.Uri
import android.os.Build
import android.os.Handler
import android.os.HandlerThread
import android.os.ParcelFileDescriptor
import android.os.ProxyFileDescriptorCallback
import android.os.storage.StorageManager
import android.system.ErrnoException
import android.system.OsConstants
import androidx.annotation.RequiresApi
import java.io.FileOutputStream
import java.io.OutputStream
import kotlin.math.max


/**
 * This is basically the same as `ContentResolver.openAssetFileDescriptor`,
 * but with a few inner modifications.
 */
@Suppress("Recycle")
fun openFd(uri: Uri, mode: String, ctx: Context): ParcelFileDescriptor {
    if (isWritableMode(mode) && needWriteViaOutputStream(uri)) {
        return openWritableFdViaOutputStream(uri, mode, ctx)
    }

    return ctx.contentResolver
        .openAssetFileDescriptor(uri, mode)
        ?.parcelFileDescriptor
        ?: throw IllegalArgumentException("Failed to open file: $uri")
}

private fun openWritableFdViaOutputStream(uri: Uri, mode: String, ctx: Context): ParcelFileDescriptor {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) {
        throw IllegalArgumentException("Unsupported URI: $uri")
    }

    val openAfd = {
        ctx.contentResolver
            .openAssetFileDescriptor(uri, mode)
            ?: throw IllegalArgumentException("Failed to open file: $uri")
    }

    val output: FileOutputStream
    val outputInitLen: Long
    val appendMode: Boolean
    when (mode) {
        "w" -> {
            // NOTE:
            // Until Android 10, "w" mode will always truncate existing contents.
            // Since Android 10, "w" mode will or will not truncate existing contents.
            // - https://issuetracker.google.com/issues/180526528

            val fd = openAfd()
            val len = try { fd.length } catch (_: Exception) { UNKNOWN_LENGTH }
            outputInitLen = len
            appendMode = false
            output = fd.createOutputStream()
        }
        "wt" -> {
            val fd = openAfd()
            outputInitLen = 0
            appendMode = false
            output = fd.createOutputStream()
        }
        "wa" -> {
            val fd = openAfd()
            val len = try { fd.length } catch (_: Exception) { UNKNOWN_LENGTH }
            outputInitLen = len
            appendMode = true
            output = fd.createOutputStream()
        }
        else -> throw IllegalArgumentException("Unsupported mode: $mode")
    }

    val sm = ctx.getSystemService(Context.STORAGE_SERVICE) as StorageManager

    return sm.openProxyFileDescriptor(
        ParcelFileDescriptor.MODE_WRITE_ONLY,
        UnseekableWriteOnlyFileBehaviorWithOutputStream(output, outputInitLen, appendMode) {
            SingleThreadHandlerManager.notifyTaskEnd()
        },
        SingleThreadHandlerManager.getHandlerAndNotifyTaskAdd()
    )
}

private fun isWritableMode(mode: String): Boolean {
    // Mode is one of: r, rw, w, wa, wt, rwt
    return mode == "w" || mode == "wt" || mode == "wa" || mode == "rw" || mode == "rwt"
}

private fun needWriteViaOutputStream(uri: Uri): Boolean {
    // Returns true if the file is on Google Drive.
    // - https://github.com/tauri-apps/plugins-workspace/issues/3109
    return uri.scheme == ContentResolver.SCHEME_CONTENT
            && uri.authority?.startsWith("com.google.android.apps.docs") == true
}

/**
 * # Note
 * Currently, a single-threaded SingleThreadHandlerManager is used with this,
 * so no synchronization is needed.
 * But it may be required if a multi-threaded handler is used. (Or not?)
 */
@RequiresApi(Build.VERSION_CODES.O)
private class UnseekableWriteOnlyFileBehaviorWithOutputStream(
    private val output: OutputStream,
    private val outputInitLen: Long,
    private val appendMode: Boolean,
    private val onRelease: (() -> Unit)?
) : ProxyFileDescriptorCallback() {

    private var writtenLen: Long = 0

    override fun onRead(offset: Long, size: Int, data: ByteArray): Int {
        throw ErrnoException("read", OsConstants.EBADF)
    }

    override fun onWrite(offset: Long, size: Int, data: ByteArray?): Int {
        try {
            if (offset < 0 || size < 0) throw ErrnoException("write", OsConstants.EINVAL)

            // If `appendMode` is enabled, it behaves the same as the O_APPEND flag.
            // That is, data is always written to the end of the file, regardless of the current seek-position.
            // So, skip the seek check and ignore the offset in that case.
            if (!appendMode) {
                // Forbid file seeking.
                //
                // Since seeking is not possible,
                // if not in append mode,
                // the seek position start at 0 and advance by the just amount written.
                // So the offset should match the number of bytes written.
                // Otherwise, it implies that a seek operation has occurred.
                if (offset != writtenLen) {
                    throw ErrnoException("write", OsConstants.ESPIPE)
                }
            }

            if (data == null) return 0
            if (data.isEmpty()) return 0
            if (size == 0) return 0

            val writeSize = size.coerceAtMost(data.size)
            output.write(data, 0, writeSize)
            writtenLen += writeSize

            return writeSize
        }
        catch (e: ErrnoException) {
            throw e
        }
        catch (e: Exception) {
            throw ErrnoException("write", OsConstants.EIO, e)
        }
    }

    override fun onFsync() {
        try {
            output.flush()
        }
        catch (e: ErrnoException) {
            throw e
        }
        catch (e: Exception) {
            throw ErrnoException("fsync", OsConstants.EIO, e)
        }
    }

    override fun onRelease() {
        try {
            output.flush()
        }
        catch (_: Exception) {}

        try {
            output.close()
        }
        catch (_: Exception) {}

        try {
            onRelease?.invoke()
        }
        catch (_: Exception) {}
    }

    override fun onGetSize(): Long {
        if (outputInitLen < 0) {
            return UNKNOWN_LENGTH
        }

        return if (appendMode) {
            // If `appendMode` is enabled, it behaves the same as the O_APPEND flag.
            // That is, data is always written to the end of the file, regardless of the current seek position.
            // So, the file size will always match the initial size plus the number of bytes written.
            outputInitLen + writtenLen
        } else {
            // Since seeking is not possible,
            // if not in append mode,
            // the seek position start at 0 and advance by the just amount written.
            // And, the file size will never be smaller than the initial size.
            max(outputInitLen, writtenLen)
        }
    }
}

private class SingleThreadHandlerManager {
    companion object {
        private var handlerThread: HandlerThread? = null
        private var handler: Handler? = null
        private var taskCount = 0

        @Synchronized
        fun getHandlerAndNotifyTaskAdd(): Handler {
            taskCount++
            handlerThread?.let { thread ->
                val currentHandler = handler
                if (thread.isAlive && currentHandler != null) return currentHandler
            }

            handlerThread = HandlerThread("ProxyFDThread").apply { start() }
            handler = Handler(handlerThread!!.looper)
            return handler!!
        }

        @Synchronized
        fun notifyTaskEnd() {
            taskCount--
            if (taskCount <= 0) {
                handlerThread?.quitSafely()
                handlerThread = null
                handler = null
                taskCount = 0
            }
        }
    }
}