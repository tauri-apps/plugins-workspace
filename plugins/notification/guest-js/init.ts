// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/core";
import type { PermissionState } from "@tauri-apps/api/core";
import type { Options } from "./index";

(function () {
  let permissionSettable = false;
  let permissionValue = "default";

  async function isPermissionGranted(): Promise<boolean> {
    // @ts-expect-error __TEMPLATE_windows__ will be replaced in rust before it's injected.
    if (window.Notification.permission !== "default" || __TEMPLATE_windows__) {
      return await Promise.resolve(
        window.Notification.permission === "granted",
      );
    }
    return await invoke("plugin:notification|is_permission_granted");
  }

  function setNotificationPermission(value: NotificationPermission): void {
    permissionSettable = true;
    // @ts-expect-error we can actually set this value on the webview
    window.Notification.permission = value;
    permissionSettable = false;
  }

  async function requestPermission(): Promise<PermissionState> {
    return await invoke<PermissionState>(
      "plugin:notification|request_permission",
    ).then((permission) => {
      setNotificationPermission(
        permission === "prompt" || permission === "prompt-with-rationale"
          ? "default"
          : permission,
      );
      return permission;
    });
  }

  async function sendNotification(options: string | Options): Promise<void> {
    if (typeof options === "object") {
      Object.freeze(options);
    }

    await invoke("plugin:notification|notify", {
      options:
        typeof options === "string"
          ? {
              title: options,
            }
          : options,
    });
  }

  // @ts-expect-error unfortunately we can't implement the whole type, so we overwrite it with our own version
  window.Notification = function (title, options) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const opts = options || {};
    void sendNotification(
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      Object.assign(opts, {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        title,
      }),
    );
  };

  // @ts-expect-error tauri does not have sync IPC :(
  window.Notification.requestPermission = requestPermission;

  Object.defineProperty(window.Notification, "permission", {
    enumerable: true,
    get: () => permissionValue,
    set: (v) => {
      if (!permissionSettable) {
        throw new Error("Readonly property");
      }
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      permissionValue = v;
    },
  });

  void isPermissionGranted().then(function (response) {
    if (response === null) {
      setNotificationPermission("default");
    } else {
      setNotificationPermission(response ? "granted" : "denied");
    }
  });
})();
