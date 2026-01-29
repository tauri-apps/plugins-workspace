I believe this should be a MINOR increase in version number, but since my implementation would raise the MSRV to 1.80 (which is a 2 year old version at this point) it could be classified as a major increase as well.

First and most important change is that now by default the single instance plugin will use the app's reverse domain with `.SingleInstance` in the end as the DBus ID instead of `org.apps_reverse_id.SingleInstance` (e.g `apps.reverse.id.SingleInstance`). This will make the developer's lives easier as they won't have to get extra approvals from Flathub and Snap stores.

There is now an option to have an environment variable called `DBUS_ID` at compile time that will set the single instance plugin's DBus ID to it's value it has to be a valid DBus ID.

There is also a setter function (`set_dbus_id`), (implementation of which causes the MSRV to be 1.80, due to my usage of LazyLock) the setter function must be ran before the Tauri app initializes on Linux (Before `tauri::Builder` is run)