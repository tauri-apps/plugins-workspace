// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import MobileCoreServices
import Photos
import PhotosUI
import SwiftRs
import Tauri
import UIKit
import WebKit

enum FilePickerEvent {
  case selected([URL])
  case cancelled
  case error(String)
}

struct MessageDialogOptions: Decodable {
  let title: String?
  let message: String
  var okButtonLabel = "OK"
  var cancelButtonLabel = "Cancel"
}

struct Filter: Decodable {
  var extensions: [String] = []
}

struct FilePickerOptions: Decodable {
  var multiple = false
  var readData = false
  var filters: [Filter] = []
}

class DialogPlugin: Plugin {

  var filePickerController: FilePickerController!
  var pendingInvoke: Invoke? = nil
  var pendingInvokeArgs: FilePickerOptions? = nil

  override init() {
    super.init()
    filePickerController = FilePickerController(self)
  }

  @objc public func showFilePicker(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(FilePickerOptions.self)

    let parsedTypes = parseFiltersOption(args.filters)

    var isMedia = true
    var uniqueMimeType: Bool? = nil
    var mimeKind: String? = nil
    if !parsedTypes.isEmpty {
      uniqueMimeType = true
      for mime in parsedTypes {
        let kind = mime.components(separatedBy: "/")[0]
        if kind != "image" && kind != "video" {
          isMedia = false
        }
        if mimeKind == nil {
          mimeKind = kind
        } else if mimeKind != kind {
          uniqueMimeType = false
        }
      }
    }

    pendingInvoke = invoke
    pendingInvokeArgs = args

    if uniqueMimeType == true || isMedia {
      DispatchQueue.main.async {
        if #available(iOS 14, *) {
          var configuration = PHPickerConfiguration(photoLibrary: PHPhotoLibrary.shared())
          configuration.selectionLimit = args.multiple ? 0 : 1

          if uniqueMimeType == true {
            if mimeKind == "image" {
              configuration.filter = .images
            } else if mimeKind == "video" {
              configuration.filter = .videos
            }
          }

          let picker = PHPickerViewController(configuration: configuration)
          picker.delegate = self.filePickerController
          picker.modalPresentationStyle = .fullScreen
          self.presentViewController(picker)
        } else {
          let picker = UIImagePickerController()
          picker.delegate = self.filePickerController

          if uniqueMimeType == true && mimeKind == "image" {
            picker.sourceType = .photoLibrary
          }

          picker.sourceType = .photoLibrary
          picker.modalPresentationStyle = .fullScreen
          self.presentViewController(picker)
        }
      }
    } else {
      let documentTypes = parsedTypes.isEmpty ? ["public.data"] : parsedTypes
      DispatchQueue.main.async {
        let picker = UIDocumentPickerViewController(documentTypes: documentTypes, in: .import)
        picker.delegate = self.filePickerController
        picker.allowsMultipleSelection = args.multiple
        picker.modalPresentationStyle = .fullScreen
        self.presentViewController(picker)
      }
    }
  }

  private func presentViewController(_ viewControllerToPresent: UIViewController) {
    self.manager.viewController?.present(viewControllerToPresent, animated: true, completion: nil)
  }

  private func parseFiltersOption(_ filters: [Filter]) -> [String] {
    var parsedTypes: [String] = []
    for filter in filters {
      for ext in filter.extensions {
        guard
          let utType: String = UTTypeCreatePreferredIdentifierForTag(
            kUTTagClassMIMEType, ext as CFString, nil)?.takeRetainedValue() as String?
        else {
          continue
        }
        parsedTypes.append(utType)
      }
    }
    return parsedTypes
  }

  public func onFilePickerEvent(_ event: FilePickerEvent) {
    switch event {
    case .selected(let urls):
      let readData = pendingInvokeArgs?.readData ?? false
      do {
        let filesResult = try urls.map { (url: URL) -> JSObject in
          var file = JSObject()

          let mimeType = filePickerController.getMimeTypeFromUrl(url)
          let isVideo = mimeType.hasPrefix("video")
          let isImage = mimeType.hasPrefix("image")

          if readData {
            file["data"] = try Data(contentsOf: url).base64EncodedString()
          }

          if isVideo {
            file["duration"] = filePickerController.getVideoDuration(url)
            let (height, width) = filePickerController.getVideoDimensions(url)
            if let height = height {
              file["height"] = height
            }
            if let width = width {
              file["width"] = width
            }
          } else if isImage {
            let (height, width) = filePickerController.getImageDimensions(url)
            if let height = height {
              file["height"] = height
            }
            if let width = width {
              file["width"] = width
            }
          }

          file["modifiedAt"] = filePickerController.getModifiedAtFromUrl(url)
          file["mimeType"] = mimeType
          file["name"] = url.lastPathComponent
          file["path"] = url.absoluteString
          file["size"] = try filePickerController.getSizeFromUrl(url)
          return file
        }
        pendingInvoke?.resolve(["files": filesResult])
      } catch let error as NSError {
        pendingInvoke?.reject(error.localizedDescription, error: error)
        return
      }

      pendingInvoke?.resolve(["files": urls])
    case .cancelled:
      let files: JSArray = []
      pendingInvoke?.resolve(["files": files])
    case .error(let error):
      pendingInvoke?.reject(error)
    }
  }

  @objc public func showMessageDialog(_ invoke: Invoke) throws {
    let manager = self.manager
    let args = try invoke.parseArgs(MessageDialogOptions.self)

    DispatchQueue.main.async { [] in
      let alert = UIAlertController(
        title: args.title, message: args.message, preferredStyle: UIAlertController.Style.alert)
      alert.addAction(
        UIAlertAction(
          title: args.cancelButtonLabel, style: UIAlertAction.Style.default,
          handler: { (_) -> Void in
            invoke.resolve([
              "value": false,
              "cancelled": false,
            ])
          }))
      alert.addAction(
        UIAlertAction(
          title: args.okButtonLabel, style: UIAlertAction.Style.default,
          handler: { (_) -> Void in
            invoke.resolve([
              "value": true,
              "cancelled": false,
            ])
          }))

      manager.viewController?.present(alert, animated: true, completion: nil)
    }
  }
}

@_cdecl("init_plugin_dialog")
func initPlugin() -> Plugin {
  return DialogPlugin()
}
