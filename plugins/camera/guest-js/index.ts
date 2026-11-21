// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Take a photo with the camera or pick images from the gallery on Android and iOS.
 *
 * @module
 */

import {
  invoke,
  checkPermissions as checkPluginPermissions,
  type PermissionState
} from '@tauri-apps/api/core'

export type { PermissionState }

/**
 * Where {@link getPhoto} gets the image from.
 *
 * @since 2.0.0
 */
export enum Source {
  /** Ask the user whether to take a picture or to pick one from the gallery. */
  Prompt = 'prompt',
  /** Take a picture with the camera app. */
  Camera = 'camera',
  /** Pick an image from the gallery. */
  Photos = 'photos'
}

/**
 * How the image is returned in {@link Image.data}.
 *
 * @since 2.0.0
 */
export enum ResultType {
  /** The base64 encoded JPEG. */
  Base64 = 'base64',
  /** A `data:image/jpeg;base64,` URL. */
  DataUrl = 'dataUrl',
  /**
   * The `file://` URI of a JPEG file written to the app's cache (Android) or temporary (iOS)
   * directory.
   */
  Uri = 'uri'
}

/**
 * The camera that is selected when the camera app opens.
 *
 * @since 2.0.0
 */
export enum CameraDirection {
  /** The camera on the back of the device. */
  Rear = 'rear',
  /** The camera facing the user. */
  Front = 'front'
}

/**
 * How the camera and gallery are presented on iOS. `popover` only differs from `fullscreen`
 * on iPad.
 *
 * @since 2.0.0
 */
export type PresentationStyle = 'fullscreen' | 'popover'

/**
 * Options for {@link getPhoto}.
 *
 * @since 2.0.0
 */
export interface ImageOptions {
  /** The JPEG quality, from 0 to 100. Defaults to 90. */
  quality?: number
  /**
   * Let the user edit (iOS: crop) the image before it is returned. Defaults to `false`.
   *
   * On Android this opens the image in an external editor app and rejects if none is installed.
   */
  allowEditing?: boolean
  /** How the image is returned. Defaults to {@link ResultType.Base64}. */
  resultType?: ResultType
  /**
   * Also save pictures taken with the camera to the device's gallery. Defaults to `false`.
   *
   * Whether that worked is reported in {@link Image.saved}.
   */
  saveToGallery?: boolean
  /** The maximum width of the image in pixels. The aspect ratio is preserved. */
  width?: number
  /** The maximum height of the image in pixels. The aspect ratio is preserved. */
  height?: number
  /**
   * Rotate the image so it is upright instead of relying on its EXIF orientation.
   * Defaults to `true`.
   */
  correctOrientation?: boolean
  /** Where to get the image from. Defaults to {@link Source.Prompt}. */
  source?: Source
  /** The camera to open. iOS only. Defaults to {@link CameraDirection.Rear}. */
  direction?: CameraDirection
  /** How the camera and gallery are presented. iOS only. Defaults to `fullscreen`. */
  presentationStyle?: PresentationStyle
  /** The title of the {@link Source.Prompt} dialog. Defaults to "Photo". */
  promptLabelHeader?: string
  /**
   * The label of the cancel button of the {@link Source.Prompt} dialog. iOS only.
   * Defaults to "Cancel".
   */
  promptLabelCancel?: string
  /**
   * The label of the {@link Source.Prompt} option that opens the gallery.
   * Defaults to "From Photos".
   */
  promptLabelPhoto?: string
  /**
   * The label of the {@link Source.Prompt} option that opens the camera.
   * Defaults to "Take Picture".
   */
  promptLabelPicture?: string
}

/**
 * Options for {@link pickImages}.
 *
 * @since 2.0.0
 */
export interface GalleryOptions {
  /** The JPEG quality, from 0 to 100. Defaults to 90. */
  quality?: number
  /** The maximum width of the images in pixels. The aspect ratio is preserved. */
  width?: number
  /** The maximum height of the images in pixels. The aspect ratio is preserved. */
  height?: number
  /**
   * Rotate the images so they are upright instead of relying on their EXIF orientation.
   * Defaults to `true`.
   */
  correctOrientation?: boolean
  /**
   * The maximum number of images the user can select. Defaults to 0, which means no limit.
   *
   * Ignored on Android 12 and older.
   */
  limit?: number
  /** How the gallery is presented. iOS only. Defaults to `fullscreen`. */
  presentationStyle?: PresentationStyle
}

