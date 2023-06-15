// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.annotation.SuppressLint
import android.content.Context

class AssetUtils {
  companion object {
    const val RESOURCE_ID_ZERO_VALUE = 0

    @SuppressLint("DiscouragedApi")
    fun getResourceID(context: Context, resourceName: String?, dir: String?): Int {
      return context.resources.getIdentifier(resourceName, dir, context.packageName)
    }

    fun getResourceBaseName(resPath: String?): String? {
      if (resPath == null) return null
      if (resPath.contains("/")) {
        return resPath.substring(resPath.lastIndexOf('/') + 1)
      }
      return if (resPath.contains(".")) {
        resPath.substring(0, resPath.lastIndexOf('.'))
      } else resPath
    }
  }
}
