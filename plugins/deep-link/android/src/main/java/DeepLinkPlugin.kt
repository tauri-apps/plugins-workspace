// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.deep_link

import android.app.Activity
import android.content.Intent
import android.os.PatternMatcher
import android.webkit.WebView
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import androidx.core.net.toUri

@InvokeArg
class SetEventHandlerArgs {
    lateinit var handler: Channel
}

@InvokeArg
class AssociatedDomain {
    var scheme: List<String> = listOf("https", "http")
    var host: String? = null
    var path: List<String> = listOf()
    var pathPattern: List<String> = listOf()
    var pathPrefix: List<String> = listOf()
    var pathSuffix: List<String> = listOf()
}

@InvokeArg
class PluginConfig {
    var mobile: List<AssociatedDomain> = listOf()
}

@TauriPlugin
class DeepLinkPlugin(private val activity: Activity): Plugin(activity) {
    //private val implementation = Example()
    private var webView: WebView? = null
    private var currentUrl: String? = null
    private var channel: Channel? = null
    private var config: PluginConfig? = null
    companion object {
        var instance: DeepLinkPlugin? = null
    }

    private fun isViewIntent(action: String?): Boolean {
        return action == Intent.ACTION_VIEW || action == "org.chromium.arc.intent.action.VIEW"
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
        config = getConfig(PluginConfig::class.java)

        super.load(webView)
        this.webView = webView

        val intent = activity.intent

        if (isViewIntent(intent.action) && intent.data != null) {
            val url = intent.data.toString()
            if (isDeepLink(url)) {
                // TODO: check if it makes sense to split up init url and last url
                this.currentUrl = url
                val event = JSObject()
                event.put("url", this.currentUrl)
                this.channel?.send(event)
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        if (isViewIntent(intent.action) && intent.data != null) {
            val url = intent.data.toString()
            if (isDeepLink(url)) {
                this.currentUrl = url
                val event = JSObject()
                event.put("url", this.currentUrl)
                this.channel?.send(event)
            }
        }
    }

    private fun isDeepLink(url: String): Boolean {
        val config = this.config ?: return false
        
        if (config.mobile.isEmpty()) {
            return false
        }

        val uri = try {
            url.toUri()
        } catch (_: Exception) {
            // not a URL
            return false
        }

        val scheme = uri.scheme ?: return false
        val host = uri.host
        val path = uri.path ?: ""

        // Check if URL matches any configured mobile deep link
        for (domain in config.mobile) {
            // Check scheme
            if (!domain.scheme.any { it.equals(scheme, ignoreCase = true) }) {
                continue
            }

            // Check host (if configured)
            if (domain.host != null) {
                if (!host.equals(domain.host, ignoreCase = true)) {
                    continue
                }
            }

            // Check path constraints
            // According to Android docs:
            // - path: exact match, must begin with /
            // - pathPrefix: matches initial part of path
            // - pathSuffix: matches ending part, doesn't need to begin with /
            // - pathPattern: simple glob pattern (., *, .*)
            val pathMatches = when {
                // Exact path match (must begin with /)
                domain.path.isNotEmpty() && domain.path.any { it == path } -> true
                // Path pattern match (simple glob: ., *, .*)
                domain.pathPattern.isNotEmpty() && domain.pathPattern.any { pattern ->
                    try {
                        PatternMatcher(pattern, PatternMatcher.PATTERN_SIMPLE_GLOB).match(path)
                    } catch (e: Exception) {
                        false
                    }
                } -> true
                // Path prefix match
                domain.pathPrefix.isNotEmpty() && domain.pathPrefix.any { prefix ->
                    path.startsWith(prefix)
                } -> true
                // Path suffix match
                domain.pathSuffix.isNotEmpty() && domain.pathSuffix.any { suffix ->
                    path.endsWith(suffix)
                } -> true
                // If no path constraints, any path is allowed
                domain.path.isEmpty() && domain.pathPattern.isEmpty() && 
                domain.pathPrefix.isEmpty() && domain.pathSuffix.isEmpty() -> true
                else -> false
            }

            if (pathMatches) {
                return true
            }
        }

        return false
    }
}
