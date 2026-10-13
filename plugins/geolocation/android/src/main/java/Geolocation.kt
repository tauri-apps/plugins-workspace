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
import android.os.SystemClock
import androidx.core.content.ContextCompat
import androidx.core.location.LocationListenerCompat
import androidx.core.location.LocationManagerCompat
import androidx.core.location.LocationRequestCompat
import app.tauri.Logger
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import com.google.android.gms.location.FusedLocationProviderClient
import com.google.android.gms.location.LocationCallback
import com.google.android.gms.location.LocationRequest as GmsLocationRequest
import com.google.android.gms.location.LocationResult
import com.google.android.gms.location.LocationServices
import com.google.android.gms.location.Priority


public class Geolocation(private val context: Context) {
    private val locationManager = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
    private var fusedLocationClient: FusedLocationProviderClient? = null
    private var locationCallback: LocationCallback? = null // For gms
    private var locationListener: LocationListenerCompat? = null // For android

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

    // Picks the best enabled LocationManager provider for devices without Google Play Services.
    // Reports an error and returns null when there is none.
    private fun androidProvider(enableHighAccuracy: Boolean, errorCallback: (error: String) -> Unit): String? {
        if (!isLocationServicesEnabled()) {
            errorCallback("Location disabled.")
            return null
        }

        val fused = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) listOf(LocationManager.FUSED_PROVIDER) else emptyList()
        val candidates = if (enableHighAccuracy) {
            fused + LocationManager.GPS_PROVIDER + LocationManager.NETWORK_PROVIDER
        } else {
            fused + LocationManager.NETWORK_PROVIDER + LocationManager.GPS_PROVIDER
        }
        val enabled = locationManager.getProviders(true)
        val provider = candidates.firstOrNull { it in enabled }
        if (provider == null) {
            errorCallback("Location unavailable.")
        }
        return provider
    }

    @SuppressLint("MissingPermission")
    fun sendLocation(enableHighAccuracy: Boolean, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        if (isGmsAvailable()) {
            if (this.isLocationServicesEnabled()) {
                LocationServices
                    .getFusedLocationProviderClient(context)
                    .getCurrentLocation(gmsPriority(enableHighAccuracy), null)
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
        } else {
            val provider = androidProvider(enableHighAccuracy, errorCallback) ?: return
            val onResult = { location: Location? ->
                if (location == null) {
                    errorCallback("Location unavailable.")
                } else {
                    successCallback(location)
                }
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                // On API 31+ the provider is usually fused, so the request quality is what honors enableHighAccuracy.
                val request = LocationRequest.Builder(0)
                    .setQuality(if (enableHighAccuracy) LocationRequest.QUALITY_HIGH_ACCURACY else LocationRequest.QUALITY_BALANCED_POWER_ACCURACY)
                    .build()
                locationManager.getCurrentLocation(provider, request, null, context.mainExecutor) { onResult(it) }
            } else {
                LocationManagerCompat.getCurrentLocation(locationManager, provider, null, ContextCompat.getMainExecutor(context)) { onResult(it) }
            }
        }
    }

    @SuppressLint("MissingPermission")
    fun requestLocationUpdates(enableHighAccuracy: Boolean, timeout: Long, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        clearLocationUpdates()

        if (isGmsAvailable()) {
            fusedLocationClient = LocationServices.getFusedLocationProviderClient(context)

            if (this.isLocationServicesEnabled()) {
                val locationRequest = GmsLocationRequest.Builder(timeout)
                    .setMaxUpdateDelayMillis(timeout)
                    .setMinUpdateIntervalMillis(timeout)
                    .setPriority(gmsPriority(enableHighAccuracy))
                    .build()

                locationCallback =
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

                fusedLocationClient?.requestLocationUpdates(locationRequest, locationCallback!!, null)
            } else {
                errorCallback("Location disabled.")
            }
        } else {
            val provider = androidProvider(enableHighAccuracy, errorCallback) ?: return
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
            locationListener = listener
            LocationManagerCompat.requestLocationUpdates(locationManager, provider, request, ContextCompat.getMainExecutor(context), listener)
        }
    }

    fun clearLocationUpdates() {
        if (locationCallback != null) {
            fusedLocationClient?.removeLocationUpdates(locationCallback!!)
            locationCallback = null
        }
        if (locationListener != null) {
            LocationManagerCompat.removeUpdates(locationManager, locationListener!!)
            locationListener = null
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
