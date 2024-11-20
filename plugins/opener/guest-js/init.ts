// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

// open <a href="..."> links with the API
window.addEventListener('click', function (evt) {
  // return early if
  if (
    // event was prevented
    evt.defaultPrevented ||
    // or not a left click
    evt.button !== 0 ||
    // or meta key pressed
    evt.metaKey ||
    // or al key pressed
    evt.altKey
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
    !a ||
    // or doesn't have a href
    !a.href ||
    // or not supposed to be open in a new tab
    !(
      a.target === '_blank' ||
      // or ctrl key pressed
      evt.ctrlKey ||
      // or shift key pressed
      evt.shiftKey
    )
  )
    return

  const url = new URL(a.href)

  // return early if
  if (
    // same origin (internal navigation)
    url.origin === window.location.origin ||
    // not default protocols
    ['http:', 'https:', 'mailto:', 'tel:'].every((p) => url.protocol !== p)
  )
    return

  evt.preventDefault()

  void invoke('plugin:opener|open_url', {
    url
  })
})
