// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import LocalAuthentication
import SwiftRs
import Tauri
import UIKit
import WebKit

class BiometricStatus {
  let available: Bool
  let biometryType: LABiometryType
  let errorReason: String?
  let errorCode: String?

  init(available: Bool, biometryType: LABiometryType, errorReason: String?, errorCode: String?) {
    self.available = available
    self.biometryType = biometryType
    self.errorReason = errorReason
    self.errorCode = errorCode
  }
}

class BiometricPlugin: Plugin {
  let authenticationErrorCodeMap: [Int: String] = [
    0: "",
    LAError.appCancel.rawValue: "appCancel",
    LAError.authenticationFailed.rawValue: "authenticationFailed",
    LAError.invalidContext.rawValue: "invalidContext",
    LAError.notInteractive.rawValue: "notInteractive",
    LAError.passcodeNotSet.rawValue: "passcodeNotSet",
    LAError.systemCancel.rawValue: "systemCancel",
    LAError.userCancel.rawValue: "userCancel",
    LAError.userFallback.rawValue: "userFallback",
    LAError.biometryLockout.rawValue: "biometryLockout",
    LAError.biometryNotAvailable.rawValue: "biometryNotAvailable",
    LAError.biometryNotEnrolled.rawValue: "biometryNotEnrolled",
  ]

  var status: BiometricStatus!

  public override func load(webview: WKWebView) {
    let context = LAContext()
    var error: NSError?
    var available = context.canEvaluatePolicy(
      .deviceOwnerAuthenticationWithBiometrics, error: &error)
    var reason: String? = nil
    var errorCode: String? = nil

    if available && context.biometryType == .faceID {
      let entry = Bundle.main.infoDictionary?["NSFaceIDUsageDescription"] as? String

      if entry == nil {
        available = false
        reason = "NSFaceIDUsageDescription is not in the app Info.plist"
        errorCode = authenticationErrorCodeMap[LAError.biometryNotAvailable.rawValue] ?? ""
      }
    } else if !available, let error = error {
      reason = error.localizedDescription
      if let failureReason = error.localizedFailureReason {
        reason = "\(reason ?? ""): \(failureReason)"
      }
      errorCode =
        authenticationErrorCodeMap[error.code] ?? authenticationErrorCodeMap[
          LAError.biometryNotAvailable.rawValue] ?? ""
    }

    self.status = BiometricStatus(
      available: available,
      biometryType: context.biometryType,
      errorReason: reason,
      errorCode: errorCode
    )
  }

  @objc func status(_ invoke: Invoke) {
    if self.status.available {
      invoke.resolve([
        "isAvailable": self.status.available,
        "biometryType": self.status.biometryType.rawValue,
      ])
    } else {
      invoke.resolve([
        "isAvailable": self.status.available,
        "biometryType": self.status.biometryType.rawValue,
        "reason": self.status.errorReason ?? "",
        "code": self.status.errorCode ?? "",
      ])
    }
  }

  @objc func authenticate(_ invoke: Invoke) {
    guard self.status.available else {
      invoke.reject(
        self.status.errorReason ?? "",
        self.status.errorCode ?? ""
      )
      return
    }

    guard let reason = invoke.getString("reason"), !reason.isEmpty else {
      invoke.reject("`reason` is required")
      return
    }

    let context = LAContext()
    context.localizedFallbackTitle = invoke.getString("fallbackTitle")
    context.localizedCancelTitle = invoke.getString("cancelTitle")
    context.touchIDAuthenticationAllowableReuseDuration = 0

    let allowDeviceCredential = invoke.getBool("allowDeviceCredential") ?? false

    // force system default fallback title if an empty string is provided (the OS hides the fallback button in this case)
    if allowDeviceCredential,
      let fallbackTitle = context.localizedFallbackTitle,
      fallbackTitle.isEmpty
    {
      context.localizedFallbackTitle = nil
    }

    context.evaluatePolicy(
      allowDeviceCredential ? .deviceOwnerAuthentication : .deviceOwnerAuthenticationWithBiometrics,
      localizedReason: reason
    ) { success, error in
      if success {
        invoke.resolve()
      } else {
        if let policyError = error as? LAError {
          let code = self.authenticationErrorCodeMap[policyError.code.rawValue]
          invoke.reject(policyError.localizedDescription, code)
        } else {
          invoke.reject(
            "Unknown error",
            self.authenticationErrorCodeMap[LAError.authenticationFailed.rawValue]
          )
        }
      }
    }

  }
}

@_cdecl("init_plugin_biometric")
func initPlugin() -> Plugin {
  return BiometricPlugin()
}