/**
 * An image returned by the camera or picked from the gallery.
 *
 * @since 2.0.0
 */
export interface Image {
  /** The image, in the representation selected with {@link ImageOptions.resultType}. */
  data: string
  /**
   * The URL to load the image from the webview with the `asset` protocol, which must be enabled
   * and scoped to the file in the app's configuration. Only set for {@link ResultType.Uri}.
   */
  assetUrl?: string
  /** The format of the image. Currently always `jpeg`. */
  format: string
  /** Whether the image was saved to the gallery, see {@link ImageOptions.saveToGallery}. */
  saved: boolean
  /** The EXIF metadata of the image. The keys are platform specific. */
  exif: Record<string, unknown>
}

/**
 * The permissions of the camera plugin.
 *
 * - `camera`: access to the camera.
 * - `photos`: saving pictures to the gallery. Picking images from the gallery needs no permission.
 *
 * @since 2.0.0
 */
export type PermissionType = 'camera' | 'photos'

/**
 * The state of the permissions of the camera plugin.
 *
 * @since 2.0.0
 */
export interface PermissionStatus {
  /**
   * Access to the camera.
   *
   * On Android the camera app is used to take the picture, which only needs the `CAMERA`
   * permission when the app declares it in its manifest. It is reported as granted otherwise.
   */
  camera: PermissionState
  /**
   * Saving pictures to the gallery, see {@link ImageOptions.saveToGallery}.
   *
   * Always granted on Android 10 and newer.
   */
  photos: PermissionState
}

/**
 * Asks the user to take a picture with the camera app or to pick an image from the gallery,
 * depending on {@link ImageOptions.source}.
 *
 * Rejects if the user denied a required permission, the device has no camera or the image could
 * not be processed.
 *
 * @example
 * ```typescript
 * import { getPhoto, ResultType, Source } from '@tauri-apps/plugin-camera'
 * const image = await getPhoto({
 *   source: Source.Camera,
 *   resultType: ResultType.DataUrl
 * })
 * if (image) {
 *   document.querySelector('img').src = image.data
 * }
 * ```
 *
 * @returns The image, or `null` if the user cancelled.
 *
 * @since 2.0.0
 */
export async function getPhoto(options?: ImageOptions): Promise<Image | null> {
  return await invoke('plugin:camera|get_photo', { ...options })
}

/**
 * Asks the user to pick any number of images from the gallery.
 *
 * The images are written to JPEG files, so {@link Image.data} is a `file://` URI as with
 * {@link ResultType.Uri}.
 *
 * @example
 * ```typescript
 * import { pickImages } from '@tauri-apps/plugin-camera'
 * const images = await pickImages({ limit: 5 })
 * for (const image of images) {
 *   console.log(image.data)
 * }
 * ```
 *
 * @returns The images, which is an empty array if the user cancelled.
 *
 * @since 2.0.0
 */
export async function pickImages(options?: GalleryOptions): Promise<Image[]> {
  return await invoke<{ images: Image[] }>('plugin:camera|pick_images', {
    ...options
  }).then((r) => r.images)
}

/**
 * Returns the state of the plugin's permissions without prompting the user.
 *
 * @example
 * ```typescript
 * import { checkPermissions } from '@tauri-apps/plugin-camera'
 * const { camera } = await checkPermissions()
 * ```
 *
 * @since 2.0.0
 */
export async function checkPermissions(): Promise<PermissionStatus> {
  return await checkPluginPermissions('camera')
}

/**
 * Prompts the user for the given permissions, or all of them if omitted, and returns the
 * resulting state.
 *
 * {@link getPhoto} requests the permissions it needs, so calling this is only required to ask for
 * them ahead of time.
 *
 * @example
 * ```typescript
 * import { requestPermissions } from '@tauri-apps/plugin-camera'
 * const { camera } = await requestPermissions(['camera'])
 * ```
 *
 * @since 2.0.0
 */
export async function requestPermissions(
  permissions?: PermissionType[]
): Promise<PermissionStatus> {
  return await invoke('plugin:camera|request_permissions', { permissions })
}
