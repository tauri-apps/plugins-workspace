// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import UIKit

// MARK: - Public

public enum CameraSource: String, Decodable {
  case prompt
  case camera
  case photos
}

public enum CameraDirection: String, Decodable {
  case rear
  case front
}

public enum CameraResultType: String, Decodable {
  case base64
  case uri
  case dataURL = "dataUrl"
}

public enum CameraPresentationStyle: String, Decodable {
  case fullscreen
  case popover
}

/// The arguments of `getPhoto` and `pickImages`.
struct ImageOptions: Decodable {
  var quality: Double?
  var allowEditing: Bool?
  var resultType: CameraResultType?
  var saveToGallery: Bool?
  var width: Double?
  var height: Double?
  var correctOrientation: Bool?
  var source: CameraSource?
  var direction: CameraDirection?
  var presentationStyle: CameraPresentationStyle?
  var promptLabelHeader: String?
  var promptLabelCancel: String?
  var promptLabelPhoto: String?
  var promptLabelPicture: String?
  var limit: Int?
}

struct RequestPermissionsArgs: Decodable {
  var permissions: [CameraPermissionType]?
}

struct CameraPromptText {
  let title: String
  let photoAction: String
  let cameraAction: String
  let cancelAction: String

  init(
    title: String? = nil, photoAction: String? = nil, cameraAction: String? = nil,
    cancelAction: String? = nil
  ) {
    self.title = title ?? "Photo"
    self.photoAction = photoAction ?? "From Photos"
    self.cameraAction = cameraAction ?? "Take Picture"
    self.cancelAction = cancelAction ?? "Cancel"
  }
}

public struct CameraSettings {
  var source: CameraSource = CameraSource.prompt
  var direction: CameraDirection = CameraDirection.rear
  var resultType = CameraResultType.base64
  var userPromptText = CameraPromptText()
  var jpegQuality: CGFloat = 0.9
  var width: CGFloat = 0
  var height: CGFloat = 0
  var allowEditing = false
  var shouldResize = false
  var shouldCorrectOrientation = true
  var saveToGallery = false
  var presentationStyle = UIModalPresentationStyle.fullScreen
  /// The maximum number of images that can be picked, 0 means no limit.
  var limit = 0

  init() {}

  init(_ options: ImageOptions) {
    source = options.source ?? .prompt
    direction = options.direction ?? .rear
    resultType = options.resultType ?? .base64
    userPromptText = CameraPromptText(
      title: options.promptLabelHeader,
      photoAction: options.promptLabelPhoto,
      cameraAction: options.promptLabelPicture,
      cancelAction: options.promptLabelCancel)
    jpegQuality = min(max(CGFloat(options.quality ?? 90) / 100.0, 0), 1)
    // the image is only resized if a dimension was provided
    width = max(CGFloat(options.width ?? 0), 0)
    height = max(CGFloat(options.height ?? 0), 0)
    shouldResize = width > 0 || height > 0
    allowEditing = options.allowEditing ?? false
    shouldCorrectOrientation = options.correctOrientation ?? true
    saveToGallery = options.saveToGallery ?? false
    presentationStyle = options.presentationStyle == .popover ? .popover : .fullScreen
    limit = max(options.limit ?? 0, 0)
  }
}

public struct CameraResult {
    let image: UIImage?
    let metadata: [AnyHashable: Any]
}

// MARK: - Internal

internal enum CameraPermissionType: String, CaseIterable, Decodable {
    case camera
    case photos

    /// The Info.plist key the system requires before it prompts for this permission.
    var usageDescriptionKey: CameraPropertyListKeys {
        switch self {
        case .camera:
            return .cameraUsage
        case .photos:
            return .photoLibraryAddUsage
        }
    }
}

internal enum CameraPropertyListKeys: String, CaseIterable {
    case photoLibraryAddUsage = "NSPhotoLibraryAddUsageDescription"
    case cameraUsage = "NSCameraUsageDescription"

    var link: String {
        switch self {
        case .photoLibraryAddUsage:
            return "https://developer.apple.com/library/content/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html#//apple_ref/doc/uid/TP40009251-SW73"
        case .cameraUsage:
            return "https://developer.apple.com/library/content/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html#//apple_ref/doc/uid/TP40009251-SW24"
        }
    }

    var missingMessage: String {
        return "You are missing \(self.rawValue) in your Info.plist file." +
            " It is required by the camera plugin. Learn more: \(self.link)"
    }
}

internal struct PhotoFlags: OptionSet {
    let rawValue: Int

    static let edited = PhotoFlags(rawValue: 1 << 0)
    static let gallery = PhotoFlags(rawValue: 1 << 1)

    static let all: PhotoFlags = [.edited, .gallery]
}

internal struct ProcessedImage {
    var image: UIImage
    var metadata: [String: Any]
    var flags: PhotoFlags = []

    /// The EXIF metadata, limited to the values that can be represented as JSON.
    var exifData: [String: Any] {
        var exifData = metadata["{Exif}"] as? [String: Any]
        exifData?["Orientation"] = metadata["Orientation"]
        exifData?["GPS"] = metadata["{GPS}"]
        return ProcessedImage.jsonValue(exifData ?? [:]) as? [String: Any] ?? [:]
    }

    private static func jsonValue(_ value: Any) -> Any? {
        switch value {
        case let string as String:
            return string
        case let number as NSNumber:
            // NaN and infinity are not valid JSON
            return number.doubleValue.isFinite ? number : nil
        case let array as [Any]:
            return array.compactMap { jsonValue($0) }
        case let dictionary as [String: Any]:
            return dictionary.compactMapValues { jsonValue($0) }
        default:
            // binary data such as maker notes
            return nil
        }
    }

    mutating func overwriteMetadataOrientation(to orientation: Int) {
        replaceDictionaryOrientation(atNode: &metadata, to: orientation)
    }

    func replaceDictionaryOrientation(atNode node: inout [String: Any], to orientation: Int) {
        for key in node.keys {
            if key == "Orientation", (node[key] as? Int) != nil {
                node[key] = orientation
            } else if var child = node[key] as? [String: Any] {
                replaceDictionaryOrientation(atNode: &child, to: orientation)
                node[key] = child
            }
        }
    }

    func generateJPEG(with quality: CGFloat) -> Data? {
        // convert the UIImage to a jpeg
        guard let data = self.image.jpegData(compressionQuality: quality) else {
            return nil
        }
        // define our jpeg data as an image source and get its type
        guard let source = CGImageSourceCreateWithData(data as CFData, nil), let type = CGImageSourceGetType(source) else {
            return data
        }
        // allocate an output buffer and create the destination to receive the new data
        guard let output = NSMutableData(capacity: data.count), let destination = CGImageDestinationCreateWithData(output, type, 1, nil) else {
            return data
        }
        // pipe the source into the destination while overwriting the metadata, this encodes the metadata information into the image
        CGImageDestinationAddImageFromSource(destination, source, 0, self.metadata as CFDictionary)
        // finish
        guard CGImageDestinationFinalize(destination) else {
            return data
        }
        return output as Data
    }
}
