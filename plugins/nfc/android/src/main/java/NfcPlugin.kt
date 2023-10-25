// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.nfc

import android.app.Activity
import android.app.PendingIntent
import android.content.Intent
import android.content.IntentFilter
import android.nfc.NdefMessage
import android.nfc.NdefRecord
import android.nfc.NfcAdapter
import android.nfc.Tag
import android.nfc.tech.Ndef
import android.nfc.tech.NdefFormatable
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import org.json.JSONArray
import java.io.IOException
import kotlin.concurrent.thread

enum class NfcAction {
    NONE, READ, WRITE
}

@TauriPlugin
class NfcPlugin(private val activity: Activity) : Plugin(activity) {
    private lateinit var webView: WebView
    private var savedIntent: Intent? = null;

    private var available = false
    private var errorReason: String? = null

    private var nfcAdapter: NfcAdapter? = null

    // all actual tag handling happens in onNewIntent so we need to keep track of how to handle the intent here
    private var pendingNfcAction = NfcAction.NONE
    private var messageToWrite: NdefMessage? = null
    private var invokeHandler: Invoke? = null

    override fun load(webView: WebView) {
        super.load(webView)
        this.webView = webView

        this.nfcAdapter = NfcAdapter.getDefaultAdapter(activity.applicationContext)

        if (this.nfcAdapter === null) {
            this.errorReason = "Device does not have NFC capabilities"
        } else if (this.nfcAdapter?.isEnabled == false) {
            // TODO: re-check this in case user enabled it.
            this.errorReason = "NFC is disabled in device settings"
        } else {
            this.available = true
        }



        this.errorReason?.let { it ->
            Logger.error("NFC", it, null)
        }
    }

    override fun onNewIntent(intent: Intent) {
        Logger.info("NFC", "onNewIntent")
        super.onNewIntent(intent)
        when (pendingNfcAction) {
            NfcAction.NONE -> {}
            NfcAction.READ -> readTag(intent)
            NfcAction.WRITE -> thread {
                writeTag(intent)
            }
        }
    }

    override fun onPause() {
        super.onPause()
        Logger.info("NFC", "onPause")
        disableNFCInForeground()
    }

    override fun onResume() {
        super.onResume()
        Logger.info("NFC", "onResume")
        if (this.pendingNfcAction != NfcAction.NONE) {
            enableNFCInForeground()
        }
    }

    @Command
    fun isAvailable(invoke: Invoke) {
        val ret = JSObject()
        ret.put("available", this.available)
        invoke.resolve(ret)
    }

    @Command
    fun read(invoke: Invoke) {
        if (this.nfcAdapter === null) {
            invoke.reject("NFC writing unavailable: " + this.errorReason)
            return
        }

        enableNFCInForeground()
        this.pendingNfcAction = NfcAction.READ
        this.invokeHandler = invoke
    }

    @Command
    fun write(invoke: Invoke) {
        if (this.nfcAdapter === null) {
            invoke.reject("NFC writing unavailable: " + this.errorReason)
            return
        }

        enableNFCInForeground()

        val records = invoke.getArray("records")
        if (records === null) {
            invoke.reject("`records` array is required")
            disableNFCInForeground()
            return
        }

        val ndefRecords: MutableList<NdefRecord> = ArrayList()
        for (record in records.toList<JSObject>()) {
            val format = record.getInteger("format", 0)
            val type = toU8Array(record.getJSONArray("kind"))
            val identifier = toU8Array(record.getJSONArray("id"))
            val payload = toU8Array(record.getJSONArray("payload"))

            ndefRecords.add(NdefRecord(format.toShort(), type, identifier, payload))
        }

        this.messageToWrite = NdefMessage(ndefRecords.toTypedArray())
        this.pendingNfcAction = NfcAction.WRITE
        this.invokeHandler = invoke

        Logger.warn("NFC", "Write Mode Enabled")
    }

    private fun readTag(intent: Intent) {}

    private fun writeTag(intent: Intent) {
        if (messageToWrite !== null) { // This should always be true

            val tag = intent.getParcelableExtra<Tag>(NfcAdapter.EXTRA_TAG)

            // This should return tags that are already in ndef format
            val ndefTag = Ndef.get(tag)

            if (ndefTag !== null) {

                // We have to connect first to check maxSize.
                try {
                    ndefTag.connect()
                } catch (e: IOException) {
                    invokeHandler?.reject("Couldn't connect to NFC tag", e)
                    disableNFCInForeground()
                    return
                }

                if (ndefTag.maxSize < messageToWrite!!.toByteArray().size) {
                    invokeHandler?.reject("The message is too large for the provided NFC tag")
                } else if (!ndefTag.isWritable) {
                    invokeHandler?.reject("NFC tag is read-only")
                } else {
                    try {
                        ndefTag.writeNdefMessage(messageToWrite)
                        invokeHandler?.resolve()
                    } catch (e: Exception) {
                        invokeHandler?.reject("Couldn't write message to NFC tag", e)
                    }
                }

                ndefTag.close() // TODO: Catch possible IOException
                disableNFCInForeground()
                return

            }

            // This should cover tags that are not yet in ndef format but can be converted
            val ndefFormatableTag = NdefFormatable.get(tag)

            if (ndefFormatableTag !== null) {

                try {
                    ndefFormatableTag.connect()
                    ndefFormatableTag.format(messageToWrite)
                    invokeHandler?.resolve()
                } catch (e: Exception) {
                    invokeHandler?.reject("Couldn't format tag as Ndef", e)
                }

                ndefFormatableTag.close() // TODO: Catch possible IOException
                disableNFCInForeground()
                return

            }
        }

        // if we get to this line, the tag was neither Ndef nor NdefFormatable compatible
        invokeHandler?.reject("Tag doesn't support Ndef format")
    }

    private fun enableNFCInForeground() {
        val pendingIntent = PendingIntent.getActivity(
            activity, 0,
            Intent(
                activity,
                activity.javaClass
            ).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP/* or Intent.FLAG_ACTIVITY_CLEAR_TOP*/),
            PendingIntent.FLAG_MUTABLE
        )

        // TODO: add other 2 actions if we actually set filters below
        val nfcIntentFilter = IntentFilter(NfcAdapter.ACTION_NDEF_DISCOVERED)
        val filters = arrayOf(nfcIntentFilter)

        val techLists =
            arrayOf(arrayOf(Ndef::class.java.name), arrayOf(NdefFormatable::class.java.name))

        // TODO: check again after adding the reader portion
        //nfcAdapter?.enableForegroundDispatch(activity, pendingIntent, filters, techLists)
        nfcAdapter?.enableForegroundDispatch(activity, pendingIntent, null, null)
    }

    private fun disableNFCInForeground() {
        nfcAdapter?.disableForegroundDispatch(activity)
    }
}

fun toU8Array(jsarray: JSONArray): ByteArray {
    return ByteArray(jsarray.length()) { i -> jsarray.getInt(i).toByte() }
}
