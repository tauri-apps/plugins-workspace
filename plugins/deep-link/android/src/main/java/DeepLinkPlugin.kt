// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.deep_link

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@InvokeArg
class SetEventHandlerArgs {
    lateinit var handler: Channel
}

@TauriPlugin
class DeepLinkPlugin(private val activity: Activity): Plugin(activity) {
    //private val implementation = Example()
    private var webView: WebView? = null
    private var currentUrl: String? = null
    private var channel: Channel? = null

    companion object {
        var instance: DeepLinkPlugin? = null
    }

    @Command
    fun getCurrent(invoke: Invoke) {
        val ret = JSObject()
        ret.put("url", this.currentUrl)
        invoke.resolve(ret)
    }

    // This command should not be added to the `build.rs` and exposed as it is only
    // used internally from the rust backend.
    @Command
    fun setEventHandler(invoke: Invoke) {
        val args = invoke.parseArgs(SetEventHandlerArgs::class.java)
        this.channel = args.handler
        invoke.resolve()
    }

    override fun load(webView: WebView) {
        instance = this

        val intent = activity.intent

        if (intent.action == Intent.ACTION_VIEW) {
            // TODO: check if it makes sense to split up init url and last url
            this.currentUrl = intent.data.toString()
            val event = JSObject()
            event.put("url", this.currentUrl)
            this.channel?.send(event)
        }

        super.load(webView)
        this.webView = webView
    }

    override fun onNewIntent(intent: Intent) {
        if (intent.action == Intent.ACTION_VIEW) {
            this.currentUrl = intent.data.toString()
            val event = JSObject()
            event.put("url", this.currentUrl)
            this.channel?.send(event)
        }
    }
}
