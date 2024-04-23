// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import AVFoundation
import Tauri
import UIKit
import WebKit

struct ScanOptions: Decodable {
  var formats: [SupportedFormat]?
  let windowed: Bool?
  let cameraDirection: String?
}

enum SupportedFormat: String, CaseIterable, Decodable {
  // UPC_A not supported
  case UPC_E
  case EAN_8
  case EAN_13
  case CODE_39
  case CODE_93
  case CODE_128
  // CODABAR not supported
  case ITF
  case AZTEC
  case DATA_MATRIX
  case PDF_417
  case QR_CODE

  var value: AVMetadataObject.ObjectType {
    switch self {
    case .UPC_E: return AVMetadataObject.ObjectType.upce
    case .EAN_8: return AVMetadataObject.ObjectType.ean8
    case .EAN_13: return AVMetadataObject.ObjectType.ean13
    case .CODE_39: return AVMetadataObject.ObjectType.code39
    case .CODE_93: return AVMetadataObject.ObjectType.code93
    case .CODE_128: return AVMetadataObject.ObjectType.code128
    case .ITF: return AVMetadataObject.ObjectType.interleaved2of5
    case .AZTEC: return AVMetadataObject.ObjectType.aztec
    case .DATA_MATRIX: return AVMetadataObject.ObjectType.dataMatrix
    case .PDF_417: return AVMetadataObject.ObjectType.pdf417
    case .QR_CODE: return AVMetadataObject.ObjectType.qr
    }
  }
}

enum CaptureError: Error {
  case backCameraUnavailable
  case frontCameraUnavailable
  case couldNotCaptureInput(error: NSError)
}

class BarcodeScannerPlugin: Plugin, AVCaptureMetadataOutputObjectsDelegate {
  var webView: WKWebView!
  var cameraView: CameraView!
  var captureSession: AVCaptureSession?
  var captureVideoPreviewLayer: AVCaptureVideoPreviewLayer?
  var metaOutput: AVCaptureMetadataOutput?

  var currentCamera = 0
  var frontCamera: AVCaptureDevice?
  var backCamera: AVCaptureDevice?

  var isScanning = false

  var windowed = false
  var previousBackgroundColor: UIColor? = UIColor.white

  var invoke: Invoke? = nil

  var scanFormats = [AVMetadataObject.ObjectType]()

  public override func load(webview: WKWebView) {
    self.webView = webview
    loadCamera()
  }

  private func loadCamera() {
    cameraView = CameraView(
      frame: CGRect(
        x: 0, y: 0, width: UIScreen.main.bounds.width, height: UIScreen.main.bounds.height))
    cameraView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
  }

  public func metadataOutput(
    _ captureOutput: AVCaptureMetadataOutput, didOutput metadataObjects: [AVMetadataObject],
    from connection: AVCaptureConnection
  ) {
    if metadataObjects.count == 0 || !self.isScanning {
      // while nothing is detected, or if scanning is false, do nothing.
      return
    }

    let found = metadataObjects[0] as! AVMetadataMachineReadableCodeObject
    if scanFormats.contains(found.type) {
      var jsObject: JsonObject = [:]

      jsObject["format"] = formatStringFromMetadata(found.type)
      if found.stringValue != nil {
        jsObject["content"] = found.stringValue
      }

      invoke?.resolve(jsObject)
      destroy()

    }
  }

  private func setupCamera(direction: String, windowed: Bool) {
    do {
      var cameraDirection = direction
      cameraView.backgroundColor = UIColor.clear
      if windowed {
        webView.superview?.insertSubview(cameraView, belowSubview: webView)
      } else {
        webView.superview?.insertSubview(cameraView, aboveSubview: webView)
      }

      let availableVideoDevices = discoverCaptureDevices()
      for device in availableVideoDevices {
        if device.position == AVCaptureDevice.Position.back {
          backCamera = device
        } else if device.position == AVCaptureDevice.Position.front {
          frontCamera = device
        }
      }

      // older iPods have no back camera
      if cameraDirection == "back" {
        if backCamera == nil {
          cameraDirection = "front"
        }
      } else {
        if frontCamera == nil {
          cameraDirection = "back"
        }
      }

      let input: AVCaptureDeviceInput
      input = try createCaptureDeviceInput(
        cameraDirection: cameraDirection, backCamera: backCamera, frontCamera: frontCamera)
      captureSession = AVCaptureSession()
      captureSession!.addInput(input)
      metaOutput = AVCaptureMetadataOutput()
      captureSession!.addOutput(metaOutput!)
      metaOutput!.setMetadataObjectsDelegate(self, queue: DispatchQueue.main)
      captureVideoPreviewLayer = AVCaptureVideoPreviewLayer(session: captureSession!)
      cameraView.addPreviewLayer(captureVideoPreviewLayer)

      self.windowed = windowed
      if windowed {
        self.previousBackgroundColor = self.webView.backgroundColor
        self.webView.isOpaque = false
        self.webView.backgroundColor = UIColor.clear
        self.webView.scrollView.backgroundColor = UIColor.clear
      }
    } catch CaptureError.backCameraUnavailable {
      //
    } catch CaptureError.frontCameraUnavailable {
      //
    } catch CaptureError.couldNotCaptureInput {
      //
    } catch {
      //
    }
  }

