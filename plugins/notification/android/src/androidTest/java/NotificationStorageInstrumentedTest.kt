// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.content.Context
import android.content.ContextWrapper
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class NotificationStorageInstrumentedTest : NotificationStorageTest() {
  override val context: Context = object : ContextWrapper(
    InstrumentationRegistry.getInstrumentation().targetContext
  ) {
    override fun getSharedPreferences(name: String, mode: Int) =
      super.getSharedPreferences("notification-storage-test-$name", mode)
  }
}
