## Default Permission

This permission set configures what kind of
operations are available from the nfc plugin.

#### Granted Permissions

Checking if the NFC functionality is available
and scanning nearby tags is allowed.
Writing to tags needs to be manually enabled.



- `allow-is-available`
- `allow-scan`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`nfc:allow-is-available`

</td>
<td>

Enables the is_available command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`nfc:deny-is-available`

</td>
<td>

Denies the is_available command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`nfc:allow-scan`

</td>
<td>

Enables the scan command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`nfc:deny-scan`

</td>
<td>

Denies the scan command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`nfc:allow-write`

</td>
<td>

Enables the write command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`nfc:deny-write`

</td>
<td>

Denies the write command without any pre-configured scope.

</td>
</tr>
</table>
