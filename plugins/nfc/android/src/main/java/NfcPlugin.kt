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
import android.os.PatternMatcher
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import com.fasterxml.jackson.annotation.JsonValue
import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.databind.DeserializationContext
import com.fasterxml.jackson.databind.JsonDeserializer
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.annotation.JsonDeserialize
import org.json.JSONArray
import java.io.IOException
import kotlin.concurrent.thread

sealed class NfcAction {
    object Read : NfcAction()
    data class Write(val message: NdefMessage) : NfcAction()
}

@InvokeArg
class UriFilter {
    var scheme: String? = null
    var host: String? = null
    var pathPrefix: String? = null
}

@InvokeArg
enum class TechKind(@JsonValue val value: String) {
    IsoDep("IsoDep"),
    MifareClassic("MifareClassic"),
    MifareUltralight("MifareUltralight"),
    Ndef("Ndef"),
    NdefFormatable("NdefFormatable"),
    NfcA("NfcA"),
    NfcB("NfcB"),
    NfcBarcode("NfcBarcode"),
    NfcF("NfcF"),
    NfcV("NfcV");

    fun className(): String {
        return when (this) {
            IsoDep -> {
                android.nfc.tech.IsoDep::class.java.name
            }
            MifareClassic -> {
                android.nfc.tech.MifareClassic::class.java.name
            }
            MifareUltralight -> {
                android.nfc.tech.MifareUltralight::class.java.name
            }
            Ndef -> {
                android.nfc.tech.Ndef::class.java.name
            }
            NdefFormatable -> {
                android.nfc.tech.NdefFormatable::class.java.name
            }
            NfcA -> {
                android.nfc.tech.NfcA::class.java.name
            }
            NfcB -> {
                android.nfc.tech.NfcB::class.java.name
            }
            NfcBarcode -> {
                android.nfc.tech.NfcBarcode::class.java.name
            }
            NfcF -> {
                android.nfc.tech.NfcF::class.java.name
            }
            NfcV -> {
                android.nfc.tech.NfcV::class.java.name
            }
        }
    }
}

private fun addDataFilters(intentFilter: IntentFilter, uri: UriFilter?, mimeType: String?) {
    uri?.let { it -> {
        it.scheme?.let {
            intentFilter.addDataScheme(it)
        }
        it.host?.let {
            intentFilter.addDataAuthority(it, null)
        }
        it.pathPrefix?.let {
            intentFilter.addDataPath(it, PatternMatcher.PATTERN_PREFIX)
        }
    }}
    mimeType?.let {
        intentFilter.addDataType(it)
    }
}

@InvokeArg
@JsonDeserialize(using = ScanKindDeserializer::class)
sealed class ScanKind {
    @JsonDeserialize
    class Tag: ScanKind() {
        var mimeType: String? = null
        var uri: UriFilter? = null
    }
    @JsonDeserialize
    class Ndef: ScanKind() {
        var mimeType: String? = null
        var uri: UriFilter? = null
        var techLists: Array<Array<TechKind>>? = null
    }

    fun filters(): Array<IntentFilter>? {
        return when (this) {
            is Tag -> {
                val intentFilter = IntentFilter(NfcAdapter.ACTION_TAG_DISCOVERED)
                addDataFilters(intentFilter, uri, mimeType)
                arrayOf(intentFilter)
            }
            is Ndef -> {
                val intentFilter = IntentFilter(if (techLists == null) NfcAdapter.ACTION_NDEF_DISCOVERED else NfcAdapter.ACTION_TECH_DISCOVERED)
                addDataFilters(intentFilter, uri, mimeType)
                arrayOf(intentFilter)
            }
        }
    }

    fun techLists(): Array<Array<String>>? {
        return when (this) {
            is Tag -> null
            is Ndef -> {
                techLists?.let {
                    val techs = mutableListOf<Array<String>>()
                    for (techList in it) {
                        val list = mutableListOf<String>()
                        for (tech in techList) {
                            list.add(tech.className())
                        }
                        techs.add(list.toTypedArray())
                    }
                    techs.toTypedArray()
                } ?: run {
                    null
                }
            }
        }
    }
}

internal class ScanKindDeserializer: JsonDeserializer<ScanKind>() {
    override fun deserialize(
        jsonParser: JsonParser,
        deserializationContext: DeserializationContext
    ): ScanKind {
        val node: JsonNode = jsonParser.codec.readTree(jsonParser)
        node.get("tag")?.let {
            return jsonParser.codec.treeToValue(it, ScanKind.Tag::class.java)
        } ?: node.get("ndef")?.let {
            return jsonParser.codec.treeToValue(it, ScanKind.Ndef::class.java)
        } ?: run {
            throw Error("unknown scan kind $node")
        }
    }
}

