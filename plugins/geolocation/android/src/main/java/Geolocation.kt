// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.geolocation

import android.annotation.SuppressLint
import android.content.Context
import android.location.Location
import android.location.LocationManager
import android.location.LocationRequest
import android.os.Build
import android.os.CancellationSignal
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import androidx.core.content.ContextCompat
import androidx.core.location.LocationListenerCompat
import androidx.core.location.LocationManagerCompat
import androidx.core.location.LocationRequestCompat
import androidx.core.os.CancellationSignal as CompatCancellationSignal
import app.tauri.Logger
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import com.google.android.gms.location.CurrentLocationRequest
import com.google.android.gms.location.FusedLocationProviderClient
import com.google.android.gms.location.LocationCallback
import com.google.android.gms.location.LocationRequest as GmsLocationRequest
import com.google.android.gms.location.LocationResult
import com.google.android.gms.location.LocationServices
import com.google.android.gms.location.Priority


public class Geolocation(private val context: Context) {
    private val locationManager = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
    private var fusedLocationClient: FusedLocationProviderClient? = null
    // Active watches, keyed by watch id.
    private val locationCallbacks = HashMap<Long, LocationCallback>() // For gms
    private val locationListeners = HashMap<Long, LocationListenerCompat>() // For android

    fun isLocationServicesEnabled(): Boolean {
        return LocationManagerCompat.isLocationEnabled(locationManager)
    }

