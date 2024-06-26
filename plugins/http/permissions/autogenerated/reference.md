| Permission | Description |
|------|-----|
|`allow-fetch`|Enables the fetch command without any pre-configured scope.|
|`deny-fetch`|Denies the fetch command without any pre-configured scope.|
|`allow-fetch-cancel`|Enables the fetch_cancel command without any pre-configured scope.|
|`deny-fetch-cancel`|Denies the fetch_cancel command without any pre-configured scope.|
|`allow-fetch-read-body`|Enables the fetch_read_body command without any pre-configured scope.|
|`deny-fetch-read-body`|Denies the fetch_read_body command without any pre-configured scope.|
|`allow-fetch-send`|Enables the fetch_send command without any pre-configured scope.|
|`deny-fetch-send`|Denies the fetch_send command without any pre-configured scope.|
|`default`|This permission set configures what kind of
fetch operations are available from the http plugin.

This enables all fetch operations but does not
allow explicitly any origins to be fetched. This needs to
be manually configured before usage.

#### Granted Permissions

All fetch operations are enabled.

|
