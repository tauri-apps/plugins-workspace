// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.store

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import java.io.File

@TauriPlugin
class StorePlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun load(invoke: Invoke) {
        try {
            val path = invoke.parseArgs(String::class.java)
            val file = File(activity.applicationContext.getExternalFilesDir(null), path)

            invoke.resolveObject(ObjectMapper().readTree(file))
        } catch (ex: Exception) {
            invoke.reject(ex.message)
        }
    }

    @Command
    fun save(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(JsonNode::class.java)
            val path = args.get("store").asText()
            val cache = args.get("cache")
            val file = File(activity.applicationContext.getExternalFilesDir(null), path)

            if (!file.exists()) {
                file.parentFile?.mkdirs()
                file.createNewFile()
            }

            file.writeText(cache.toString())

            invoke.resolve()
        } catch (ex: Exception) {
            invoke.reject(ex.message)
        }
    }
}