// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import WebKit
import CoreHaptics
import AudioToolbox

class ImpactFeedbackOptions: Decodable {
  let style: ImpactFeedbackStyle
}

enum ImpactFeedbackStyle: String, Decodable {
  case light, medium, heavy, soft, rigid

  func into() -> UIImpactFeedbackGenerator.FeedbackStyle {
    switch self {
    case .light:
      return .light
    case .medium:
      return .medium
    case .heavy:
      return .heavy
    case .soft:
      return .soft
    case .rigid:
      return .rigid
    }
  }
}

class NotificationFeedbackOptions: Decodable {
  let type: NotificationFeedbackType
}

enum NotificationFeedbackType: String, Decodable {
  case success, warning, error

  func into() -> UINotificationFeedbackGenerator.FeedbackType {
    switch self {
    case .success:
      return .success
    case .warning:
      return .warning
    case .error:
      return .error
    }
  }
}

class VibrateOptions: Decodable {
  // TODO: Array
  let duration: Double
}

class HapticsPlugin: Plugin {
  //
  // Tauri commands
  //

  @objc public func vibrate(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(VibrateOptions.self)
    if CHHapticEngine.capabilitiesForHardware().supportsHaptics {
      do {
        let engine = try CHHapticEngine()
        try engine.start()
        engine.resetHandler = { [] in
          do {
              try engine.start()
          } catch {
            AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
          }
        }
        // TODO: Make some of this (or all) configurable?
        let intensity: CHHapticEventParameter = CHHapticEventParameter(parameterID: .hapticIntensity, value: 1.0)
        let sharpness: CHHapticEventParameter = CHHapticEventParameter(parameterID: .hapticSharpness, value: 1.0)
        let continuousEvent = CHHapticEvent(
          eventType: .hapticContinuous,
          parameters: [intensity, sharpness],
          relativeTime: 0.0,
          duration: args.duration/1000
        )
        let pattern = try CHHapticPattern(events: [continuousEvent], parameters: [])
        let player = try engine.makePlayer(with: pattern)

        try player.start(atTime: 0)
      } catch {
        AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
      }
    } else {
      AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
    }

    Logger.error("VIBRATE END")

    invoke.resolve()
  }

  @objc public func impactFeedback(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ImpactFeedbackOptions.self)
    let generator = UIImpactFeedbackGenerator(style: args.style.into())
    generator.prepare()
    generator.impactOccurred()

    invoke.resolve()
  }

  @objc public func notificationFeedback(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(NotificationFeedbackOptions.self)
    let generator = UINotificationFeedbackGenerator()
    generator.prepare()
    generator.notificationOccurred(args.type.into())

    invoke.resolve()
  }

  // TODO: Consider breaking this up into Start,Change,End like capacitor
  @objc public func selectionFeedback(_ invoke: Invoke) throws {
    let generator = UISelectionFeedbackGenerator()
    generator.prepare()
    generator.selectionChanged()

    invoke.resolve()
  }
}

@_cdecl("init_plugin_haptics")
func initPlugin() -> Plugin {
  return HapticsPlugin()
}
