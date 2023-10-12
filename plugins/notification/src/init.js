// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

(function () {
  let permissionSettable = false;
  let permissionValue = "default";

  function isPermissionGranted() {
    if (window.Notification.permission !== "default") {
      return Promise.resolve(window.Notification.permission === "granted");
    }
    return window.__TAURI_INVOKE__("plugin:notification|is_permission_granted");
  }

  function setNotificationPermission(value) {
    permissionSettable = true;
    // @ts-expect-error we can actually set this value on the webview
    window.Notification.permission = value;
    permissionSettable = false;
  }

  function requestPermission() {
    return window
      .__TAURI_INVOKE__("plugin:notification|request_permission")
      .then(function (permission) {
        setNotificationPermission(
          permission === "prompt" ? "default" : permission,
        );
        return permission;
      });
  }

  function sendNotification(options) {
    if (typeof options === "object") {
      Object.freeze(options);
    }

    return window.__TAURI_INVOKE__("plugin:notification|notify", {
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
    const opts = options || {};
    sendNotification(Object.assign(opts, { title }));
  };

  window.Notification.requestPermission = requestPermission;

  Object.defineProperty(window.Notification, "permission", {
    enumerable: true,
    get: function () {
      return permissionValue;
    },
    set: function (v) {
      if (!permissionSettable) {
        throw new Error("Readonly property");
      }
      permissionValue = v;
    },
  });

  isPermissionGranted().then(function (response) {
    if (response === null) {
      setNotificationPermission("default");
    } else {
      setNotificationPermission(response ? "granted" : "denied");
    }
  });
})();
