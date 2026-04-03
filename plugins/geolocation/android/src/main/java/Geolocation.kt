// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.geolocation

import android.annotation.SuppressLint
import android.content.Context
import android.location.Location
import android.location.LocationManager
import android.os.SystemClock
import androidx.core.location.LocationManagerCompat
import app.tauri.Logger
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import com.google.android.gms.location.FusedLocationProviderClient
import com.google.android.gms.location.LocationCallback
import com.google.android.gms.location.LocationRequest as GmsLocationRequest
import com.google.android.gms.location.LocationResult
import com.google.android.gms.location.LocationServices
import com.google.android.gms.location.Priority
import android.location.LocationRequest
import android.location.LocationListener


public class Geolocation(private val context: Context) {
    private var fusedLocationClient: FusedLocationProviderClient? = null
    private var locationCallback: LocationCallback? = null // For gms
    private var locationListener: LocationListener? = null // For android

    fun isLocationServicesEnabled(): Boolean {
        val lm = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
        return LocationManagerCompat.isLocationEnabled(lm)
    }

    @SuppressWarnings("MissingPermission")
    fun sendLocation(enableHighAccuracy: Boolean, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        val resultCode = GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context);
        if (resultCode == ConnectionResult.SUCCESS) {
            val lm = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager

            if (this.isLocationServicesEnabled()) {
                var networkEnabled = false

                try {
                    networkEnabled = lm.isProviderEnabled(LocationManager.NETWORK_PROVIDER)
                } catch (_: Exception) {
                    Logger.error("isProviderEnabled failed")
                }

                val lowPrio = if (networkEnabled) Priority.PRIORITY_BALANCED_POWER_ACCURACY else Priority.PRIORITY_LOW_POWER
                val prio = if (enableHighAccuracy) Priority.PRIORITY_HIGH_ACCURACY else lowPrio

                LocationServices
                    .getFusedLocationProviderClient(context)
                    .getCurrentLocation(prio, null)
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
            val locationManager = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
            val provider = locationManager.getProviderProperties(LocationManager.GPS_PROVIDER)
            if (provider == null) {
                errorCallback("Location unavailable.")
                return
            }
            if (!locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)) {
                errorCallback("Location disabled.")
                return
            }
            val req = LocationRequest.Builder(1_000L)
                .setQuality(LocationRequest.QUALITY_HIGH_ACCURACY)
                .setMaxUpdates(1)
                .build()
            locationManager.getCurrentLocation(LocationManager.GPS_PROVIDER, req, null, context.mainExecutor) { location ->
                if (location == null) {
                    errorCallback("Location unavailable.")
                } else {
                    successCallback(location)
                }
            }
        }
    }

    @SuppressLint("MissingPermission")
    fun requestLocationUpdates(enableHighAccuracy: Boolean, timeout: Long, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        val resultCode = GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context);
        if (resultCode == ConnectionResult.SUCCESS) {
            clearLocationUpdates()
            fusedLocationClient = LocationServices.getFusedLocationProviderClient(context)

            val lm = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager

            if (this.isLocationServicesEnabled()) {
                var networkEnabled = false

                try {
                    networkEnabled = lm.isProviderEnabled(LocationManager.NETWORK_PROVIDER)
                } catch (_: Exception) {
                    Logger.error("isProviderEnabled failed")
                }

                val lowPrio = if (networkEnabled) Priority.PRIORITY_BALANCED_POWER_ACCURACY else Priority.PRIORITY_LOW_POWER
                val prio = if (enableHighAccuracy) Priority.PRIORITY_HIGH_ACCURACY else lowPrio

                val locationRequest = GmsLocationRequest.Builder(timeout)
                    .setMaxUpdateDelayMillis(timeout)
                    .setMinUpdateIntervalMillis(timeout)
                    .setPriority(prio)
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
            val locationManager = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
            val provider = locationManager.getProviderProperties(LocationManager.GPS_PROVIDER)
            if (provider == null) {
                errorCallback("Location unavailable.")
                return
            }
            if (!locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)) {
                errorCallback("Location disabled.")
                return
            }
            val req = LocationRequest.Builder(timeout)
                .setQuality(if (enableHighAccuracy) LocationRequest.QUALITY_HIGH_ACCURACY else LocationRequest.QUALITY_LOW_POWER)
                .build()
            val listener = object : android.location.LocationListener {
                override fun onLocationChanged(location: android.location.Location) {
                    successCallback(location)
                }
            }
            locationListener = listener
            locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER, req, context.mainExecutor, listener)
        }
    }

    fun clearLocationUpdates() {
        if (locationCallback != null) {
            fusedLocationClient?.removeLocationUpdates(locationCallback!!)
            locationCallback = null
        }
        if (locationListener != null) {
            val locationManager = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
            locationManager.removeUpdates(locationListener!!)
            locationListener = null
        }
    }

    @SuppressLint("MissingPermission")
    fun getLastLocation(maximumAge: Long): Location? {
        var lastLoc: Location? = null
        val lm = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager

        for (provider in lm.allProviders) {
            val tmpLoc = lm.getLastKnownLocation(provider)
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
