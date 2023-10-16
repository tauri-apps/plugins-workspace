// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import UIKit

// MARK: - Public

public enum CameraSource: String {
    case prompt = "PROMPT"
    case camera = "CAMERA"
    case photos = "PHOTOS"
}

public enum CameraDirection: String {
    case rear = "REAR"
    case front = "FRONT"
}

public enum CameraResultType: String {
    case base64
    case uri
    case dataURL = "dataUrl"
}

struct CameraPromptText {
    let title: String
    let photoAction: String
    let cameraAction: String
    let cancelAction: String

    init(title: String? = nil, photoAction: String? = nil, cameraAction: String? = nil, cancelAction: String? = nil) {
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
    var jpegQuality: CGFloat = 1.0
    var width: CGFloat = 0
    var height: CGFloat = 0
    var allowEditing = false
    var shouldResize = false
    var shouldCorrectOrientation = true
    var saveToGallery = false
    var presentationStyle = UIModalPresentationStyle.fullScreen
}

public struct CameraResult {
    let image: UIImage?
    let metadata: [AnyHashable: Any]
}

// MARK: - Internal

internal enum CameraPermissionType: String, CaseIterable {
    case camera
    case photos
}

internal enum CameraPropertyListKeys: String, CaseIterable {
    case photoLibraryAddUsage = "NSPhotoLibraryAddUsageDescription"
    case photoLibraryUsage = "NSPhotoLibraryUsageDescription"
    case cameraUsage = "NSCameraUsageDescription"

    var link: String {
        switch self {
        case .photoLibraryAddUsage:
            return "https://developer.apple.com/library/content/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html#//apple_ref/doc/uid/TP40009251-SW73"
        case .photoLibraryUsage:
            return "https://developer.apple.com/library/content/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html#//apple_ref/doc/uid/TP40009251-SW17"
        case .cameraUsage:
            return "https://developer.apple.com/library/content/documentation/General/Reference/InfoPlistKeyReference/Articles/CocoaKeys.html#//apple_ref/doc/uid/TP40009251-SW24"
        }
    }

    var missingMessage: String {
        return "You are missing \(self.rawValue) in your Info.plist file." +
            " Camera will not function without it. Learn more: \(self.link)"
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

    var exifData: [String: Any] {
        var exifData = metadata["{Exif}"] as? [String: Any]
        exifData?["Orientation"] = metadata["Orientation"]
        exifData?["GPS"] = metadata["{GPS}"]
        return exifData ?? [:]
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