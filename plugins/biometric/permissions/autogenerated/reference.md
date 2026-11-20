## Default Permission

This permission set configures which
biometric features are by default exposed.

#### Granted Permissions

It allows checking the biometric status and
prompting for authentication. Encrypting and
decrypting data with `biometric_cipher` is not
included and must be enabled explicitly with
`biometric:allow-biometric-cipher`.

#### This default permission set includes the following:

- `allow-authenticate`
- `allow-status`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`biometric:allow-authenticate`

</td>
<td>

Enables the authenticate command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`biometric:deny-authenticate`

</td>
<td>

Denies the authenticate command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`biometric:allow-biometric-cipher`

</td>
<td>

Enables the biometric_cipher command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`biometric:deny-biometric-cipher`

</td>
<td>

Denies the biometric_cipher command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`biometric:allow-status`

</td>
<td>

Enables the status command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`biometric:deny-status`

</td>
<td>

Denies the status command without any pre-configured scope.

</td>
</tr>
</table>
