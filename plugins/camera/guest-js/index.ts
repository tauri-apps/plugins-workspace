// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
import { invoke } from '@tauri-apps/api/tauri'

export enum Source {
  Prompt = 'PROMPT',
  Camera = 'CAMERA',
  Photos = 'PHOTOS'
}

export enum ResultType {
  Uri = 'uri',
  Base64 = 'base64',
  DataUrl = 'dataUrl'
}

export enum CameraDirection {
  Rear = 'REAR',
  Front = 'FRONT'
}

export interface ImageOptions {
  quality?: number
  allowEditing?: boolean
  resultType?: ResultType
  saveToGallery?: boolean
  width?: number
  height?: number
  correctOrientation?: boolean
  source?: Source
  direction?: CameraDirection
  presentationStyle?: 'fullscreen' | 'popover'
  promptLabelHeader?: string
  promptLabelCancel?: string
  promptLabelPhoto?: string
  promptLabelPicture?: string
}

export interface Image {
  data: string
  assetUrl?: string
  format: string
  saved: boolean
  exif: unknown
}

export async function getPhoto(options?: ImageOptions): Promise<Image> {
  return await invoke('plugin:camera|getPhoto', { ...options })
}
