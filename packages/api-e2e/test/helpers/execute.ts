// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Runs a callback-style async script in the page: `script` is a function body
 * that receives `args` followed by a `done` callback as its `arguments`, and
 * the value passed to `done` is returned.
 *
 * Stands in for `browser.executeAsync`, which WebdriverIO v10 removed: its
 * replacement (`execute` with an `async` function) only takes functions, which
 * the drivers' script transport would stringify anyway. Classic sessions use
 * the WebDriver "Execute Async Script" command directly; BiDi has no such
 * command, so the script is wrapped in a promise, which BiDi awaits.
 *
 * Takes the browser as a parameter (rather than importing `@wdio/globals`) so
 * the wdio configs can use it from their hooks.
 */
export async function executeAsync(
  browser: WebdriverIO.Browser,
  script: string,
  ...args: unknown[]
): Promise<unknown> {
  if (browser.isBidi) {
    return browser.execute(
      `var args = Array.prototype.slice.call(arguments);
      return new Promise(function (done) {
        (function () { ${script} }).apply(null, args.concat([done]));
      });`,
      ...args
    )
  }
  return browser.executeAsyncScript(
    script,
    // Any JSON-serializable values; the driver rejects anything else.
    args as Parameters<WebdriverIO.Browser['executeAsyncScript']>[1]
  )
}
