// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import common from '@ohos.app.ability.common';
import { BusinessError } from '@ohos.base';
import { NativeModule, NativePlugin } from './NativePlugin';
import { Clipboard } from './plugins/Clipboard';
import { Dialog } from './plugins/Dialog';
import { Files } from './plugins/Files';
import { Notification } from './plugins/Notification';
import { Opener } from './plugins/Opener';
import { Barcode } from './plugins/Barcode';

interface Request { id: number; plugin: string; command: string; payload: string; }
interface NativeError { code: string; message: string; }

/** Installed in the native Ability, never in the untrusted WebView JavaScript context. */
export class TauriPlugins {
  private native: NativeModule;
  private plugins: Map<string, NativePlugin> = new Map();
  private active: boolean = true;

  constructor(context: common.UIAbilityContext, native: NativeModule) {
    this.native = native;
    const files = new Files();
    this.plugins.set('fs', files);
    this.plugins.set('clipboard-manager', new Clipboard(context));
    this.plugins.set('dialog', new Dialog(context, files));
    this.plugins.set('notification', new Notification(context));
    this.plugins.set('opener', new Opener(context));
    this.plugins.set('barcode-scanner', new Barcode(context));
    native.tauriOhosPluginInitialize((request: string): void => {
      void this.invoke(JSON.parse(request) as Request);
    }, context.filesDir, context.cacheDir, context.tempDir);
  }

  private async invoke(request: Request): Promise<void> {
    if (!this.active) return;
    try {
      const plugin = this.plugins.get(request.plugin);
      if (plugin === undefined) throw new Error(`Unknown native plugin: ${request.plugin}`);
      const response = await plugin.invoke(request.command, request.payload);
      if (this.active) this.native.tauriOhosPluginResponse(request.id, true, response);
    } catch (error) {
      if (!this.active) return;
      const nativeError = error as BusinessError;
      const response: NativeError = { code: typeof nativeError.code === 'number' ? String(nativeError.code) : 'OHOS_NATIVE_ERROR', message: nativeError.message || String(error) };
      this.native.tauriOhosPluginResponse(request.id, false, JSON.stringify(response));
    }
  }

  close(): void {
    if (!this.active) return;
    this.active = false;
    this.native.tauriOhosPluginClose();
    this.plugins.forEach((plugin: NativePlugin): void => plugin.close());
    this.plugins.clear();
  }
}
