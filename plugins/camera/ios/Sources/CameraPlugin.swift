// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import AVFoundation
import ImageIO
import Photos
import PhotosUI
import Tauri
import UIKit
import WebKit

public class CameraPlugin: Plugin {
  // the request waiting for the user, see `resolve`, `reject` and `cancel`
  private var invoke: Invoke?
  private var settings = CameraSettings()
  // whether the pending request is `pickImages`
  private var multiple = false
  // keeps the saver alive until the system reports the result
  private var imageSaver: ImageSaver?

  @objc override public func checkPermissions(_ invoke: Invoke) {
    let result: JsonObject = [
      CameraPermissionType.camera.rawValue:
        AVCaptureDevice.authorizationStatus(for: .video).authorizationState,
      CameraPermissionType.photos.rawValue: photosAuthorizationStatus().authorizationState,
    ]
    invoke.resolve(result)
  }

  @objc override public func requestPermissions(_ invoke: Invoke) {
    let permissions: [CameraPermissionType]
    do {
      permissions =
        try invoke.parseArgs(RequestPermissionsArgs.self).permissions
        ?? CameraPermissionType.allCases
    } catch {
      invoke.reject(error.localizedDescription)
      return
    }

    // the system terminates the app if it prompts without the usage description
    for permission in permissions {
      if let message = missingUsageDescription(permission.usageDescriptionKey) {
        invoke.reject(message)
        return
      }
    }

    let group = DispatchGroup()
    for permission in permissions {
      group.enter()
      switch permission {
      case .camera:
        AVCaptureDevice.requestAccess(for: .video) { _ in
          group.leave()
        }
      case .photos:
        if #available(iOS 14, *) {
          PHPhotoLibrary.requestAuthorization(for: .addOnly) { _ in
            group.leave()
          }
        } else {
          PHPhotoLibrary.requestAuthorization { _ in
            group.leave()
          }
        }
      }
    }
    group.notify(queue: DispatchQueue.main) { [weak self] in
      self?.checkPermissions(invoke)
    }
  }

  @objc public func getPhoto(_ invoke: Invoke) throws {
    let settings = CameraSettings(try invoke.parseArgs(ImageOptions.self))

    if settings.saveToGallery,
      let message = missingUsageDescription(.photoLibraryAddUsage)
    {
      invoke.reject(message)
      return
    }

    DispatchQueue.main.async {
      self.start(invoke, settings: settings, multiple: false)
      switch settings.source {
      case .prompt:
        self.showPrompt()
      case .camera:
        self.showCamera()
      case .photos:
        self.showPhotos()
      }
    }
  }

  @objc public func pickImages(_ invoke: Invoke) throws {
    var settings = CameraSettings(try invoke.parseArgs(ImageOptions.self))
    // the images are always returned as files
    settings.resultType = .uri
    settings.saveToGallery = false
    settings.allowEditing = false

    DispatchQueue.main.async {
      self.start(invoke, settings: settings, multiple: true)
      self.showPhotos()
    }
  }

  /// Makes `invoke` the pending request. A request that is still pending is cancelled.
  private func start(_ invoke: Invoke, settings: CameraSettings, multiple: Bool) {
    cancel()
    self.invoke = invoke
    self.settings = settings
    self.multiple = multiple
  }

  private func resolve(_ data: JsonObject) {
    invoke?.resolve(data)
    invoke = nil
  }

  private func reject(_ message: String) {
    invoke?.reject(message)
    invoke = nil
  }

  /// Resolves the pending request as cancelled by the user: no image for `getPhoto`, an empty list
  /// for `pickImages`.
  private func cancel() {
    if multiple {
      let data: JsonObject = ["images": [JsonObject]()]
      invoke?.resolve(data)
    } else {
      invoke?.resolve()
    }
    invoke = nil
  }

  private func photosAuthorizationStatus() -> PHAuthorizationStatus {
    // the plugin only needs the library to save pictures: picking images needs no permission
    if #available(iOS 14, *) {
      return PHPhotoLibrary.authorizationStatus(for: .addOnly)
    } else {
      return PHPhotoLibrary.authorizationStatus()
    }
  }

  /// Returns an error message if the app's Info.plist does not have the given usage description.
  private func missingUsageDescription(_ key: CameraPropertyListKeys) -> String? {
    if Bundle.main.object(forInfoDictionaryKey: key.rawValue) == nil {
      return key.missingMessage
    }
    return nil
  }
}

