// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import assert from 'node:assert/strict';
import { registerHooks } from 'node:module';
import test from 'node:test';
import { pathToFileURL } from 'node:url';
import { calls, deferOpen, reset } from './fs-double.mjs';

registerHooks({ resolve(specifier, context, nextResolve) {
  if (specifier === '@ohos.file.fs') return { url: new URL('./fs-double.mjs', import.meta.url).href, shortCircuit: true };
  return nextResolve(specifier, context);
} });
const { Files } = await import(pathToFileURL(process.env.OHOS_FILES_TEST_MODULE).href);
const request = (uri, extra = {}) => JSON.stringify({ uri, read: true, write: false, append: false, truncate: false, create: false, createNew: false, ...extra });

test('unpicked URIs cannot bypass filesystem scope through the mobile URL path', async () => {
  reset();
  const files = new Files();
  await assert.rejects(files.invoke('openFile', request('file:///data/private/secret')), /not granted/);
  assert.equal(calls.opened.length, 0);
});

test('a read grant permits reading but never truncation, creation, append or writing', async () => {
  reset();
  const files = new Files();
  const uri = 'file://media/Photo/42/image.jpg';
  files.grant(uri, false);
  for (const flag of ['write', 'append', 'truncate', 'create', 'createNew']) {
    await assert.rejects(files.invoke('openFile', request(uri, { write: true, [flag]: true })), /not granted/);
  }
  assert.equal(calls.opened.length, 0);
  assert.equal(await files.invoke('openFile', request(uri)), '17');
  await files.invoke('closeFile', JSON.stringify({ fd: 17 }));
  files.close();
  assert.deepEqual(calls.closed, [17]);
});

test('an open completing after Ability teardown is rejected and its descriptor is closed', async () => {
  reset();
  let finish;
  deferOpen(new Promise(resolve => { finish = resolve; }));
  const files = new Files();
  const uri = 'file://docs/document.txt';
  files.grant(uri, true);
  const pending = files.invoke('openFile', request(uri));
  files.close();
  finish({ fd: 91 });
  await assert.rejects(pending, /destroyed/);
  files.close();
  assert.deepEqual(calls.closed, [91]);
});

test('exclusive creation cannot silently overwrite a picked document', async () => {
  reset();
  const files = new Files();
  const uri = 'file://docs/existing.txt';
  files.grant(uri, true);
  await assert.rejects(files.invoke('openFile', request(uri, { write: true, createNew: true })), /Exclusive/);
  assert.equal(calls.opened.length, 0);
});
