// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::plugin::PermissionState;

/// The current permission state for the geolocation APIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    /// Permission state for the location alias.
    ///
    /// On Android it requests/checks both ACCESS_COARSE_LOCATION and ACCESS_FINE_LOCATION permissions.
    ///
    /// On iOS it requests/checks location permissions.
    pub location: PermissionState,
    /// Permissions state for the coarseLoaction alias.
    ///
    /// On Android it requests/checks ACCESS_COARSE_LOCATION.
    ///
    /// On Android 12+, users can choose between Approximate location (ACCESS_COARSE_LOCATION) and Precise location (ACCESS_FINE_LOCATION).
    ///
    /// On iOS it will have the same value as the `location` alias.
    pub coarse_location: PermissionState,
}

/// Options used to configure a [`get_current_position`](crate::Geolocation::get_current_position) or [`watch_position`](crate::Geolocation::watch_position) request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct PositionOptions {
    /// High accuracy mode (such as GPS, if available)
    /// Will be ignored on Android 12+ if users didn't grant the ACCESS_FINE_LOCATION permission.
    pub enable_high_accuracy: bool,
    /// The maximum wait time in milliseconds for location updates.
    /// Default: 10000
    /// On Android the timeout gets ignored for getCurrentPosition.
    /// Ignored on iOS.
    // TODO: Handle Infinity and default to it.
    // TODO: Should be u64+ but specta doesn't like that?
    pub timeout: u32,
    /// The maximum age in milliseconds of a possible cached position that is acceptable to return.
    /// Default: 0
    /// Ignored on iOS.
    // TODO: Handle Infinity.
    // TODO: Should be u64+ but specta doesn't like that?
    pub maximum_age: u32,
}

/// The individual permission aliases that can be requested with [`request_permissions`](crate::Geolocation::request_permissions).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum PermissionType {
    /// The `location` alias. On Android this maps to both `ACCESS_COARSE_LOCATION` and `ACCESS_FINE_LOCATION`. On iOS it maps to the standard location permission.
    Location,
    /// The `coarseLocation` alias. On Android this maps to `ACCESS_COARSE_LOCATION` only. On iOS it behaves the same as [`Location`](Self::Location).
    CoarseLocation,
}

/// The GPS coordinates of a [`Position`], along with the accuracy of each reading.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Coordinates {
    /// Latitude in decimal degrees.
    pub latitude: f64,
    /// Longitude in decimal degrees.
    pub longitude: f64,
    /// Accuracy level of the latitude and longitude coordinates in meters.
    pub accuracy: f64,
    /// Accuracy level of the altitude coordinate in meters, if available.
    /// Available on all iOS versions and on Android 8 and above.
    pub altitude_accuracy: Option<f64>,
    /// The altitude the user is at, if available.
    pub altitude: Option<f64>,
    /// The speed the user is traveling, in meters per second, if available.
    pub speed: Option<f64>,
    /// The heading the user is facing, if available.
    pub heading: Option<f64>,
}

/// A geolocation reading, as returned by [`get_current_position`](crate::Geolocation::get_current_position) and reported through [`WatchEvent::Position`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Creation time for these coordinates.
    // TODO: Check if we're actually losing precision.
    pub timestamp: u64,
    /// The GPS coordinates along with the accuracy of the data.
    pub coords: Coordinates,
}

/// A single update sent through the channel callback registered with [`watch_position`](crate::Geolocation::watch_position).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(untagged)]
pub enum WatchEvent {
    /// A new position was read successfully.
    Position(Position),
    /// The platform failed to read a position; the string is the platform-provided error message.
    Error(String),
}
