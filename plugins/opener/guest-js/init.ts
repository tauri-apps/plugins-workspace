// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Webview script injected by the opener plugin on every page load.
 *
 * It installs a `click` listener on `window` that intercepts clicks on `<a>` elements whose
 * `target` is `_blank` (or that are clicked while holding `Ctrl` or `Shift`) and whose `href`
 * uses the `http:`, `https:`, `mailto:` or `tel:` protocol, cancels the navigation and opens the
 * link with the system's default browser through the `plugin:opener|open_url` command instead.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

// open <a href="..."> links with the API
window.addEventListener('click', function (evt) {
  // return early if
  if (
    // event was prevented
    evt.defaultPrevented
    // or not a left click
    || evt.button !== 0
    // or meta key pressed
    || evt.metaKey
    // or al key pressed
    || evt.altKey
  )
    return

  const a = evt
    .composedPath()
    .find((el) => el instanceof Node && el.nodeName.toUpperCase() === 'A') as
    | HTMLAnchorElement
    | undefined

  // return early if
  if (
    // not tirggered from <a> element
    !a
    // or doesn't have a href
    || !a.href
    // or not supposed to be open in a new tab
    || !(
      a.target === '_blank'
      // or ctrl key pressed
      || evt.ctrlKey
      // or shift key pressed
      || evt.shiftKey
    )
  )
    return

  const url = new URL(a.href)

  // return early if
  if (
    // not default protocols
    ['http:', 'https:', 'mailto:', 'tel:'].every((p) => url.protocol !== p)
  )
    return

  evt.preventDefault()

  void invoke('plugin:opener|open_url', {
    url
  })
})
