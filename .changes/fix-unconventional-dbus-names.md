I believe this should be a MINOR increase in version number.

First and most important change is that now by default the single instance plugin will use the app's reverse domain with `.SingleInstance` in the end as the DBus ID instead of `org.apps_reverse_id.SingleInstance` (e.g `apps.reverse.id.SingleInstance`). This will make the developer's lives easier as they won't have to get extra approvals from Flathub and Snap stores.

There is now a builder on Linux, letting users set a custom DBus ID