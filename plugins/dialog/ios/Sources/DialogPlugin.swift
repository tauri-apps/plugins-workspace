import UIKit
import MobileCoreServices
import PhotosUI
import Photos
import WebKit
import Tauri
import SwiftRs

enum FilePickerEvent {
	case selected([URL])
	case cancelled
	case error(String)
}

class DialogPlugin: Plugin {

	var filePickerController: FilePickerController!
	var pendingInvoke: Invoke? = nil

	override init() {
		super.init()
		filePickerController = FilePickerController(self)
	}

	@objc public func showFilePicker(_ invoke: Invoke) {
		let multiple = invoke.getBool("multiple", false)
		let filters = invoke.getArray("filters") ?? []
		let parsedTypes = parseFiltersOption(filters)

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
				if (mimeKind == nil) {
					mimeKind = kind
				} else if (mimeKind != kind) {
					uniqueMimeType = false
				}
      }
		}

		pendingInvoke = invoke

		if uniqueMimeType == true || isMedia {
			DispatchQueue.main.async {
				if #available(iOS 14, *) {
					var configuration = PHPickerConfiguration(photoLibrary: PHPhotoLibrary.shared())
					configuration.selectionLimit = multiple ? 0 : 1

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
				picker.allowsMultipleSelection = multiple
				picker.modalPresentationStyle = .fullScreen
				self.presentViewController(picker)
			}
		}
	}

	private func presentViewController(_ viewControllerToPresent: UIViewController) {
		self.manager.viewController?.present(viewControllerToPresent, animated: true, completion: nil)
	}

	private func parseFiltersOption(_ filters: JSArray) -> [String] {
		var parsedTypes: [String] = []
		for (_, filter) in filters.enumerated() {
			let filterObj = filter as? JSObject
			if let filterObj = filterObj {
				let extensions = filterObj["extensions"] as? JSArray
				if let extensions = extensions {
					for e in extensions {
						let ext = e as? String ?? ""
						guard let utType: String = UTTypeCreatePreferredIdentifierForTag(kUTTagClassMIMEType, ext as CFString, nil)?.takeRetainedValue() as String? else {
							continue
						}
						parsedTypes.append(utType)
					}
				}
			}
		}
		return parsedTypes
  }

	public func onFilePickerEvent(_ event: FilePickerEvent) {
		switch event {
			case .selected(let urls):
				let readData = pendingInvoke?.getBool("readData", false) ?? false
        do {
					let filesResult = try urls.map {(url: URL) -> JSObject in
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
					pendingInvoke?.reject(error.localizedDescription, nil, error)
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

	@objc public func showMessageDialog(_ invoke: Invoke) {
		let manager = self.manager
		let title = invoke.getString("title")
		guard let message = invoke.getString("message") else {
			invoke.reject("The `message` argument is required")
			return
		}
		let okButtonLabel = invoke.getString("okButtonLabel") ?? "OK"
		let cancelButtonLabel = invoke.getString("cancelButtonLabel") ?? "Cancel"
		
		DispatchQueue.main.async { [] in
			let alert = UIAlertController(title: title, message: message, preferredStyle: UIAlertController.Style.alert)
			alert.addAction(UIAlertAction(title: cancelButtonLabel, style: UIAlertAction.Style.default, handler: { (_) -> Void in
				invoke.resolve([
					"value": false,
					"cancelled": false
				])
			}))
			alert.addAction(UIAlertAction(title: okButtonLabel, style: UIAlertAction.Style.default, handler: { (_) -> Void in
				invoke.resolve([
					"value": true,
					"cancelled": false
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
