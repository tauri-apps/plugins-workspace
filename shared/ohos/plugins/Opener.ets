// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import common from '@ohos.app.ability.common';
import fileUri from '@ohos.file.fileuri';
import wantConstant from '@ohos.app.ability.wantConstant';
import { NativePlugin } from '../NativePlugin';

interface OpenRequest { url: string; with?: string; }

export class Opener implements NativePlugin {
  private context: common.UIAbilityContext;
  constructor(context: common.UIAbilityContext) { this.context = context; }
  async invoke(command: string, payload: string): Promise<string> {
    if (command !== 'open') throw new Error(`Unsupported opener command: ${command}`);
    let uri: string;
    if (payload.startsWith('"')) {
      const path = JSON.parse(payload) as string;
      uri = path.startsWith('/') ? fileUri.getUriFromPath(path) : path;
    } else {
      const request = JSON.parse(payload) as OpenRequest;
      if (request.with) throw new Error('Choosing a specific browser is not supported on OHOS');
      uri = request.url;
    }
    if (uri.startsWith('https://') || uri.startsWith('http://')) {
      await this.context.openLink(uri);
    } else {
      await this.context.startAbility({
        action: 'ohos.want.action.viewData', uri,
        flags: wantConstant.Flags.FLAG_AUTH_READ_URI_PERMISSION
      });
    }
    return 'null';
  }
  close(): void {}
}
