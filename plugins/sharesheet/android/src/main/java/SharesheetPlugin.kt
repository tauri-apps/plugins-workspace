// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.sharesheet

import android.app.Activity
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.webkit.WebView
import android.net.Uri
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
class ShareTextOptions {
    lateinit var text: String
    var mimeType: String = "text/plain"
    var title: String? = null
    var thumbnailUri: Uri? = null
}


@TauriPlugin
class SharesheetPlugin(private val activity: Activity): Plugin(activity) {
    /**
     * Open the Sharesheet to share some text
     */
    @Command
    fun shareText(invoke: Invoke) {        
        val args = invoke.parseArgs(ShareTextOptions::class.java)

        val sendIntent: Intent = Intent().apply {
            action = Intent.ACTION_SEND
            putExtra(Intent.EXTRA_TEXT, args.text)
            putExtra(Intent.EXTRA_TITLE, args.title)
            type = args.mimeType
            data = args.thumbnailUri
            flags = Intent.FLAG_GRANT_READ_URI_PERMISSION
        }

        val shareIntent = Intent.createChooser(sendIntent, null);
        shareIntent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
        activity.applicationContext?.startActivity(shareIntent);
    }
}
