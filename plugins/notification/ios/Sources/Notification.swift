// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Tauri
import UserNotifications

enum NotificationError: LocalizedError {
  case triggerRepeatIntervalTooShort
  case attachmentFileNotFound(path: String)
  case attachmentUnableToCreate(String)
  case pastScheduledTime
  case invalidDate(String)

  var errorDescription: String? {
    switch self {
    case .triggerRepeatIntervalTooShort:
      return "Schedule interval too short, must be a least 1 minute"
    case .attachmentFileNotFound(let path):
      return "Unable to find file \(path) for attachment"
    case .attachmentUnableToCreate(let error):
      return "Failed to create attachment: \(error)"
    case .pastScheduledTime:
      return "Scheduled time must be *after* current time"
    case .invalidDate(let date):
      return "Could not parse date \(date)"
    }
  }
}

func makeNotificationContent(_ notification: Notification) throws -> UNNotificationContent {
  let content = UNMutableNotificationContent()
  content.title = NSString.localizedUserNotificationString(
    forKey: notification.title, arguments: nil)
  content.body = NSString.localizedUserNotificationString(
    forKey: notification.body,
    arguments: nil)

  content.userInfo = [
    "__EXTRA__": notification.extra as Any,
    "__SCHEDULE__": notification.schedule as Any,
  ]

  if let actionTypeId = notification.actionTypeId {
    content.categoryIdentifier = actionTypeId
  }

  if let threadIdentifier = notification.group {
    content.threadIdentifier = threadIdentifier
  }

  if let summaryArgument = notification.summary {
    content.summaryArgument = summaryArgument
  }

  if let sound = notification.sound {
    content.sound = UNNotificationSound(named: UNNotificationSoundName(sound))
  }

  if let attachments = notification.attachments {
    content.attachments = try makeAttachments(attachments)
  }

  return content
}

func makeAttachments(_ attachments: [NotificationAttachment]) throws -> [UNNotificationAttachment] {
  var createdAttachments = [UNNotificationAttachment]()

  for attachment in attachments {

    guard let urlObject = makeAttachmentUrl(attachment.url) else {
      throw NotificationError.attachmentFileNotFound(path: attachment.url)
    }

    let options = attachment.options != nil ? makeAttachmentOptions(attachment.options!) : nil

    do {
      let newAttachment = try UNNotificationAttachment(
        identifier: attachment.id, url: urlObject, options: options)
      createdAttachments.append(newAttachment)
    } catch {
      throw NotificationError.attachmentUnableToCreate(error.localizedDescription)
    }
  }

  return createdAttachments
}

func makeAttachmentUrl(_ path: String) -> URL? {
  return URL(string: path)
}

func makeAttachmentOptions(_ options: NotificationAttachmentOptions) -> [AnyHashable: Any] {
  var opts: [AnyHashable: Any] = [:]

  if let value = options.iosUNNotificationAttachmentOptionsTypeHintKey {
    opts[UNNotificationAttachmentOptionsTypeHintKey] = value
  }
  if let value = options.iosUNNotificationAttachmentOptionsThumbnailHiddenKey {
    opts[UNNotificationAttachmentOptionsThumbnailHiddenKey] = value
  }
  if let value = options.iosUNNotificationAttachmentOptionsThumbnailClippingRectKey {
    opts[UNNotificationAttachmentOptionsThumbnailClippingRectKey] = value
  }
  if let value = options
    .iosUNNotificationAttachmentOptionsThumbnailTimeKey

  {
    opts[UNNotificationAttachmentOptionsThumbnailTimeKey] = value
  }
  return opts
}

func handleScheduledNotification(_ schedule: NotificationSchedule) throws
  -> UNNotificationTrigger?
{
  switch schedule {
  case .at(let date, let repeating):
    let dateFormatter = DateFormatter()
    dateFormatter.locale = Locale(identifier: "en_US_POSIX")
    dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"

    if let at = dateFormatter.date(from: date) {
      let dateInfo = Calendar.current.dateComponents(in: TimeZone.current, from: at)

      if dateInfo.date! < Date() {
        throw NotificationError.pastScheduledTime
      }

      let dateInterval = DateInterval(start: Date(), end: dateInfo.date!)

      // Notifications that repeat have to be at least a minute between each other
      if repeating && dateInterval.duration < 60 {
        throw NotificationError.triggerRepeatIntervalTooShort
      }

      return UNTimeIntervalNotificationTrigger(
        timeInterval: dateInterval.duration, repeats: repeating)

    } else {
      throw NotificationError.invalidDate(date)
    }
  case .interval(let interval):
    let dateComponents = getDateComponents(interval)
    return UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: true)
  case .every(let interval, let count):
    if let repeatDateInterval = getRepeatDateInterval(interval, count) {
      // Notifications that repeat have to be at least a minute between each other
      if repeatDateInterval.duration < 60 {
        throw NotificationError.triggerRepeatIntervalTooShort
      }

      return UNTimeIntervalNotificationTrigger(
        timeInterval: repeatDateInterval.duration, repeats: true)
    }
  }

  return nil
}

/// Given our schedule format, return a DateComponents object
/// that only contains the components passed in.

func getDateComponents(_ at: ScheduleInterval) -> DateComponents {
  // var dateInfo = Calendar.current.dateComponents(in: TimeZone.current, from: Date())
  // dateInfo.calendar = Calendar.current
  var dateInfo = DateComponents()

  if let year = at.year {
    dateInfo.year = year
  }
  if let month = at.month {
    dateInfo.month = month
  }
  if let day = at.day {
    dateInfo.day = day
  }
  if let hour = at.hour {
    dateInfo.hour = hour
  }
  if let minute = at.minute {
    dateInfo.minute = minute
  }
  if let second = at.second {
    dateInfo.second = second
  }
  if let weekday = at.weekday {
    dateInfo.weekday = weekday
  }
  return dateInfo
}

/// Compute the difference between the string representation of a date
/// interval and today. For example, if every is "month", then we
/// return the interval between today and a month from today.

func getRepeatDateInterval(_ every: ScheduleEveryKind, _ count: Int) -> DateInterval? {
  let cal = Calendar.current
  let now = Date()
  switch every {
  case .year:
    let newDate = cal.date(byAdding: .year, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case .month:
    let newDate = cal.date(byAdding: .month, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case .twoWeeks:
    let newDate = cal.date(byAdding: .weekOfYear, value: 2 * count, to: now)!
    return DateInterval(start: now, end: newDate)
  case .week:
    let newDate = cal.date(byAdding: .weekOfYear, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case .day:
    let newDate = cal.date(byAdding: .day, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case .hour:
    let newDate = cal.date(byAdding: .hour, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case .minute:
    let newDate = cal.date(byAdding: .minute, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case .second:
    let newDate = cal.date(byAdding: .second, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  }
}