// public delegate methods
extension CameraPlugin: UIImagePickerControllerDelegate, UINavigationControllerDelegate,
  UIPopoverPresentationControllerDelegate
{
  public func imagePickerControllerDidCancel(_ picker: UIImagePickerController) {
    picker.dismiss(animated: true)
    cancel()
  }

  public func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
    cancel()
  }

  public func imagePickerController(
    _ picker: UIImagePickerController,
    didFinishPickingMediaWithInfo info: [UIImagePickerController.InfoKey: Any]
  ) {
    picker.dismiss(animated: true) {
      if let processedImage = self.processImage(from: info) {
        self.returnProcessedImage(processedImage)
      } else {
        self.reject("Error processing image")
      }
    }
  }
}

@available(iOS 14, *)
extension CameraPlugin: PHPickerViewControllerDelegate {
  public func picker(_ picker: PHPickerViewController, didFinishPicking results: [PHPickerResult]) {
    picker.dismiss(animated: true, completion: nil)
    if results.isEmpty {
      cancel()
      return
    }

    // The images are loaded from their data instead of as UIImage to also get their metadata,
    // which otherwise is only available from the photo library, i.e. with its permission.
    let group = DispatchGroup()
    let lock = NSLock()
    var images = [ProcessedImage?](repeating: nil, count: results.count)
    for (index, result) in results.enumerated() {
      group.enter()
      result.itemProvider.loadDataRepresentation(forTypeIdentifier: "public.image") {
        [weak self] (data, _) in
        defer { group.leave() }
        guard let self = self, let data = data, let image = UIImage(data: data) else {
          return
        }
        var processedImage = self.processedImage(from: image, with: self.metadata(from: data))
        processedImage.flags = .gallery
        lock.lock()
        images[index] = processedImage
        lock.unlock()
      }
    }

    group.notify(queue: DispatchQueue.global(qos: .userInitiated)) { [weak self] in
      guard let self = self else {
        return
      }
      let loadedImages = images.compactMap { $0 }
      if loadedImages.count != results.count {
        self.reject("Error loading image")
      } else if self.multiple {
        self.returnImages(loadedImages)
      } else {
        self.returnProcessedImage(loadedImages[0])
      }
    }
  }
}

extension CameraPlugin {
  /// Encodes the image and resolves the pending `getPhoto` request with it.
  fileprivate func returnImage(_ processedImage: ProcessedImage, isSaved: Bool) {
    DispatchQueue.global(qos: .userInitiated).async {
      guard let jpeg = processedImage.generateJPEG(with: self.settings.jpegQuality) else {
        self.reject("Unable to convert image to jpeg")
        return
      }

      switch self.settings.resultType {
      case .uri:
        guard let result = self.fileResult(jpeg, processedImage, isSaved: isSaved) else {
          self.reject("Unable to write the image to a file")
          return
        }
        self.resolve(result)
      case .base64, .dataURL:
        var data = jpeg.base64EncodedString()
        if self.settings.resultType == .dataURL {
          data = "data:image/jpeg;base64," + data
        }
        let result: JsonObject = [
          "data": data,
          "exif": processedImage.exifData,
          "format": "jpeg",
          "saved": isSaved,
        ]
        self.resolve(result)
      }
    }
  }