@InvokeArg
class ScanOptions {
    lateinit var kind: ScanKind
    var keepSessionAlive: Boolean = false
}

@InvokeArg
class NDEFRecordData {
    var format: Short = 0
    var kind: ByteArray = ByteArray(0)
    var id: ByteArray = ByteArray(0)
    var payload: ByteArray = ByteArray(0)
}

@InvokeArg
class WriteOptions {
    var kind: ScanKind? = null
    lateinit var records: Array<NDEFRecordData>
}

class Session(
    val action: NfcAction,
    val invoke: Invoke,
    val keepAlive: Boolean,
    var tag: Tag? = null,
    val filters: Array<IntentFilter>? = null,
    val techLists: Array<Array<String>>? = null
)

@TauriPlugin
class NfcPlugin(private val activity: Activity) : Plugin(activity) {
    private lateinit var webView: WebView

    private var nfcAdapter: NfcAdapter? = null
    private var session: Session? = null

    override fun load(webView: WebView) {
        super.load(webView)
        this.webView = webView
        this.nfcAdapter = NfcAdapter.getDefaultAdapter(activity.applicationContext)
    }

    override fun onNewIntent(intent: Intent) {
        Logger.info("NFC", "onNewIntent")
        super.onNewIntent(intent)

        val extraTag = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            intent.getParcelableExtra(NfcAdapter.EXTRA_TAG, Tag::class.java)
        } else {
            @Suppress("DEPRECATION")
            intent.getParcelableExtra(NfcAdapter.EXTRA_TAG)
        }

        extraTag?.let { tag ->
            session?.let {
                if (it.keepAlive) {
                    it.tag = tag
                }
            }

            when (session?.action) {
                is NfcAction.Read -> readTag(tag, intent)
                is NfcAction.Write -> thread {
                    if (session?.action is NfcAction.Write) {
                        try {
                            writeTag(tag, (session?.action as NfcAction.Write).message)
                            session?.invoke?.resolve()
                        } catch (e: Exception) {
                            session?.invoke?.reject(e.toString())
                        } finally {
                            if (this.session?.keepAlive != true) {
                                this.session = null
                                disableNFCInForeground()
                            }
                        }
                    }
                }

                else -> {}
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
        session?.let {
            enableNFCInForeground(it.filters, it.techLists)
        }
    }

    private fun isAvailable(): Pair<Boolean, String?> {
        val available: Boolean
        var errorReason: String? = null

        if (this.nfcAdapter === null) {
            available = false
            errorReason = "Device does not have NFC capabilities"
        } else if (this.nfcAdapter?.isEnabled == false) {
            available = false
            errorReason = "NFC is disabled in device settings"
        } else {
            available = true
        }

        return Pair(available, errorReason)
    }

    @Command
    fun isAvailable(invoke: Invoke) {
        val ret = JSObject()
        ret.put("available", isAvailable().first)
        invoke.resolve(ret)
    }

    @Command
    fun scan(invoke: Invoke) {
        val status = isAvailable()
        if (!status.first) {
            invoke.reject("NFC unavailable: " + status.second)
            return
        }

        val args = invoke.parseArgs(ScanOptions::class.java)

        val filters = args.kind.filters()
        val techLists = args.kind.techLists()
        enableNFCInForeground(filters, techLists)

        session = Session(NfcAction.Read, invoke, args.keepSessionAlive, null, filters, techLists)
    }

    @Command
    fun write(invoke: Invoke) {
        val status = isAvailable()
        if (!status.first) {
            invoke.reject("NFC unavailable: " + status.second)
            return
        }

        val args = invoke.parseArgs(WriteOptions::class.java)

        val ndefRecords: MutableList<NdefRecord> = ArrayList()
        for (record in args.records) {
            ndefRecords.add(NdefRecord(record.format, record.kind, record.id, record.payload))
        }

        val message = NdefMessage(ndefRecords.toTypedArray())

        session?.let { session ->
            session.tag?.let {
                try {
                    writeTag(it, message)
                    invoke.resolve()
                } catch (e: Exception) {
                    invoke.reject(e.toString())
                } finally {
                    if (this.session?.keepAlive != true) {
                        this.session = null
                        disableNFCInForeground()
                    }
                }
            } ?: run {
                invoke.reject("connected tag not found, please wait for it to be available and then call write()")
            }
        } ?: run {
            args.kind?.let { kind -> {
                val filters = kind.filters()
                val techLists = kind.techLists()
                enableNFCInForeground(filters, techLists)
                session = Session(NfcAction.Write(message), invoke, true, null, filters, techLists)
                Logger.warn("NFC", "Write Mode Enabled")
            }} ?: run {
                invoke.reject("Missing `kind` for write")
            }

        }
    }

    private fun readTag(tag: Tag, intent: Intent) {
        try {
            val rawMessages = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                intent.getParcelableArrayExtra(NfcAdapter.EXTRA_NDEF_MESSAGES, Parcelable::class.java)
            } else {
                @Suppress("DEPRECATION")
                intent.getParcelableArrayExtra(NfcAdapter.EXTRA_NDEF_MESSAGES)
            }

            when (intent.action) {
                NfcAdapter.ACTION_NDEF_DISCOVERED -> {
                    // For some reason this one never triggers.
                    Logger.info("NFC", "new NDEF intent")
                    readTagInner(tag, rawMessages)
                }
                NfcAdapter.ACTION_TECH_DISCOVERED -> {
                    // For some reason this always triggers instead of NDEF_DISCOVERED even though we set ndef filters right now
                    Logger.info("NFC", "new TECH intent")
                    // TODO: handle different techs. Don't assume ndef.
                    readTagInner(tag, rawMessages)
                }
                NfcAdapter.ACTION_TAG_DISCOVERED -> {
                    // This should never trigger when an app handles NDEF and TECH
                    // TODO: Don't assume ndef.
                    readTagInner(tag, rawMessages)
                }
            }
        } catch (e: Exception) {
            session?.invoke?.reject("failed to read tag", e)
        } finally {
            if (this.session?.keepAlive != true) {
                this.session = null
            }
            // TODO this crashes? disableNFCInForeground()
        }
    }