  private func dismantleCamera() {
    if self.captureSession != nil {
      self.captureSession!.stopRunning()
      self.cameraView.removePreviewLayer()
      self.captureVideoPreviewLayer = nil
      self.metaOutput = nil
      self.captureSession = nil
      self.frontCamera = nil
      self.backCamera = nil
    }

    self.isScanning = false
  }

  private func destroy() {
    dismantleCamera()
    invoke = nil
    if windowed {
      let backgroundColor = previousBackgroundColor ?? UIColor.white
      webView.isOpaque = true
      webView.backgroundColor = backgroundColor
      webView.scrollView.backgroundColor = backgroundColor
    }
  }

  private func getPermissionState() -> String {
    var permissionState: String

    switch AVCaptureDevice.authorizationStatus(for: .video) {
    case .authorized:
      permissionState = "granted"
    case .denied:
      permissionState = "denied"
    default:
      permissionState = "prompt"
    }

    return permissionState
  }

  @objc override func checkPermissions(_ invoke: Invoke) {
    let permissionState = getPermissionState()
    invoke.resolve(["camera": permissionState])
  }

  @objc override func requestPermissions(_ invoke: Invoke) {
    let state = getPermissionState()
    if state == "prompt" {
      AVCaptureDevice.requestAccess(for: .video) { (authorized) in
        invoke.resolve(["camera": authorized ? "granted" : "denied"])
      }
    } else {
      invoke.resolve(["camera": state])
    }
  }

  @objc func openAppSettings(_ invoke: Invoke) {
    guard let settingsUrl = URL(string: UIApplication.openSettingsURLString) else {
      return
    }

    DispatchQueue.main.async {
      if UIApplication.shared.canOpenURL(settingsUrl) {
        UIApplication.shared.open(
          settingsUrl,
          completionHandler: { (success) in
            invoke.resolve()
          })
      }
    }
  }

  private func runScanner(_ invoke: Invoke, args: ScanOptions) {
    scanFormats = [AVMetadataObject.ObjectType]()

    (args.formats ?? []).forEach { format in
      scanFormats.append(format.value)
    }

    if scanFormats.count == 0 {
      for supportedFormat in SupportedFormat.allCases {
        scanFormats.append(supportedFormat.value)
      }
    }

    self.metaOutput!.metadataObjectTypes = self.scanFormats
    self.captureSession!.startRunning()

    self.isScanning = true
  }

  @objc private func scan(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ScanOptions.self)

    self.invoke = invoke

    var iOS14min: Bool = false
    if #available(iOS 14.0, *) { iOS14min = true }
    if !iOS14min && self.getPermissionState() != "granted" {
      var authorized = false
      AVCaptureDevice.requestAccess(for: .video) { (isAuthorized) in
        authorized = isAuthorized
      }
      if !authorized {
        invoke.reject("denied by the user")
        return
      }
    }

    DispatchQueue.main.async { [self] in
      self.loadCamera()
      self.dismantleCamera()
      self.setupCamera(
        direction: args.cameraDirection ?? "back",
        windowed: args.windowed ?? false
      )
      self.runScanner(invoke, args: args)
    }
  }

  @objc private func cancel(_ invoke: Invoke) {
    self.invoke?.reject("cancelled")

    destroy()
    invoke.resolve()
  }
}

@_cdecl("init_plugin_barcode_scanner")
func initPlugin() -> Plugin {
  return BarcodeScannerPlugin()
}
