// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Tauri
import UserNotifications

public class NotificationHandler: NSObject, NotificationHandlerProtocol {

  public weak var plugin: Plugin?

  private var notificationsMap = [String: Notification]()

  internal func saveNotification(_ key: String, _ notification: Notification) {
    notificationsMap.updateValue(notification, forKey: key)
  }

  public func requestPermissions(with completion: ((Bool, Error?) -> Void)? = nil) {
    let center = UNUserNotificationCenter.current()
    center.requestAuthorization(options: [.badge, .alert, .sound]) { (granted, error) in
      completion?(granted, error)
    }
  }

  public func checkPermissions(with completion: ((UNAuthorizationStatus) -> Void)? = nil) {
    let center = UNUserNotificationCenter.current()
    center.getNotificationSettings { settings in
      completion?(settings.authorizationStatus)
    }
  }

  public func willPresent(notification: UNNotification) -> UNNotificationPresentationOptions {
    let notificationData = toActiveNotification(notification.request)
    try? self.plugin?.trigger("notification", data: notificationData)

    if let options = notificationsMap[notification.request.identifier] {
      if options.silent ?? false {
        return UNNotificationPresentationOptions.init(rawValue: 0)
      }
    }

    return [
      .badge,
      .sound,
      .alert,
    ]
  }

  public func didReceive(response: UNNotificationResponse) {
    let originalNotificationRequest = response.notification.request
    let actionId = response.actionIdentifier

    var actionIdValue: String
    // We turn the two default actions (open/dismiss) into generic strings
    if actionId == UNNotificationDefaultActionIdentifier {
      actionIdValue = "tap"
    } else if actionId == UNNotificationDismissActionIdentifier {
      actionIdValue = "dismiss"
    } else {
      actionIdValue = actionId
    }

    var inputValue: String? = nil
    // If the type of action was for an input type, get the value
    if let inputType = response as? UNTextInputNotificationResponse {
      inputValue = inputType.userText
    }

    // Payload contract note:
    // Keep this structure aligned with Android for `actionPerformed`:
    // - `actionId`
    // - `inputValue` (optional)
    // - `notification` (with at least `id`, optionally `actionTypeId`)
    //
    // Delivery note:
    // This event is emitted as soon as the OS callback arrives. If the JS side has
    // not attached `onAction` yet (for example during cold start), the event may be
    // missed. Android mitigates this with a pending-action queue and an explicit
    // "listener ready" handshake from JS.
    //
    // iOS parity guidance:
    // 1. Queue `ReceivedNotification` payloads at this boundary when listeners are not ready.
    // 2. Expose a small command that JS calls after `onAction` registration to drain the queue.
    // 3. Consider persistence (for example `UserDefaults`) so queued actions survive process restarts.
    // 4. Keep the payload contract aligned with Android (`actionId`, `inputValue`, `notification`).
    //
    // Important: queue payloads should be built from `UNNotificationResponse` fields directly
    // where possible, since `notificationsMap` is in-memory and may be unavailable after restart.
    try? self.plugin?.trigger(
      "actionPerformed",
      data: ReceivedNotification(
        actionId: actionIdValue,
        inputValue: inputValue,
        notification: toActiveNotification(originalNotificationRequest)
      ))
  }

  func toActiveNotification(_ request: UNNotificationRequest) -> ActiveNotification {
    let notificationRequest = notificationsMap[request.identifier]!
    return ActiveNotification(
      id: Int(request.identifier) ?? -1,
      title: request.content.title,
      body: request.content.body,
      sound: notificationRequest.sound ?? "",
      actionTypeId: request.content.categoryIdentifier,
      attachments: notificationRequest.attachments
    )
  }

  func toPendingNotification(_ request: UNNotificationRequest) -> PendingNotification {
    return PendingNotification(
      id: Int(request.identifier) ?? -1,
      title: request.content.title,
      body: request.content.body
    )
  }
}

struct PendingNotification: Encodable {
  let id: Int
  let title: String
  let body: String
}

struct ActiveNotification: Encodable {
  let id: Int
  let title: String
  let body: String
  let sound: String
  let actionTypeId: String
  let attachments: [NotificationAttachment]?
}

struct ReceivedNotification: Encodable {
  let actionId: String
  let inputValue: String?
  let notification: ActiveNotification
}
