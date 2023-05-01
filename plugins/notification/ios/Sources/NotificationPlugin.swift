import SwiftRs
import Tauri
import UIKit
import UserNotifications
import WebKit

enum ShowNotificationError: LocalizedError {
  case noId
  case make(Error)
  case create(Error)

  var errorDescription: String? {
    switch self {
    case .noId:
      return "notification `id` missing"
    case .make(let error):
      return "Unable to make notification: \(error)"
    case .create(let error):
      return "Unable to create notification: \(error)"
    }
  }
}

func showNotification(invoke: Invoke, notification: JSObject)
  throws -> UNNotificationRequest
{
  guard let identifier = notification["id"] as? Int else {
    throw ShowNotificationError.noId
  }

  var content: UNNotificationContent
  do {
    content = try makeNotificationContent(notification)
  } catch {
    throw ShowNotificationError.make(error)
  }

  var trigger: UNNotificationTrigger?

  do {
    if let schedule = notification["schedule"] as? JSObject {
      try trigger = handleScheduledNotification(schedule)
    }
  } catch {
    throw ShowNotificationError.create(error)
  }

  // Schedule the request.
  let request = UNNotificationRequest(
    identifier: "\(identifier)", content: content, trigger: trigger
  )

  let center = UNUserNotificationCenter.current()
  center.add(request) { (error: Error?) in
    if let theError = error {
      invoke.reject(theError.localizedDescription)
    }
  }

  return request
}

class NotificationPlugin: Plugin {
  let notificationHandler = NotificationHandler()
  let notificationManager = NotificationManager()

  override init() {
    super.init()
    notificationManager.notificationHandler = notificationHandler
    notificationHandler.plugin = self
  }

  @objc public func show(_ invoke: Invoke) throws {
    let request = try showNotification(invoke: invoke, notification: invoke.data)
    notificationHandler.saveNotification(request.identifier, invoke.data)
    invoke.resolve([
      "id": Int(request.identifier) ?? -1
    ])
  }

  @objc public func batch(_ invoke: Invoke) throws {
    guard let notifications = invoke.getArray("notifications", JSObject.self) else {
      invoke.reject("`notifications` array is required")
      return
    }
    var ids = [Int]()

    for notification in notifications {
      let request = try showNotification(invoke: invoke, notification: notification)
      notificationHandler.saveNotification(request.identifier, notification)
      ids.append(Int(request.identifier) ?? -1)
    }

    invoke.resolve([
      "notifications": ids
    ])
  }

  @objc public override func requestPermissions(_ invoke: Invoke) {
    notificationHandler.requestPermissions { granted, error in
      guard error == nil else {
        invoke.reject(error!.localizedDescription)
        return
      }
      invoke.resolve(["permissionState": granted ? "granted" : "denied"])
    }
  }

  @objc public override func checkPermissions(_ invoke: Invoke) {
    notificationHandler.checkPermissions { status in
      let permission: String

      switch status {
      case .authorized, .ephemeral, .provisional:
        permission = "granted"
      case .denied:
        permission = "denied"
      case .notDetermined:
        permission = "default"
      @unknown default:
        permission = "default"
      }

      invoke.resolve(["permissionState": permission])
    }
  }

  @objc func cancel(_ invoke: Invoke) {
    guard let notifications = invoke.getArray("notifications", NSNumber.self),
      notifications.count > 0
    else {
      invoke.reject("`notifications` input is required")
      return
    }

    UNUserNotificationCenter.current().removePendingNotificationRequests(
      withIdentifiers: notifications.map({ (id) -> String in
        return id.stringValue
      })
    )
    invoke.resolve()
  }

  @objc func getPending(_ invoke: Invoke) {
    UNUserNotificationCenter.current().getPendingNotificationRequests(completionHandler: {
      (notifications) in
      let ret = notifications.compactMap({ [weak self] (notification) -> JSObject? in
        return self?.notificationHandler.makePendingNotificationRequestJSObject(notification)
      })

      invoke.resolve([
        "notifications": ret
      ])
    })
  }

  @objc func registerActionTypes(_ invoke: Invoke) throws {
    guard let types = invoke.getArray("types", JSObject.self) else {
      return
    }
    try makeCategories(types)
    invoke.resolve()
  }

  @objc func removeActive(_ invoke: Invoke) {
    if let notifications = invoke.getArray("notifications", JSObject.self) {
      let ids = notifications.map { "\($0["id"] ?? "")" }
      UNUserNotificationCenter.current().removeDeliveredNotifications(withIdentifiers: ids)
      invoke.resolve()
    } else {
      UNUserNotificationCenter.current().removeAllDeliveredNotifications()
      DispatchQueue.main.async(execute: {
        UIApplication.shared.applicationIconBadgeNumber = 0
      })
      invoke.resolve()
    }
  }

  @objc func getActive(_ invoke: Invoke) {
    UNUserNotificationCenter.current().getDeliveredNotifications(completionHandler: {
      (notifications) in
      let ret = notifications.map({ (notification) -> [String: Any] in
        return self.notificationHandler.makeNotificationRequestJSObject(
          notification.request)
      })
      invoke.resolve([
        "notifications": ret
      ])
    })
  }

  @objc func createChannel(_ invoke: Invoke) {
    invoke.reject("not implemented")
  }

  @objc func deleteChannel(_ invoke: Invoke) {
    invoke.reject("not implemented")
  }

  @objc func listChannels(_ invoke: Invoke) {
    invoke.reject("not implemented")
  }

}

@_cdecl("init_plugin_notification")
func initPlugin() -> Plugin {
  return NotificationPlugin()
}
