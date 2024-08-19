// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/* eslint-disable @typescript-eslint/unbound-method */

import { commands } from "./bindings";

export const {
  vibrate,
  impactFeedback,
  notificationFeedback,
  selectionFeedback,
} = commands;

export { ImpactFeedbackStyle, NotificationFeedbackType } from "./bindings";

// export { events };
