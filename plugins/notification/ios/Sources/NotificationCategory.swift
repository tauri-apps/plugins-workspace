import Tauri
import UserNotifications

public func makeCategories(_ actionTypes: [JSObject]) {
  var createdCategories = [UNNotificationCategory]()

  let generalCategory = UNNotificationCategory(
    identifier: "GENERAL",
    actions: [],
    intentIdentifiers: [],
    options: .customDismissAction)

  createdCategories.append(generalCategory)
  for type in actionTypes {
    guard let id = type["id"] as? String else {
      Logger.error("Action type must have an id field")
      continue
    }
    let hiddenBodyPlaceholder = type["iosHiddenPreviewsBodyPlaceholder"] as? String ?? ""
    let actions = type["actions"] as? [JSObject] ?? []

    let newActions = makeActions(actions)

    // Create the custom actions for the TIMER_EXPIRED category.
    var newCategory: UNNotificationCategory?

    newCategory = UNNotificationCategory(
      identifier: id,
      actions: newActions,
      intentIdentifiers: [],
      hiddenPreviewsBodyPlaceholder: hiddenBodyPlaceholder,
      options: makeCategoryOptions(type))

    createdCategories.append(newCategory!)
  }

  let center = UNUserNotificationCenter.current()
  center.setNotificationCategories(Set(createdCategories))
}

func makeActions(_ actions: [JSObject]) -> [UNNotificationAction] {
  var createdActions = [UNNotificationAction]()

  for action in actions {
    guard let id = action["id"] as? String else {
      Logger.error("Action must have an id field")
      continue
    }
    let title = action["title"] as? String ?? ""
    let input = action["input"] as? Bool ?? false

    var newAction: UNNotificationAction
    if input {
      let inputButtonTitle = action["inputButtonTitle"] as? String
      let inputPlaceholder = action["inputPlaceholder"] as? String ?? ""

      if inputButtonTitle != nil {
        newAction = UNTextInputNotificationAction(
          identifier: id,
          title: title,
          options: makeActionOptions(action),
          textInputButtonTitle: inputButtonTitle!,
          textInputPlaceholder: inputPlaceholder)
      } else {
        newAction = UNTextInputNotificationAction(
          identifier: id, title: title, options: makeActionOptions(action))
      }
    } else {
      // Create the custom actions for the TIMER_EXPIRED category.
      newAction = UNNotificationAction(
        identifier: id,
        title: title,
        options: makeActionOptions(action))
    }
    createdActions.append(newAction)
  }

  return createdActions
}

func makeActionOptions(_ action: JSObject) -> UNNotificationActionOptions {
  let foreground = action["foreground"] as? Bool ?? false
  let destructive = action["destructive"] as? Bool ?? false
  let requiresAuthentication = action["requiresAuthentication"] as? Bool ?? false

  if foreground {
    return .foreground
  }
  if destructive {
    return .destructive
  }
  if requiresAuthentication {
    return .authenticationRequired
  }
  return UNNotificationActionOptions(rawValue: 0)
}

func makeCategoryOptions(_ type: JSObject) -> UNNotificationCategoryOptions {
  let customDismiss = type["iosCustomDismissAction"] as? Bool ?? false
  let carPlay = type["iosAllowInCarPlay"] as? Bool ?? false
  let hiddenPreviewsShowTitle = type["iosHiddenPreviewsShowTitle"] as? Bool ?? false
  let hiddenPreviewsShowSubtitle = type["iosHiddenPreviewsShowSubtitle"] as? Bool ?? false

  if customDismiss {
    return .customDismissAction
  }
  if carPlay {
    return .allowInCarPlay
  }

  if hiddenPreviewsShowTitle {
    return .hiddenPreviewsShowTitle
  }
  if hiddenPreviewsShowSubtitle {
    return .hiddenPreviewsShowSubtitle
  }

  return UNNotificationCategoryOptions(rawValue: 0)
}
