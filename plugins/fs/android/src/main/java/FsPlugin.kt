// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package com.plugin.fs

import android.app.Activity
import android.net.Uri
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@InvokeArg
class WriteTextFileArgs {
  val uri: String = ""
  val content: String = ""
}

@TauriPlugin
class FsPlugin(private val activity: Activity): Plugin(activity) {
    @Command
    fun writeTextFile(invoke: Invoke) {
        val args = invoke.parseArgs(WriteTextFileArgs::class.java)
        val uri = Uri.parse(args.uri)
        val content = args.content

        if(uri != null){
          activity.getContentResolver().openOutputStream(uri).use { ost ->
            if(ost != null && content != null){
              ost.write(content.toByteArray());
            }
          }
        }

        val ret = JSObject()
        invoke.resolve(ret)
    }
}

