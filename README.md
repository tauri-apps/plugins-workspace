# Official Tauri Plugins

This repo and all plugins require a Rust version of at least **1.77.2**

## Plugins Found Here

|                                                |                                                                                                                                                                | Win | Mac | Lin | iOS | And |
| ---------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | --- | --- | --- | --- |
| [autostart](plugins/autostart)                 | Automatically launch your app at system startup.                                                                                                               | ✅  | ✅  | ✅  | ❌  | ❌  |
| [barcode-scanner](plugins/barcode-scanner)     | Allows your mobile application to use the camera to scan QR codes, EAN-13 and other kinds of barcodes.                                                         | ?   | ?   | ?   | ✅  | ✅  |
| [biometric](plugins/biometric)                 | Prompt the user for biometric authentication on Android and iOS.                                                                                               | ?   | ?   | ?   | ✅  | ✅  |
| [cli](plugins/cli)                             | Parse arguments from your Command Line Interface                                                                                                               | ✅  | ✅  | ✅  | ❌  | ❌  |
| [clipboard-manager](plugins/clipboard-manager) | Read and write to the system clipboard.                                                                                                                        | ✅  | ✅  | ✅  | ✅  | ✅  |
| [deep-link](plugins/deep-link)                 | Set your Tauri application as the default handler for an URL.                                                                                                  | ✅  | ✅  | ✅  | ✅  | ✅  |
| [dialog](plugins/dialog)                       | Native system dialogs for opening and saving files along with message dialogs.                                                                                 | ✅  | ✅  | ✅  | ✅  | ✅  |
| [fs](plugins/fs)                               | Access the file system.                                                                                                                                        | ✅  | ✅  | ✅  | ?   | ?   |
| [geolocation](plugins/geolocation)             | Get and track current device position.                                                                                                                         | ?   | ?   | ?   | ✅  | ✅  |
| [global-shortcut](plugins/global-shortcut)     | Register global shortcuts.                                                                                                                                     | ✅  | ✅  | ✅  | ?   | ?   |
| [haptics](plugins/haptics)                     | Haptic feedback and vibrations.                                                                                                                                | ?   | ?   | ?   | ✅  | ✅  |
| [http](plugins/http)                           | Access the HTTP client written in Rust.                                                                                                                        | ✅  | ✅  | ✅  | ✅  | ✅  |
| [localhost](plugins/localhost)                 | Use a localhost server in production apps.                                                                                                                     | ✅  | ✅  | ✅  | ?   | ?   |
| [log](plugins/log)                             | Configurable logging.                                                                                                                                          | ✅  | ✅  | ✅  | ✅  | ✅  |
| [nfc](plugins/nfc)                             | Read and write NFC tags on Android and iOS.                                                                                                                    | ?   | ?   | ?   | ✅  | ✅  |
| [notification](plugins/notification)           | Send message notifications (brief auto-expiring OS window element) to your user. Can also be used with the Notification Web API.                               | ✅  | ✅  | ✅  | ✅  | ✅  |
| [opener](plugins/opener)                       | Open files and URLs using their default application.                                                                                                           | ✅  | ✅  | ✅  | ?   | ?   |
| [os](plugins/os)                               | Read information about the operating system.                                                                                                                   | ✅  | ✅  | ✅  | ✅  | ✅  |
| [persisted-scope](plugins/persisted-scope)     | Persist runtime scope changes on the filesystem.                                                                                                               | ✅  | ✅  | ✅  | ?   | ?   |
| [positioner](plugins/positioner)               | Move windows to common locations.                                                                                                                              | ✅  | ✅  | ✅  | ❌  | ❌  |
| [process](plugins/process)                     | This plugin provides APIs to access the current process. To spawn child processes, see the [`shell`](https://github.com/tauri-apps/tauri-plugin-shell) plugin. | ✅  | ✅  | ✅  | ?   | ?   |
| [shell](plugins/shell)                         | Access the system shell. Allows you to spawn child processes and manage files and URLs using their default application.                                        | ✅  | ✅  | ✅  | ?   | ?   |
| [single-instance](plugins/single-instance)     | Ensure a single instance of your tauri app is running.                                                                                                         | ✅  | ✅  | ✅  | ❌  | ❌  |
| [sql](plugins/sql)                             | Interface with SQL databases.                                                                                                                                  | ✅  | ✅  | ✅  | ✅  | ✅  |
| [store](plugins/store)                         | Persistent key value storage.                                                                                                                                  | ✅  | ✅  | ✅  | ✅  | ✅  |
| [stronghold](plugins/stronghold)               | Encrypted, secure database.                                                                                                                                    | ✅  | ✅  | ✅  | ?   | ?   |
| [updater](plugins/updater)                     | In-app updates for Tauri applications.                                                                                                                         | ✅  | ✅  | ✅  | ❌  | ❌  |
| [upload](plugins/upload)                       | Tauri plugin for file uploads through HTTP.                                                                                                                    | ✅  | ✅  | ✅  | ?   | ?   |
| [websocket](plugins/websocket)                 | Open a WebSocket connection using a Rust client in JS.                                                                                                         | ✅  | ✅  | ✅  | ?   | ?   |
| [window-state](plugins/window-state)           | Persist window sizes and positions.                                                                                                                            | ✅  | ✅  | ✅  | ❌  | ❌  |

- ✅: (Partially) Supported
- ❌: Not supported
- `?` : Unknown/Untested or Planned

## Contributing

PRs accepted. Please make sure to read the [Contributing Guide](https://github.com/tauri-apps/tauri/blob/dev/.github/CONTRIBUTING.md) before making a pull request.

## Partners

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src=".github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).