    private fun readTagInner(tag: Tag?, rawMessages: Array<Parcelable>?) {
        val ndefMessage = rawMessages?.get(0) as NdefMessage?

        val records = ndefMessage?.records ?: arrayOf()

        val jsonRecords = Array(records.size) { i -> recordToJson(records[i]) }

        val ret = JSObject()
        if (tag !== null) {
            ret.put("id", fromU8Array(tag.id))
            // TODO There's also ndef.type which returns the ndef spec type which may be interesting to know too?
            ret.put("kind", JSArray.from(tag.techList))
        }
        ret.put("records", JSArray.from(jsonRecords))

        session?.invoke?.resolve(ret)
    }

    private fun writeTag(tag: Tag, message: NdefMessage) {
        // This should return tags that are already in ndef format
        val ndefTag = Ndef.get(tag)
        if (ndefTag !== null) {
            // We have to connect first to check maxSize.
            try {
                ndefTag.connect()
            } catch (e: IOException) {
                throw Exception("Couldn't connect to NFC tag", e)
            }

            if (ndefTag.maxSize < message.toByteArray().size) {
                throw Exception("The message is too large for the provided NFC tag")
            } else if (!ndefTag.isWritable) {
                throw Exception("NFC tag is read-only")
            } else {
                try {
                    ndefTag.writeNdefMessage(message)
                } catch (e: Exception) {
                    throw Exception("Couldn't write message to NFC tag", e)
                }
            }

            try {
                ndefTag.close()
            } catch (e: IOException) {
                Logger.error("failed to close tag", e)
            }

            return
        }

        // This should cover tags that are not yet in ndef format but can be converted
        val ndefFormatableTag = NdefFormatable.get(tag)
        if (ndefFormatableTag !== null) {
            try {
                ndefFormatableTag.connect()
                ndefFormatableTag.format(message)
            } catch (e: Exception) {
                throw Exception("Couldn't format tag as Ndef", e)
            }

            try {
                ndefFormatableTag.close()
            } catch (e: IOException) {
                Logger.error("failed to close tag", e)
            }

            return
        }

        // if we get to this line, the tag was neither Ndef nor NdefFormatable compatible
        throw Exception("Tag doesn't support Ndef format")
    }

    // TODO: Use ReaderMode instead of ForegroundDispatch
    private fun enableNFCInForeground(filters: Array<IntentFilter>?, techLists: Array<Array<String>>?) {
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

        nfcAdapter?.enableForegroundDispatch(activity, pendingIntent, filters, techLists)
    }

    private fun disableNFCInForeground() {
        activity.runOnUiThread {
            nfcAdapter?.disableForegroundDispatch(activity)
        }
    }
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