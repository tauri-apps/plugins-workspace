// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.geolocation

import android.Manifest
import android.app.Activity
import android.location.Location
import android.os.Build
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.PermissionState
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class PositionOptions {
  var enableHighAccuracy: Boolean = false
  var maximumAge: Long = 0
  var timeout: Long = 10000
}

@InvokeArg
class WatchArgs {
    var options: PositionOptions = PositionOptions()
    lateinit var channel: Channel
}

@InvokeArg
class ClearWatchArgs {
    var channelId: Long = 0
}

// TODO: App requires a reload after permissions were granted
// TODO: Check if above assumption is actually correct

// TODO: Plugin does not ask user to enable google location services (like gmaps does)

private const val ALIAS_LOCATION: String = "location"
private const val ALIAS_COARSE_LOCATION: String = "coarseLocation"

@TauriPlugin(
    permissions = [
        Permission(strings = [
            Manifest.permission.ACCESS_FINE_LOCATION,
            Manifest.permission.ACCESS_COARSE_LOCATION
        ],
            alias = ALIAS_LOCATION
        ),
        Permission(strings = [
            Manifest.permission.ACCESS_COARSE_LOCATION
        ],
            alias = ALIAS_COARSE_LOCATION
        )
    ]
)
class GeolocationPlugin(private val activity: Activity): Plugin(activity) {
    private lateinit var implementation: Geolocation// = Geolocation(activity.applicationContext)
    private var watchers = hashMapOf<Long, Invoke>()

    override fun load(webView: WebView) {
        super.load(webView)
        implementation = Geolocation(activity.applicationContext)
    }

    override fun onPause() {
        super.onPause()
        // Clear all location updates on pause to avoid possible background location calls
        implementation.clearLocationUpdates()
    }

    override fun onResume() {
        super.onResume()
        for (watcher in watchers.values) {
            startWatch(watcher)
        }
    }

    @Command
    override fun checkPermissions(invoke: Invoke) {
        if (implementation.isLocationServicesEnabled()) {
            super.checkPermissions(invoke)
        } else {
            invoke.reject("Location services are disabled.")
        }
    }

    @Command
    override fun requestPermissions(invoke: Invoke) {
        if (implementation.isLocationServicesEnabled()) {
            super.requestPermissions(invoke)
        } else {
            invoke.reject("Location services are disabled.")
        }
    }

    @Command
    fun getCurrentPosition(invoke: Invoke) {
        val args = invoke.parseArgs(PositionOptions::class.java)
        val alias = getAlias(args.enableHighAccuracy)

        if (getPermissionState(alias) != PermissionState.GRANTED) {
            Logger.error("NOT GRANTED");
            requestPermissionForAlias(alias, invoke, "getCurrentPositionCallback")
        } else {
            Logger.error("GRANTED");
            getPosition(invoke, args)
        }
    }

    @PermissionCallback
    private fun getCurrentPositionCallback(invoke: Invoke) {
        val args = invoke.parseArgs(PositionOptions::class.java)
        Logger.error("CURPOS CALLBACK")
        // TODO: capacitor only checks for coarse here
        if (getPermissionState(ALIAS_COARSE_LOCATION) == PermissionState.GRANTED) {
            Logger.error("CURPOS CALLBACK GRANTED")
            implementation.sendLocation(args.enableHighAccuracy,
                { location -> invoke.resolve(convertLocation(location)) },
                { error -> invoke.reject(error) })
        } else {
            Logger.error("CURPOS CALLBACK DENIED")
            invoke.reject("Location permission was denied.")
        }
    }

    @Command
    fun watchPosition(invoke: Invoke) {
        val args = invoke.parseArgs(WatchArgs::class.java)
        val alias = getAlias(args.options.enableHighAccuracy)

        if (getPermissionState(alias) != PermissionState.GRANTED) {
            requestPermissionForAlias(alias, invoke, "watchPositionCallback")
        } else {
            startWatch(invoke)
        }
    }

    @PermissionCallback
    private fun watchPositionCallback(invoke: Invoke) {
        Logger.error("WATCHPOS CALLBACK")
        // TODO: capacitor only checks for coarse here
        if (getPermissionState(ALIAS_COARSE_LOCATION) == PermissionState.GRANTED) {
            Logger.error("WATCHPOS CALLBACK GRANTED")
            startWatch(invoke)
        } else {
            Logger.error("WATCHPOS CALLBACK DENIED")
            invoke.reject("Location permissions was denied.")
        }
    }

    private fun startWatch(invoke: Invoke) {
        val args = invoke.parseArgs(WatchArgs::class.java)

        implementation.requestLocationUpdates(
            args.options.enableHighAccuracy,
            args.options.timeout,
            { location -> args.channel.send(convertLocation(location)) },
            { error -> args.channel.sendObject(error) })

        watchers[args.channel.id] = invoke
    }

    @Command
    fun clearWatch(invoke: Invoke) {
        val args = invoke.parseArgs(ClearWatchArgs::class.java)

        watchers.remove(args.channelId)

        if (watchers.isEmpty()) {
            implementation.clearLocationUpdates()
        }

        invoke.resolve()
    }

    private fun getPosition(invoke: Invoke, options: PositionOptions) {
        val location = implementation.getLastLocation(options.maximumAge)
        if (location != null) {
            Logger.error("getPosition location non-null")
            invoke.resolve(convertLocation(location))
        } else {
            Logger.error("getPosition location null")
            implementation.sendLocation(options.enableHighAccuracy,
                { loc -> invoke.resolve(convertLocation(loc)) },
                { error -> invoke.reject(error) })
        }
    }

    private fun convertLocation(location: Location): JSObject {
        val ret = JSObject()
        val coords = JSObject()

        coords.put("latitude", location.latitude)
        coords.put("longitude", location.longitude)
        coords.put("accuracy", location.accuracy)
        coords.put("altitude", location.altitude)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            coords.put("altitudeAccuracy", location.verticalAccuracyMeters)
        }
        coords.put("speed", location.speed)
        coords.put("heading", location.bearing)
        ret.put("timestamp", location.time)
        ret.put("coords", coords)

        return ret
    }

    private fun getAlias(enableHighAccuracy: Boolean): String {
        var alias = ALIAS_LOCATION;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            if (!enableHighAccuracy) {
                alias = ALIAS_COARSE_LOCATION;
            }
        }
        return alias
    }
}
