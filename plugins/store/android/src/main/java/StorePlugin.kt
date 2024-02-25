// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.store

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import com.fasterxml.jackson.databind.annotation.JsonDeserialize

@InvokeArg
@JsonDeserialize
class KeyValue {
    lateinit var store: String
    lateinit var key: String
    // TODO Any not only string
    var value: String? = null
}

@InvokeArg
@JsonDeserialize
class Key {
    lateinit var key: String
}

@TauriPlugin
class StorePlugin(private val activity: Activity) : Plugin(activity) {
    private val manager: SharedPreferences = activity.applicationContext.getSharedPreferences(
        "${activity.applicationContext.packageName}.Store",
        Context.MODE_PRIVATE
    )

    private val storesPreferences: HashMap<String, SharedPreferences> = hashMapOf()

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
    }

    override fun load(webView: WebView) {
        super.load(webView)


    }

    @Command
    fun set(invoke: Invoke) {
        val args = invoke.parseArgs(KeyValue::class.java)

        if (storesPreferences[args.store] == null) {
            storesPreferences[args.store] = activity.applicationContext.getSharedPreferences(
                "${activity.applicationContext.packageName}.${args.store}",
                Context.MODE_PRIVATE
            )
        }

        // TODO Any not only string
        storesPreferences[args.store]!!.edit().putString(args.key, args.value).apply()
    }

    @Command
    fun setFloat(invoke: Invoke) {
        val args = invoke.parseArgs(KeyValue::class.java)
        // TODO Any not only string
        manager.edit().putFloat(args.key, args.value!!.toFloat()).apply()
    }


    @Command
    fun get(invoke: Invoke) {
        val args = invoke.parseArgs(Key::class.java)
        // TODO Any not only string
        val value = manager.getString(args.key, null)
        if (value == null) {
            invoke.resolve()
        } else {
            invoke.resolveObject(value)
        }
    }

    @Command
    fun has(invoke: Invoke) {
        val args = invoke.parseArgs(Key::class.java)
        val has = manager.contains(args.key)
        invoke.resolveObject(has)
    }

    @Command
    fun delete(invoke: Invoke) {
        val args = invoke.parseArgs(Key::class.java)
        if (manager.contains(args.key)) {
            manager.edit().remove(args.key).apply()
        } else {
            invoke.reject("key not found")
        }
    }

    @Command
    fun clear(invoke: Invoke) {
        manager.edit().clear().apply()
    }

    @Command
    fun reset(invoke: Invoke) {
        clear(invoke)
    }

    @Command
    fun keys(invoke: Invoke) {
        val keys = manager.all.keys
        invoke.resolveObject(keys.toTypedArray())
    }

    @Command
    fun values(invoke: Invoke) {
        val values = manager.all.values
        invoke.resolveObject(values.toTypedArray().map {
            // TODO Any not only string
            it.toString()
        }.toTypedArray())
    }

    @Command
    fun length(invoke: Invoke) {
        val length = manager.all.size
        invoke.resolveObject(length)
    }

    @Command
    fun entries(invoke: Invoke) {
        // TODO map
        val entries = manager.all.entries.toTypedArray()
        invoke.resolveObject(entries)
    }

    @Command
    fun load(invoke: Invoke) {
        invoke.resolve()
    }

    @Command
    fun save(invoke: Invoke) {
        invoke.resolve()
    }
}
