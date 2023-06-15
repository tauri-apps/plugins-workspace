// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