    private fun isGmsAvailable(): Boolean {
        return GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context) == ConnectionResult.SUCCESS
    }

    private fun gmsPriority(enableHighAccuracy: Boolean): Int {
        if (enableHighAccuracy) {
            return Priority.PRIORITY_HIGH_ACCURACY
        }

        var networkEnabled = false
        try {
            networkEnabled = locationManager.isProviderEnabled(LocationManager.NETWORK_PROVIDER)
        } catch (_: Exception) {
            Logger.error("isProviderEnabled failed")
        }

        return if (networkEnabled) Priority.PRIORITY_BALANCED_POWER_ACCURACY else Priority.PRIORITY_LOW_POWER
    }

    // Lists the enabled LocationManager providers for devices without Google Play Services, best first.
    private fun androidProviders(enableHighAccuracy: Boolean): List<String> {
        val fused = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) listOf(LocationManager.FUSED_PROVIDER) else emptyList()
        val candidates = if (enableHighAccuracy) {
            fused + LocationManager.GPS_PROVIDER + LocationManager.NETWORK_PROVIDER
        } else {
            fused + LocationManager.NETWORK_PROVIDER + LocationManager.GPS_PROVIDER
        }
        val enabled = locationManager.getProviders(true)
        return candidates.filter { it in enabled }
    }

    // Asks each provider in turn for a fresh fix, so one that is enabled but can't produce a fix
    // (e.g. network without Wi-Fi or cell signal) falls back to the next.
    // The timeout covers all attempts; 0 or less means no timeout.
    @SuppressLint("MissingPermission")
    private fun androidCurrentLocation(providers: List<String>, enableHighAccuracy: Boolean, timeout: Long, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        // Everything below runs on the main thread, so no synchronization is needed.
        val handler = Handler(Looper.getMainLooper())
        var done = false
        var cancelAttempt: (() -> Unit)? = null
        val onTimeout = Runnable {
            if (!done) {
                done = true
                cancelAttempt?.invoke()
                errorCallback("Timeout expired.")
            }
        }
        if (timeout > 0) {
            handler.postDelayed(onTimeout, timeout)
        }

        fun attempt(remaining: List<String>) {
            val provider = remaining.firstOrNull()
            if (provider == null) {
                done = true
                handler.removeCallbacks(onTimeout)
                errorCallback("Location unavailable.")
                return
            }
            val onResult = { location: Location? ->
                if (!done) {
                    if (location == null) {
                        attempt(remaining.drop(1))
                    } else {
                        done = true
                        handler.removeCallbacks(onTimeout)
                        successCallback(location)
                    }
                }
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                // On API 31+ the provider is usually fused, so the request quality is what honors enableHighAccuracy.
                val request = LocationRequest.Builder(0)
                    .setQuality(if (enableHighAccuracy) LocationRequest.QUALITY_HIGH_ACCURACY else LocationRequest.QUALITY_BALANCED_POWER_ACCURACY)
                    .build()
                val signal = CancellationSignal()
                cancelAttempt = { signal.cancel() }
                locationManager.getCurrentLocation(provider, request, signal, context.mainExecutor) { onResult(it) }
            } else {
                val signal = CompatCancellationSignal()
                cancelAttempt = { signal.cancel() }
                LocationManagerCompat.getCurrentLocation(locationManager, provider, signal, ContextCompat.getMainExecutor(context)) { onResult(it) }
            }
        }

        attempt(providers)
    }

    @SuppressLint("MissingPermission")
    fun sendLocation(enableHighAccuracy: Boolean, timeout: Long, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        if (isGmsAvailable()) {
            if (this.isLocationServicesEnabled()) {
                val currentLocationRequest = CurrentLocationRequest.Builder()
                    .setPriority(gmsPriority(enableHighAccuracy))
                if (timeout > 0) {
                    currentLocationRequest.setDurationMillis(timeout)
                }
                LocationServices
                    .getFusedLocationProviderClient(context)
                    .getCurrentLocation(currentLocationRequest.build(), null)
                    .addOnFailureListener { e -> e.message?.let { errorCallback(it) } }
                    .addOnSuccessListener { location ->
                        if (location == null) {
                            errorCallback("Location unavailable.")
                        } else {
                            successCallback(location)
                        }
                    }
            } else {
                errorCallback("Location disabled.")
            }
        } else if (!this.isLocationServicesEnabled()) {
            errorCallback("Location disabled.")
        } else {
            // Reports "Location unavailable." itself when there is no provider.
            androidCurrentLocation(androidProviders(enableHighAccuracy), enableHighAccuracy, timeout, successCallback, errorCallback)
        }
    }

    @SuppressLint("MissingPermission")
    fun requestLocationUpdates(id: Long, enableHighAccuracy: Boolean, timeout: Long, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        clearLocationUpdates(id)

        if (isGmsAvailable()) {
            fusedLocationClient = LocationServices.getFusedLocationProviderClient(context)

            if (this.isLocationServicesEnabled()) {
                val locationRequest = GmsLocationRequest.Builder(timeout)
                    .setMaxUpdateDelayMillis(timeout)
                    .setMinUpdateIntervalMillis(timeout)
                    .setPriority(gmsPriority(enableHighAccuracy))
                    .build()

                val locationCallback =
                    object : LocationCallback() {
                        override fun onLocationResult(locationResult: LocationResult) {
                            val lastLocation = locationResult.lastLocation
                            if (lastLocation == null) {
                                errorCallback("Location unavailable.")
                            } else {
                                successCallback(lastLocation)
                            }
                        }
                    }

                locationCallbacks[id] = locationCallback
                fusedLocationClient?.requestLocationUpdates(locationRequest, locationCallback, null)
            } else {
                errorCallback("Location disabled.")
            }
        } else if (!this.isLocationServicesEnabled()) {
            errorCallback("Location disabled.")
        } else {
            val provider = androidProviders(enableHighAccuracy).firstOrNull()
            if (provider == null) {
                errorCallback("Location unavailable.")
                return
            }
            val request = LocationRequestCompat.Builder(timeout)
                .setQuality(if (enableHighAccuracy) LocationRequestCompat.QUALITY_HIGH_ACCURACY else LocationRequestCompat.QUALITY_LOW_POWER)
                .build()
            val listener = object : LocationListenerCompat {
                override fun onLocationChanged(location: Location) {
                    successCallback(location)
                }

                override fun onProviderDisabled(provider: String) {
                    errorCallback("Location disabled.")
                }
            }
            locationListeners[id] = listener
            LocationManagerCompat.requestLocationUpdates(locationManager, provider, request, ContextCompat.getMainExecutor(context), listener)
        }
    }

    fun clearLocationUpdates(id: Long) {
        locationCallbacks.remove(id)?.let { fusedLocationClient?.removeLocationUpdates(it) }
        locationListeners.remove(id)?.let { LocationManagerCompat.removeUpdates(locationManager, it) }
    }

    fun clearAllLocationUpdates() {
        for (id in locationCallbacks.keys + locationListeners.keys) {
            clearLocationUpdates(id)
        }
    }

    @SuppressLint("MissingPermission")
    fun getLastLocation(maximumAge: Long): Location? {
        var lastLoc: Location? = null

        for (provider in locationManager.allProviders) {
            val tmpLoc = locationManager.getLastKnownLocation(provider)
            if (tmpLoc != null) {
                val locationAge = SystemClock.elapsedRealtimeNanos() - tmpLoc.elapsedRealtimeNanos
                val maxAgeNano = maximumAge * 1000000L
                if (locationAge <= maxAgeNano && (lastLoc == null || lastLoc.elapsedRealtimeNanos > tmpLoc.elapsedRealtimeNanos)) {
                    lastLoc = tmpLoc
                }
            }
        }

        return lastLoc
    }
}
