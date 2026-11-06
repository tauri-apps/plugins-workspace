// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import common from '@ohos.app.ability.common';
import notificationManager from '@ohos.notificationManager';
import notification from '@ohos.notification';
import { BusinessError } from '@ohos.base';
import { NativePlugin } from '../NativePlugin';

interface PermissionResponse { permissionState: string; }
interface NotificationRequest {
  id: number; title?: string; body?: string; summary?: string;
  schedule?: Object; actionTypeId?: string; attachments?: Object[];
  sound?: string; silent?: boolean;
}
interface NotificationId { id: number; }
interface RemoveRequest { notifications?: NotificationId[]; }
interface ActiveNotification { id: number; title?: string; body?: string; }

export class Notification implements NativePlugin {
  private context: common.UIAbilityContext;
  constructor(context: common.UIAbilityContext) { this.context = context; }
  async invoke(command: string, payload: string): Promise<string> {
    switch (command) {
      case 'checkPermissions': {
        const result: PermissionResponse = { permissionState: await notificationManager.isNotificationEnabled() ? 'granted' : 'prompt' };
        return JSON.stringify(result);
      }
      case 'requestPermissions': {
        try { await notificationManager.requestEnableNotification(this.context); }
        catch (error) {
          if ((error as BusinessError).code !== 1600004) throw error as Error;
        }
        const result: PermissionResponse = { permissionState: await notificationManager.isNotificationEnabled() ? 'granted' : 'denied' };
        return JSON.stringify(result);
      }
      case 'show': {
        const request = JSON.parse(payload) as NotificationRequest;
        if (request.schedule || request.actionTypeId || request.sound || request.silent || (request.attachments?.length ?? 0) > 0) {
          throw new Error('Scheduled notifications, actions, custom sounds and attachments are not supported by this OHOS backend');
        }
        const nativeRequest: notificationManager.NotificationRequest = {
          id: request.id,
          content: {
            contentType: notification.ContentType.NOTIFICATION_CONTENT_BASIC_TEXT,
            normal: { title: request.title ?? '', text: request.body ?? '', additionalText: request.summary ?? '' }
          }
        };
        await notificationManager.publish(nativeRequest);
        return JSON.stringify(request.id);
      }
      case 'removeActive': {
        if (payload === 'null') await notificationManager.cancelAll();
        else {
          const request = JSON.parse(payload) as RemoveRequest;
          if (request.notifications === undefined) throw new Error('Notification IDs are required');
          for (const notification of request.notifications) await notificationManager.cancel(notification.id);
        }
        return 'null';
      }
      case 'getActive': {
        const active = await notificationManager.getActiveNotifications();
        const result: ActiveNotification[] = active.map((item: notificationManager.NotificationRequest): ActiveNotification => {
          return { id: item.id ?? 0, title: item.content?.normal?.title, body: item.content?.normal?.text };
        });
        return JSON.stringify(result);
      }
      default: throw new Error(`Unsupported notification command: ${command}`);
    }
  }
  close(): void {}
}
