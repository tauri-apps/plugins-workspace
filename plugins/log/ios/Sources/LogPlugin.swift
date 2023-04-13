import UIKit
import Tauri
import SwiftRs

@_cdecl("tauri_log")
func log(level: Int, message: NSString) {
	switch level {
		case 1: Logger.debug(message as String)
		case 2: Logger.info(message as String)
		case 3: Logger.error(message as String)
		default: break
	}
}
