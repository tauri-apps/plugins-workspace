// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import AVFoundation

func createCaptureDeviceInput(
  cameraDirection: String, backCamera: AVCaptureDevice?, frontCamera: AVCaptureDevice?
) throws
  -> AVCaptureDeviceInput
{
  var captureDevice: AVCaptureDevice
  if cameraDirection == "back" {
    if backCamera != nil {
      captureDevice = backCamera!
    } else {
      throw CaptureError.backCameraUnavailable
    }
  } else {
    if frontCamera != nil {
      captureDevice = frontCamera!
    } else {
      throw CaptureError.frontCameraUnavailable
    }
  }
  let captureDeviceInput: AVCaptureDeviceInput
  do {
    captureDeviceInput = try AVCaptureDeviceInput(device: captureDevice)
  } catch let error as NSError {
    throw CaptureError.couldNotCaptureInput(error: error)
  }
  return captureDeviceInput
}

func discoverCaptureDevices() -> [AVCaptureDevice] {
  if #available(iOS 13.0, *) {
    return AVCaptureDevice.DiscoverySession(
      deviceTypes: [
        .builtInTripleCamera, .builtInDualCamera, .builtInTelephotoCamera,
        .builtInTrueDepthCamera,
        .builtInUltraWideCamera, .builtInDualWideCamera, .builtInWideAngleCamera,
      ], mediaType: .video, position: .unspecified
    ).devices
  } else {
    return AVCaptureDevice.DiscoverySession(
      deviceTypes: [
        .builtInDualCamera, .builtInWideAngleCamera, .builtInTelephotoCamera,
        .builtInTrueDepthCamera,
      ], mediaType: .video, position: .unspecified
    ).devices
  }
}

func formatStringFromMetadata(_ type: AVMetadataObject.ObjectType) -> String {
  switch type {
  case AVMetadataObject.ObjectType.upce:
    return "UPC_E"
  case AVMetadataObject.ObjectType.ean8:
    return "EAN_8"
  case AVMetadataObject.ObjectType.ean13:
    return "EAN_13"
  case AVMetadataObject.ObjectType.code39:
    return "CODE_39"
  case AVMetadataObject.ObjectType.code93:
    return "CODE_93"
  case AVMetadataObject.ObjectType.code128:
    return "CODE_128"
  case AVMetadataObject.ObjectType.interleaved2of5:
    return "ITF"
  case AVMetadataObject.ObjectType.aztec:
    return "AZTEC"
  case AVMetadataObject.ObjectType.dataMatrix:
    return "DATA_MATRIX"
  case AVMetadataObject.ObjectType.pdf417:
    return "PDF_417"
  case AVMetadataObject.ObjectType.qr:
    return "QR_CODE"
  default:
    return type.rawValue
  }
}
