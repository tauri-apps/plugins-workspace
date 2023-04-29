import Tauri
import UserNotifications

enum NotificationError: LocalizedError {
  case contentNoId
  case contentNoTitle
  case contentNoBody
  case triggerConstructionFailed
  case triggerRepeatIntervalTooShort
  case attachmentNoId
  case attachmentNoUrl
  case attachmentFileNotFound(path: String)
  case attachmentUnableToCreate(String)

  var errorDescription: String? {
    switch self {
    case .attachmentFileNotFound(let path):
      return "Unable to find file \(path) for attachment"
    default:
      return ""
    }
  }
}

func makeNotificationContent(_ notification: JSObject) throws -> UNNotificationContent {
  guard let title = notification["title"] as? String else {
    throw NotificationError.contentNoTitle
  }
  guard let body = notification["body"] as? String else {
    throw NotificationError.contentNoBody
  }

  let extra = notification["extra"] as? JSObject ?? [:]
  let schedule = notification["schedule"] as? JSObject ?? [:]
  let content = UNMutableNotificationContent()
  content.title = NSString.localizedUserNotificationString(forKey: title, arguments: nil)
  content.body = NSString.localizedUserNotificationString(
    forKey: body,
    arguments: nil)

  content.userInfo = [
    "__EXTRA__": extra,
    "__SCHEDULE__": schedule,
  ]

  if let actionTypeId = notification["actionTypeId"] as? String {
    content.categoryIdentifier = actionTypeId
  }

  if let threadIdentifier = notification["group"] as? String {
    content.threadIdentifier = threadIdentifier
  }

  if let summaryArgument = notification["summary"] as? String {
    content.summaryArgument = summaryArgument
  }

  if let sound = notification["sound"] as? String {
    content.sound = UNNotificationSound(named: UNNotificationSoundName(sound))
  }

  if let attachments = notification["attachments"] as? [JSObject] {
    content.attachments = try makeAttachments(attachments)
  }

  return content
}

