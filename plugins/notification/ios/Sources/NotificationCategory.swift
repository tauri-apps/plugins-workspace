// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Tauri
import UserNotifications

internal func makeCategories(_ actionTypes: [ActionType]) {
  var createdCategories = [UNNotificationCategory]()

  let generalCategory = UNNotificationCategory(
    identifier: "GENERAL",
    actions: [],
    intentIdentifiers: [],
    options: .customDismissAction)

  createdCategories.append(generalCategory)
  for type in actionTypes {
    let newActions = makeActions(type.actions)

    // Create the custom actions for the TIMER_EXPIRED category.
    var newCategory: UNNotificationCategory?

    newCategory = UNNotificationCategory(
      identifier: type.id,
      actions: newActions,
      intentIdentifiers: [],
      hiddenPreviewsBodyPlaceholder: type.hiddenBodyPlaceholder ?? "",
      options: makeCategoryOptions(type))

    createdCategories.append(newCategory!)
  }

  let center = UNUserNotificationCenter.current()
  center.setNotificationCategories(Set(createdCategories))
}

func makeActions(_ actions: [Action]) -> [UNNotificationAction] {
  var createdActions = [UNNotificationAction]()

  for action in actions {
    var newAction: UNNotificationAction
    if action.input ?? false {
      if action.inputButtonTitle != nil {
        newAction = UNTextInputNotificationAction(
          identifier: action.id,
          title: action.title,
          options: makeActionOptions(action),
          textInputButtonTitle: action.inputButtonTitle ?? "",
          textInputPlaceholder: action.inputPlaceholder ?? "")
      } else {
        newAction = UNTextInputNotificationAction(
          identifier: action.id, title: action.title, options: makeActionOptions(action))
      }
    } else {
      // Create the custom actions for the TIMER_EXPIRED category.
      newAction = UNNotificationAction(
        identifier: action.id,
        title: action.title,
        options: makeActionOptions(action))
    }
    createdActions.append(newAction)
  }

  return createdActions
}

func makeActionOptions(_ action: Action) -> UNNotificationActionOptions {
  if action.foreground ?? false {
    return .foreground
  }
  if action.destructive ?? false {
    return .destructive
  }
  if action.requiresAuthentication ?? false {
    return .authenticationRequired
  }
  return UNNotificationActionOptions(rawValue: 0)
}

func makeCategoryOptions(_ type: ActionType) -> UNNotificationCategoryOptions {
  if type.customDismissAction ?? false {
    return .customDismissAction
  }
  if type.allowInCarPlay ?? false {
    return .allowInCarPlay
  }

  if type.hiddenPreviewsShowTitle ?? false {
    return .hiddenPreviewsShowTitle
  }
  if type.hiddenPreviewsShowSubtitle ?? false {
    return .hiddenPreviewsShowSubtitle
  }

  return UNNotificationCategoryOptions(rawValue: 0)
}
