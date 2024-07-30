/** user-defined commands **/
export declare const commands: {
    getCurrentPosition(options: PositionOptions | null): Promise<Result<Position, Error>>;
    watchPosition(options: PositionOptions, channel: any): Promise<Result<null, Error>>;
    clearWatch(channelId: number): Promise<Result<null, Error>>;
    checkPermissions(): Promise<Result<PermissionStatus, Error>>;
    requestPermissions(permissions: PermissionType[] | null): Promise<Result<PermissionStatus, Error>>;
};
/** user-defined events **/
/** user-defined statics **/
/** user-defined types **/
export type Coordinates = {
    /**
     * Latitude in decimal degrees.
     */
    latitude: number;
    /**
     * Longitude in decimal degrees.
     */
    longitude: number;
    /**
     * Accuracy level of the latitude and longitude coordinates in meters.
     */
    accuracy: number;
    /**
     * Accuracy level of the altitude coordinate in meters, if available.
     * Available on all iOS versions and on Android 8 and above.
     */
    altitudeAccuracy: number | null;
    /**
     * The altitude the user is at, if available.
     */
    altitude: number | null;
    speed: number | null;
    /**
     * The heading the user is facing, if available.
     */
    heading: number | null;
};
export type Error = never;
/**
 * Permission state.
 */
export type PermissionState = 
/**
 * Permission access has been granted.
 */
"granted"
/**
 * Permission access has been denied.
 */
 | "denied"
/**
 * The end user should be prompted for permission.
 */
 | "prompt";
export type PermissionStatus = {
    /**
     * Permission state for the location alias.
     *
     * On Android it requests/checks both ACCESS_COARSE_LOCATION and ACCESS_FINE_LOCATION permissions.
     *
     * On iOS it requests/checks location permissions.
     */
    location: PermissionState;
    /**
     * Permissions state for the coarseLoaction alias.
     *
     * On Android it requests/checks ACCESS_COARSE_LOCATION.
     *
     * On Android 12+, users can choose between Approximate location (ACCESS_COARSE_LOCATION) and Precise location (ACCESS_FINE_LOCATION).
     *
     * On iOS it will have the same value as the `location` alias.
     */
    coarseLocation: PermissionState;
};
export type PermissionType = "location" | "coarseLocation";
export type Position = {
    /**
     * Creation time for these coordinates.
     */
    timestamp: number;
    /**
     * The GPD coordinates along with the accuracy of the data.
     */
    coords: Coordinates;
};
export type PositionOptions = {
    /**
     * High accuracy mode (such as GPS, if available)
     * Will be ignored on Android 12+ if users didn't grant the ACCESS_FINE_LOCATION permission.
     */
    enableHighAccuracy: boolean;
    /**
     * The maximum wait time in milliseconds for location updates.
     * On Android the timeout gets ignored for getCurrentPosition.
     * Ignored on iOS
     */
    timeout: number;
    /**
     * The maximum age in milliseconds of a possible cached position that is acceptable to return.
     * Default: 0
     * Ignored on iOS
     */
    maximumAge: number;
};
export type Result<T, E> = {
    status: "ok";
    data: T;
} | {
    status: "error";
    error: E;
};