func makeAttachments(_ attachments: [JSObject]) throws -> [UNNotificationAttachment] {
  var createdAttachments = [UNNotificationAttachment]()

  for attachment in attachments {
    guard let id = attachment["id"] as? String else {
      throw NotificationError.attachmentNoId
    }
    guard let url = attachment["url"] as? String else {
      throw NotificationError.attachmentNoUrl
    }
    guard let urlObject = makeAttachmentUrl(url) else {
      throw NotificationError.attachmentFileNotFound(path: url)
    }

    let options = attachment["options"] as? JSObject ?? [:]

    do {
      let newAttachment = try UNNotificationAttachment(
        identifier: id, url: urlObject, options: makeAttachmentOptions(options))
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

func makeAttachmentOptions(_ options: JSObject) -> JSObject {
  var opts: JSObject = [:]

  if let iosUNNotificationAttachmentOptionsTypeHintKey = options[
    "iosUNNotificationAttachmentOptionsTypeHintKey"] as? String
  {
    opts[UNNotificationAttachmentOptionsTypeHintKey] = iosUNNotificationAttachmentOptionsTypeHintKey
  }
  if let iosUNNotificationAttachmentOptionsThumbnailHiddenKey = options[
    "iosUNNotificationAttachmentOptionsThumbnailHiddenKey"] as? String
  {
    opts[UNNotificationAttachmentOptionsThumbnailHiddenKey] =
      iosUNNotificationAttachmentOptionsThumbnailHiddenKey
  }
  if let iosUNNotificationAttachmentOptionsThumbnailClippingRectKey = options[
    "iosUNNotificationAttachmentOptionsThumbnailClippingRectKey"] as? String
  {
    opts[UNNotificationAttachmentOptionsThumbnailClippingRectKey] =
      iosUNNotificationAttachmentOptionsThumbnailClippingRectKey
  }
  if let iosUNNotificationAttachmentOptionsThumbnailTimeKey = options[
    "iosUNNotificationAttachmentOptionsThumbnailTimeKey"] as? String
  {
    opts[UNNotificationAttachmentOptionsThumbnailTimeKey] =
      iosUNNotificationAttachmentOptionsThumbnailTimeKey
  }
  return opts
}

func handleScheduledNotification(_ invoke: Invoke, _ schedule: JSObject) throws
  -> UNNotificationTrigger?
{
  let kind = schedule["kind"] as? String ?? ""
  let payload = schedule["data"] as? JSObject ?? [:]
  switch kind {
  case "At":
    let date = payload["date"] as? String ?? ""
    let dateFormatter = DateFormatter()
    dateFormatter.locale = Locale(identifier: "en_US_POSIX")
    dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"

    if let at = dateFormatter.date(from: date) {
      let repeats = payload["repeats"] as? Bool ?? false

      let dateInfo = Calendar.current.dateComponents(in: TimeZone.current, from: at)

      if dateInfo.date! < Date() {
        invoke.reject("Scheduled time must be *after* current time")
        return nil
      }

      let dateInterval = DateInterval(start: Date(), end: dateInfo.date!)

      // Notifications that repeat have to be at least a minute between each other
      if repeats && dateInterval.duration < 60 {
        throw NotificationError.triggerRepeatIntervalTooShort
      }

      return UNTimeIntervalNotificationTrigger(
        timeInterval: dateInterval.duration, repeats: repeats)

    } else {
      invoke.reject("could not parse `at` date \(date)")
    }
  case "Interval":
    let dateComponents = getDateComponents(payload)
    return UNCalendarNotificationTrigger(dateMatching: dateComponents, repeats: true)
  case "Every":
    let interval = payload["interval"] as? String ?? ""
    let count = schedule["count"] as? Int ?? 1

    if let repeatDateInterval = getRepeatDateInterval(interval, count) {
      // Notifications that repeat have to be at least a minute between each other
      if repeatDateInterval.duration < 60 {
        throw NotificationError.triggerRepeatIntervalTooShort
      }

      return UNTimeIntervalNotificationTrigger(
        timeInterval: repeatDateInterval.duration, repeats: true)
    }

  default:
    return nil
  }

  return nil
}

/// Given our schedule format, return a DateComponents object
/// that only contains the components passed in.

func getDateComponents(_ at: JSObject) -> DateComponents {
  // var dateInfo = Calendar.current.dateComponents(in: TimeZone.current, from: Date())
  // dateInfo.calendar = Calendar.current
  var dateInfo = DateComponents()

  if let year = at["year"] as? Int {
    dateInfo.year = year
  }
  if let month = at["month"] as? Int {
    dateInfo.month = month
  }
  if let day = at["day"] as? Int {
    dateInfo.day = day
  }
  if let hour = at["hour"] as? Int {
    dateInfo.hour = hour
  }
  if let minute = at["minute"] as? Int {
    dateInfo.minute = minute
  }
  if let second = at["second"] as? Int {
    dateInfo.second = second
  }
  if let weekday = at["weekday"] as? Int {
    dateInfo.weekday = weekday
  }
  return dateInfo
}

/// Compute the difference between the string representation of a date
/// interval and today. For example, if every is "month", then we
/// return the interval between today and a month from today.

func getRepeatDateInterval(_ every: String, _ count: Int) -> DateInterval? {
  let cal = Calendar.current
  let now = Date()
  switch every {
  case "Year":
    let newDate = cal.date(byAdding: .year, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case "Month":
    let newDate = cal.date(byAdding: .month, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case "TwoWeeks":
    let newDate = cal.date(byAdding: .weekOfYear, value: 2 * count, to: now)!
    return DateInterval(start: now, end: newDate)
  case "Week":
    let newDate = cal.date(byAdding: .weekOfYear, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case "Day":
    let newDate = cal.date(byAdding: .day, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case "Hour":
    let newDate = cal.date(byAdding: .hour, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case "Minute":
    let newDate = cal.date(byAdding: .minute, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  case "Second":
    let newDate = cal.date(byAdding: .second, value: count, to: now)!
    return DateInterval(start: now, end: newDate)
  default:
    return nil
  }
}
