// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import os.log

#if targetEnvironment(simulator)
  var logReady = false
#else
  var logReady = true
#endif

var pendingLogs: [(Int, NSString)] = []
var elapsedTime: TimeInterval = 0
var logFlushScheduled = false

@_cdecl("tauri_log")
func log(level: Int, message: NSString) {
  if logReady {
    os_log(level, message)
  } else {
    pendingLogs.append((level, message))
    scheduleLogFlush()
  }
}

// delay logging when the logger isn't immediately available
// in some cases when using the simulator the app would hang when calling os_log too soon
// better be safe here and wait a few seconds than actually freeze the app in dev mode
// in production this isn't a problem
func scheduleLogFlush() {
  guard !logFlushScheduled else { return }
  logFlushScheduled = true

  DispatchQueue.main.asyncAfter(deadline: .now() + 10) {
    flushLogs()
  }
}

func flushLogs() {
  for (level, message) in pendingLogs {
    os_log(level, message)
  }
  pendingLogs.removeAll()
}

func os_log(_ level: Int, _ message: NSString) {
  os_log("%{public}@", log: OSLog.default, type: osLogType(from: level), message)
}

func osLogType(from level: Int) -> OSLogType {
  switch level {
  case 1: return .debug
  case 2: return .info
  case 3: return .error
  default: return .default
  }
}