  /// Encodes the images and resolves the pending `pickImages` request with them.
  fileprivate func returnImages(_ processedImages: [ProcessedImage]) {
    DispatchQueue.global(qos: .userInitiated).async {
      var images: [JsonObject] = []
      for processedImage in processedImages {
        guard let jpeg = processedImage.generateJPEG(with: self.settings.jpegQuality) else {
          self.reject("Unable to convert image to jpeg")
          return
        }
        guard let result = self.fileResult(jpeg, processedImage, isSaved: false) else {
          self.reject("Unable to write the image to a file")
          return
        }
        images.append(result)
      }
      let result: JsonObject = ["images": images]
      self.resolve(result)
    }
  }

  /// Writes the JPEG to a temporary file and returns the result describing it.
  fileprivate func fileResult(_ jpeg: Data, _ processedImage: ProcessedImage, isSaved: Bool)
    -> JsonObject?
  {
    guard let fileURL = try? saveTemporaryImage(jpeg),
      let webURL = manager.assetUrl(fromLocalURL: fileURL)
    else {
      return nil
    }
    return [
      "data": fileURL.absoluteString,
      "exif": processedImage.exifData,
      "assetUrl": webURL.absoluteString,
      "format": "jpeg",
      "saved": isSaved,
    ]
  }

  fileprivate func returnProcessedImage(_ processedImage: ProcessedImage) {
    if multiple {
      returnImages([processedImage])
      return
    }
    // images picked from the gallery are already there, unless they were edited
    if settings.saveToGallery
      && (processedImage.flags.contains(.edited) || !processedImage.flags.contains(.gallery))
    {
      DispatchQueue.main.async {
        self.imageSaver = ImageSaver(image: processedImage.image) { error in
          self.imageSaver = nil
          self.returnImage(processedImage, isSaved: error == nil)
        }
      }
    } else {
      returnImage(processedImage, isSaved: false)
    }
  }

  fileprivate func showPrompt() {
    // Build the action sheet
    let alert = UIAlertController(
      title: settings.userPromptText.title, message: nil,
      preferredStyle: UIAlertController.Style.actionSheet)
    alert.addAction(
      UIAlertAction(
        title: settings.userPromptText.photoAction, style: .default,
        handler: { [weak self] (_: UIAlertAction) in
          self?.showPhotos()
        }))

    alert.addAction(
      UIAlertAction(
        title: settings.userPromptText.cameraAction, style: .default,
        handler: { [weak self] (_: UIAlertAction) in
          self?.showCamera()
        }))

    alert.addAction(
      UIAlertAction(
        title: settings.userPromptText.cancelAction, style: .cancel,
        handler: { [weak self] (_: UIAlertAction) in
          self?.cancel()
        }))
    UIUtils.centerPopover(rootViewController: manager.viewController, popoverController: alert)
    present(alert)
  }

  fileprivate func present(_ controller: UIViewController) {
    guard let viewController = manager.viewController else {
      reject("Unable to present the view: the app has no view controller")
      return
    }
    viewController.present(controller, animated: true, completion: nil)
  }

  fileprivate func showCamera() {
    // check if we have a camera
    if manager.isSimEnvironment
      || !UIImagePickerController.isSourceTypeAvailable(UIImagePickerController.SourceType.camera)
    {
      reject("Camera not available on this device or while running in the Simulator")
      return
    }
    if let message = missingUsageDescription(.cameraUsage) {
      reject(message)
      return
    }
    // check for permission
    let authStatus = AVCaptureDevice.authorizationStatus(for: .video)
    if authStatus == .restricted || authStatus == .denied {
      reject("User denied access to camera")
      return
    }
    // we either already have permission or can prompt
    AVCaptureDevice.requestAccess(for: .video) { [weak self] granted in
      DispatchQueue.main.async {
        if granted {
          self?.presentCameraPicker()
        } else {
          self?.reject("User denied access to camera")
        }
      }
    }
  }

