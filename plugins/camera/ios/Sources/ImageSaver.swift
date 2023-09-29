// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import UIKit

class ImageSaver: NSObject {

    var onResult: ((Error?) -> Void) = {_ in }

    init(image: UIImage, onResult:@escaping ((Error?) -> Void)) {
        self.onResult = onResult
        super.init()
        UIImageWriteToSavedPhotosAlbum(image, self, #selector(saveResult), nil)
    }

    @objc func saveResult(_ image: UIImage, didFinishSavingWithError error: Error?, contextInfo: UnsafeRawPointer) {
        if let error = error {
            onResult(error)
        } else {
            onResult(nil)
        }
    }
}