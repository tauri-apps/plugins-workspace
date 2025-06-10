// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import UserNotifications

@objc public protocol NotificationHandlerProtocol {
  func willPresent(notification: UNNotification) -> UNNotificationPresentationOptions
  func didReceive(response: UNNotificationResponse)
}

@objc public class NotificationManager: NSObject, UNUserNotificationCenterDelegate {
  public weak var notificationHandler: NotificationHandlerProtocol?

  override init() {
    super.init()
    let center = UNUserNotificationCenter.current()
    center.delegate = self
  }

  public func userNotificationCenter(
    _ center: UNUserNotificationCenter,
    willPresent notification: UNNotification,
    withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
  ) {
    var presentationOptions: UNNotificationPresentationOptions? = nil

    if notification.request.trigger?.isKind(of: UNPushNotificationTrigger.self) != true {
      presentationOptions = notificationHandler?.willPresent(notification: notification)
    }

    completionHandler(presentationOptions ?? [])
  }

  public func userNotificationCenter(
    _ center: UNUserNotificationCenter,
    didReceive response: UNNotificationResponse,
    withCompletionHandler completionHandler: @escaping () -> Void
  ) {
    if response.notification.request.trigger?.isKind(of: UNPushNotificationTrigger.self) != true {
      notificationHandler?.didReceive(response: response)
    }

    completionHandler()
  }
}
