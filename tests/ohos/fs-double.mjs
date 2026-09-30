// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
export const calls = { opened: [], closed: [] };
let openResult;
export function reset() { calls.opened.length = 0; calls.closed.length = 0; openResult = undefined; }
export function deferOpen(result) { openResult = result; }
export default {
  OpenMode: { READ_ONLY: 0, WRITE_ONLY: 1, READ_WRITE: 2, APPEND: 4, TRUNC: 8, CREATE: 16 },
  async open(uri, mode) { calls.opened.push({ uri, mode }); return openResult ?? { fd: 17 }; },
  closeSync(file) { calls.closed.push(file.fd); }
};
