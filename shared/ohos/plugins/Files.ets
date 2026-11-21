// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import fs from '@ohos.file.fs';
import { NativePlugin } from '../NativePlugin';

interface OpenRequest {
  uri: string; read: boolean; write: boolean; append: boolean;
  truncate: boolean; create: boolean; createNew: boolean;
}
interface CloseRequest { fd: number; }

export class Files implements NativePlugin {
  private grants: Map<string, number> = new Map();
  private handles: Map<number, fs.File> = new Map();
  private active: boolean = true;

  grant(uri: string, writable: boolean): void {
    if (!this.active) throw new Error('Ability has been destroyed');
    // Only exact URIs returned by the native picker receive access.
    this.grants.set(uri, (this.grants.get(uri) ?? 0) | (writable ? 3 : 1));
  }

  async invoke(command: string, payload: string): Promise<string> {
    if (!this.active) throw new Error('Ability has been destroyed');
    if (command === 'closeFile') {
      const request = JSON.parse(payload) as CloseRequest;
      const file = this.handles.get(request.fd);
      if (file === undefined) throw new Error('Unknown native file handle');
      fs.closeSync(file);
      this.handles.delete(request.fd);
      return 'null';
    }
    if (command !== 'openFile') throw new Error(`Unsupported filesystem command: ${command}`);
    const request = JSON.parse(payload) as OpenRequest;
    const access = this.grants.get(request.uri) ?? 0;
    const writes = request.write || request.append || request.truncate || request.create || request.createNew;
    if (access === 0 || (writes && (access & 2) === 0)) {
      throw new Error('The native picker has not granted the requested URI access');
    }
    const writeAccess = request.write || request.append;
    if ((!request.read && !writeAccess) || ((request.truncate || request.create || request.createNew) && !writeAccess)) {
      throw new Error('Invalid file access mode');
    }
    let mode = writeAccess ? (request.read ? fs.OpenMode.READ_WRITE : fs.OpenMode.WRITE_ONLY) : fs.OpenMode.READ_ONLY;
    if (request.append) mode |= fs.OpenMode.APPEND;
    if (request.truncate) mode |= fs.OpenMode.TRUNC;
    if (request.create || request.createNew) mode |= fs.OpenMode.CREATE;
    if (request.createNew) throw new Error('Exclusive creation is not supported for picker URIs');
    const file = await fs.open(request.uri, mode);
    if (!this.active) {
      fs.closeSync(file);
      throw new Error('Ability was destroyed while opening the file');
    }
    this.handles.set(file.fd, file);
    return JSON.stringify(file.fd);
  }

  close(): void {
    this.active = false;
    this.grants.clear();
    this.handles.forEach((file: fs.File): void => {
      try { fs.closeSync(file); } catch (error) { console.error(`Native file close failed: ${String(error)}`); }
    });
    this.handles.clear();
  }
}
