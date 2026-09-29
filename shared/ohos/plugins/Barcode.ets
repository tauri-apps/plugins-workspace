// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import common from '@ohos.app.ability.common';
import abilityAccessCtrl from '@ohos.abilityAccessCtrl';
import { scanBarcode, scanCore } from '@kit.ScanKit';
import { NativePlugin } from '../NativePlugin';

interface ScanRequest { formats?: string[]; cameraDirection?: string; windowed?: boolean; }
interface ScanResponse { content: string; format: string; bounds?: scanBarcode.ScanCodeRect; }
interface PermissionResponse { camera: string; }

const FORMATS: Map<string, scanCore.ScanType> = new Map([
  ['QR_CODE', scanCore.ScanType.QR_CODE], ['UPC_A', scanCore.ScanType.UPC_A_CODE],
  ['UPC_E', scanCore.ScanType.UPC_E_CODE], ['EAN_8', scanCore.ScanType.EAN8_CODE],
  ['EAN_13', scanCore.ScanType.EAN13_CODE], ['CODE_39', scanCore.ScanType.CODE39_CODE],
  ['CODE_93', scanCore.ScanType.CODE93_CODE], ['CODE_128', scanCore.ScanType.CODE128_CODE],
  ['CODABAR', scanCore.ScanType.CODABAR_CODE], ['ITF', scanCore.ScanType.ITF14_CODE],
  ['AZTEC', scanCore.ScanType.AZTEC_CODE], ['DATA_MATRIX', scanCore.ScanType.DATAMATRIX_CODE],
  ['PDF_417', scanCore.ScanType.PDF417_CODE]
]);

export class Barcode implements NativePlugin {
  private context: common.UIAbilityContext;
  private scanning: boolean = false;
  private active: boolean = true;
  constructor(context: common.UIAbilityContext) { this.context = context; }
  async invoke(command: string, payload: string): Promise<string> {
    if (!this.active) throw new Error('Ability has been destroyed');
    const manager = abilityAccessCtrl.createAtManager();
    if (command === 'checkPermissions' || command === 'requestPermissions') {
      let granted: boolean;
      if (command === 'requestPermissions') {
        const result = await manager.requestPermissionsFromUser(this.context, ['ohos.permission.CAMERA']);
        granted = result.authResults.length === 1 && result.authResults[0] === 0;
      } else {
        granted = await manager.checkAccessToken(this.context.applicationInfo.accessTokenId, 'ohos.permission.CAMERA') === abilityAccessCtrl.GrantStatus.PERMISSION_GRANTED;
      }
      const response: PermissionResponse = { camera: granted ? 'granted' : 'prompt' };
      if (command === 'requestPermissions' && !granted) response.camera = 'denied';
      return JSON.stringify(response);
    }
    if (command === 'open_app_settings') {
      await this.context.startAbility({ action: 'ohos.settings.appInfo', parameters: { 'settingsParamBundleName': this.context.applicationInfo.name } });
      return 'null';
    }
    if (command === 'cancel') {
      if (!this.scanning) return 'null';
      throw new Error('ScanKit system UI must be cancelled using its native Back/Cancel control');
    }
    if (command !== 'scan') throw new Error(`Unsupported scanner command: ${command}`);
    if (!canIUse('SystemCapability.Multimedia.Scan.ScanBarcode')) throw new Error('ScanKit is unavailable on this device');
    if (this.scanning) throw new Error('A scanner is already open');
    const request = JSON.parse(payload) as ScanRequest;
    if (request.windowed || request.cameraDirection === 'front') throw new Error('ScanKit supports fullscreen back-camera scanning');
    const types: scanCore.ScanType[] = [];
    for (const format of request.formats ?? []) {
      const type = FORMATS.get(format);
      if (type === undefined) throw new Error(`Unsupported barcode format: ${format}`);
      types.push(type);
    }
    const options: scanBarcode.ScanOptions = { scanTypes: types.length > 0 ? types : [scanCore.ScanType.ALL], enableMultiMode: false, enableAlbum: true };
    this.scanning = true;
    try {
      const result = await scanBarcode.startScanForResult(this.context, options);
      if (!this.active) throw new Error('Ability was destroyed while scanning');
      let format = '';
      FORMATS.forEach((type: scanCore.ScanType, name: string): void => { if (type === result.scanType) format = name; });
      if (format.length === 0) throw new Error(`Unmapped barcode format: ${result.scanType}`);
      const response: ScanResponse = { content: result.originalValue, format, bounds: result.scanCodeRect };
      return JSON.stringify(response);
    } finally { this.scanning = false; }
  }
  close(): void { this.active = false; }
}
