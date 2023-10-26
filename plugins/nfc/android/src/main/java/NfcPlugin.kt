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
import android.os.Build
import android.os.Parcelable
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.json.JSONArray
import java.io.IOException
import kotlin.concurrent.thread


enum class NfcAction {
    NONE, READ, WRITE
}

@TauriPlugin
class NfcPlugin(private val activity: Activity) : Plugin(activity) {
    private lateinit var webView: WebView

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

        this.errorReason?.let {
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
        disableNFCInForeground()
        super.onPause()
        Logger.info("NFC", "onPause")
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
    fun scan(invoke: Invoke) {
        if (this.nfcAdapter === null) {
            invoke.reject("NFC writing unavailable: " + this.errorReason)
            return
        }
        val kind = invoke.getString("kind")
        if (kind != "tag" && kind != "ndef") {
            invoke.reject("invalid `kind` argument, expected one of `tag`, `ndef`, got: '$kind'")
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

    // TODO: keepAlive?
    private fun readTag(intent: Intent) {
        try {
            val tag = intent.getParcelableExtra<Tag>(NfcAdapter.EXTRA_TAG)
            val rawMessages = intent.getParcelableArrayExtra(NfcAdapter.EXTRA_NDEF_MESSAGES)

            // For some reason this one never triggers.
            if (intent.action == NfcAdapter.ACTION_NDEF_DISCOVERED) {
                Logger.info("NFC", "new NDEF intent")

                readTagInner(tag, rawMessages)

            } else if (intent.action == NfcAdapter.ACTION_TECH_DISCOVERED) {
                // For some reason this always triggers instead of NDEF_DISCOVERED even though we set ndef filters right now
                Logger.info("NFC", "new TECH intent")

                // TODO: handle different techs. Don't assume ndef.

                readTagInner(tag, rawMessages)

            } else if (intent.action == NfcAdapter.ACTION_TAG_DISCOVERED) {
                // This should never trigger when an app handles NDEF and TECH

                // TODO: Don't assume ndef.

                readTagInner(tag, rawMessages)
            }

            this.pendingNfcAction = NfcAction.NONE
            disableNFCInForeground()
        } catch (e: Exception) {
            Logger.error("wtf", e)
        }
    }

    private fun readTagInner(tag: Tag?, rawMessages: Array<Parcelable>?) {
        val ndef = Ndef.get(tag)

        // iOS part only reads the first message (? - there are 2 conflicting impls so not sure) so we do too. i think that covers most if not all use cases anyway.
        val ndefMessage = rawMessages?.get(0) as NdefMessage

        val records = ndefMessage.records

        val jsonRecords = Array(records.size) { i -> recordToJson(records[i]) }

        val ret = JSObject()
        if (tag !== null) {
            ret.put("id", fromU8Array(tag.id))
            // There's also ndef.type which returns the ndef spec type which may be interesting to know too?
            ret.put("kind", JSArray.from(tag.techList))
        }
        ret.put("records", JSArray.from(jsonRecords))

        invokeHandler?.resolve(ret)
    }

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
                this.pendingNfcAction = NfcAction.NONE
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

    // TODO: Use ReaderMode instead of ForegroundDispatch
    private fun enableNFCInForeground() {
        val flag =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE else PendingIntent.FLAG_UPDATE_CURRENT
        val pendingIntent = PendingIntent.getActivity(
            activity, 0,
            Intent(
                activity,
                activity.javaClass
            ).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
            flag
        )

        // For some reason this is ignored
        val nfcIntentFilter = IntentFilter(NfcAdapter.ACTION_NDEF_DISCOVERED)
        val filters = arrayOf(nfcIntentFilter)

        // Probably also ignored, if not it should include more than just these 2 once we don't just read simple ndefs.
        val techLists =
            arrayOf(arrayOf(Ndef::class.java.name), arrayOf(NdefFormatable::class.java.name))

        nfcAdapter?.enableForegroundDispatch(activity, pendingIntent, filters, techLists)
    }

    private fun disableNFCInForeground() {
        nfcAdapter?.disableForegroundDispatch(activity)
    }
}

private fun toU8Array(jsonArray: JSONArray): ByteArray {
    return ByteArray(jsonArray.length()) { i -> jsonArray.getInt(i).toByte() }
}

private fun fromU8Array(byteArray: ByteArray): JSONArray {
    val json = JSONArray()
    for (byte in byteArray) {
        json.put(byte)
    }
    return json
}

private fun recordToJson(record: NdefRecord): JSObject {
    val json = JSObject()
    json.put("tnf", record.tnf)
    json.put("kind", fromU8Array(record.type))
    json.put("id", fromU8Array(record.id))
    json.put("payload", fromU8Array(record.payload))
    return json
}