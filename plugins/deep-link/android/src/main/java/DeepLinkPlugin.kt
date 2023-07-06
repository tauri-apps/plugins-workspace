// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.deep_link

import android.app.Activity
import android.content.Intent
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@TauriPlugin
class DeepLinkPlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = Example()
    private var webView: WebView? = null
    private var lastUrl: String? = null

    companion object {
        var instance: DeepLinkPlugin? = null
    }

    @Command
    fun getLastLink(invoke: Invoke) {
        val ret = JSObject()
        ret.put("url", this.lastUrl ?: "")
        invoke.resolve(ret)
    }

    /* @Command
    fun registerListenerRust(invoke: Invoke) {
        val value = invoke.getString("value") ?: ""
        val ret = JSObject()
        ret.put("value", this.lastUrl ?: "none")
        invoke.resolve(ret)
    } */

    override fun load(webView: WebView) {
        instance = this

        if (intent.action == intent.ACTION_VIEW) {
            // TODO: check if it makes sense to split up init url and last url
            this.lastUrl = intent.action.toString() + intent.data.toString()
            // TODO: Emit event - may not work here timing wise
        }

        super.load(webView)
        this.webView = webView
    }

    override fun onNewIntent(intent: Intent) {
        if (intent.action == intent.ACTION_VIEW) {
            this.lastUrl = intent.data.toString()
            // TODO: Emit event
        }
    }
}