  fileprivate func showPhotos() {
    // neither picker needs a permission: the app only gets the images the user selects
    if #available(iOS 14, *) {
      presentPhotoPicker()
    } else {
      presentImagePicker()
    }
  }

  fileprivate func presentCameraPicker() {
    let picker = UIImagePickerController()
    picker.delegate = self
    picker.allowsEditing = self.settings.allowEditing
    // select the input
    picker.sourceType = .camera
    if settings.direction == .rear, UIImagePickerController.isCameraDeviceAvailable(.rear) {
      picker.cameraDevice = .rear
    } else if settings.direction == .front, UIImagePickerController.isCameraDeviceAvailable(.front)
    {
      picker.cameraDevice = .front
    }
    presentPicker(picker)
  }

  fileprivate func presentImagePicker() {
    let picker = UIImagePickerController()
    picker.delegate = self
    picker.allowsEditing = self.settings.allowEditing
    // select the input
    picker.sourceType = .photoLibrary
    presentPicker(picker)
  }

  @available(iOS 14, *)
  fileprivate func presentPhotoPicker() {
    var configuration = PHPickerConfiguration()
    // 0 means no limit
    configuration.selectionLimit = multiple ? settings.limit : 1
    configuration.filter = .images
    let picker = PHPickerViewController(configuration: configuration)
    picker.delegate = self
    presentPicker(picker)
  }

  fileprivate func presentPicker(_ picker: UIViewController) {
    picker.modalPresentationStyle = settings.presentationStyle
    if settings.presentationStyle == .popover {
      picker.popoverPresentationController?.delegate = self
      UIUtils.centerPopover(rootViewController: manager.viewController, popoverController: picker)
    }
    present(picker)
  }

  fileprivate func saveTemporaryImage(_ data: Data) throws -> URL {
    let url = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent(
      "photo-\(UUID().uuidString).jpg")
    try data.write(to: url, options: .atomic)
    return url
  }

  /// Reads the metadata (EXIF, GPS, orientation...) embedded in the image data.
  fileprivate func metadata(from data: Data) -> [String: Any] {
    let options = [kCGImageSourceShouldCache as String: kCFBooleanFalse] as CFDictionary
    guard let source = CGImageSourceCreateWithData(data as CFData, options),
      let metadata = CGImageSourceCopyPropertiesAtIndex(source, 0, options) as? [String: Any]
    else {
      return [:]
    }
    return metadata
  }

  fileprivate func processImage(from info: [UIImagePickerController.InfoKey: Any])
    -> ProcessedImage?
  {
    var selectedImage: UIImage?
    var flags: PhotoFlags = []
    // get the image
    if let edited = info[UIImagePickerController.InfoKey.editedImage] as? UIImage {
      selectedImage = edited  // use the edited version
      flags = flags.union([.edited])
    } else if let original = info[UIImagePickerController.InfoKey.originalImage] as? UIImage {
      selectedImage = original  // use the original version
    }
    guard let image = selectedImage else {
      return nil
    }
    var metadata: [String: Any] = [:]
    // get the image's metadata from the picker or from the photo album
    if let photoMetadata = info[UIImagePickerController.InfoKey.mediaMetadata] as? [String: Any] {
      metadata = photoMetadata
    } else {
      flags = flags.union([.gallery])
    }
    // only set if the app has access to the photo library
    if let asset = info[UIImagePickerController.InfoKey.phAsset] as? PHAsset {
      metadata = asset.imageData
    }
    // get the result
    var result = processedImage(from: image, with: metadata)
    result.flags = flags
    return result
  }

  fileprivate func processedImage(from image: UIImage, with metadata: [String: Any]?)
    -> ProcessedImage
  {
    var result = ProcessedImage(image: image, metadata: metadata ?? [:])
    // resizing the image only makes sense if we have real values to which to constrain it
    if settings.shouldResize, settings.width > 0 || settings.height > 0 {
      result.image = result.image.reformat(
        to: CGSize(width: settings.width, height: settings.height))
      result.overwriteMetadataOrientation(to: 1)
    } else if settings.shouldCorrectOrientation {
      // resizing implicitly reformats the image so this is only needed if we aren't resizing
      result.image = result.image.reformat()
      result.overwriteMetadataOrientation(to: 1)
    }
    return result
  }
}

@_cdecl("init_plugin_camera")
func initPlugin() -> Plugin {
  return CameraPlugin()
}
