import UIKit
import Tauri
import SwiftRs

@_cdecl("tauri_log")
func log(level: Int, message: SRString) {
	switch level {
		case 1: Logger.debug(message.toString())
		case 2: Logger.info(message.toString())
		case 3: Logger.error(message.toString())
		default: break
	}
}
