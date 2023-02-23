import UIKit
import Tauri
import SwiftRs

@_cdecl("tauri_log")
func log(level: Int, message: UnsafePointer<SRString>) {
	switch level {
		case 1: Logger.debug(message.pointee.to_string())
		case 2: Logger.info(message.pointee.to_string())
		case 3: Logger.error(message.pointee.to_string())
		default: break
	}
}
