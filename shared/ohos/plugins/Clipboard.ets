// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import pasteboard from '@ohos.pasteboard';
import common from '@ohos.app.ability.common';
import abilityAccessCtrl from '@ohos.abilityAccessCtrl';
import { NativePlugin } from '../NativePlugin';

interface PlainText { text: string; label?: string; }
interface TextPayload { plainText: PlainText; }

export class Clipboard implements NativePlugin {
  private context: common.UIAbilityContext;
  constructor(context: common.UIAbilityContext) { this.context = context; }
  async invoke(command: string, payload: string): Promise<string> {
    const board = pasteboard.getSystemPasteboard();
    switch (command) {
      case 'writeText': {
        const request = JSON.parse(payload) as TextPayload;
        await board.setData(pasteboard.createData(pasteboard.MIMETYPE_TEXT_PLAIN, request.plainText.text));
        return 'null';
      }
      case 'readText': {
        const manager = abilityAccessCtrl.createAtManager();
        if (await manager.checkAccessToken(this.context.applicationInfo.accessTokenId, 'ohos.permission.READ_PASTEBOARD') !== abilityAccessCtrl.GrantStatus.PERMISSION_GRANTED) {
          const access = await manager.requestPermissionsFromUser(this.context, ['ohos.permission.READ_PASTEBOARD']);
          if (access.authResults.length !== 1 || access.authResults[0] !== 0) throw new Error('Clipboard permission denied');
        }
        const data = await board.getData();
        const result: TextPayload = { plainText: { text: data.getPrimaryText() } };
        return JSON.stringify(result);
      }
      case 'clear':
        await board.clearData();
        return 'null';
      default: throw new Error(`Unsupported clipboard command: ${command}`);
    }
  }
  close(): void {}
}
