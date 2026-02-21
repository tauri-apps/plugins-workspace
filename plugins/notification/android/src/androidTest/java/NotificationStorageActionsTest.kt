// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.fasterxml.jackson.databind.ObjectMapper
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class NotificationStorageActionsTest {
  @Test
  fun actionGroup_roundTrip() {
    val context = InstrumentationRegistry.getInstrumentation().targetContext
    val storage = NotificationStorage(context, ObjectMapper())

    val reply = NotificationAction().apply {
      id = "reply"
      title = "Reply"
      input = true
    }
    val markRead = NotificationAction().apply {
      id = "mark-read"
      title = "Mark Read"
      input = false
    }

    val type = ActionType().apply {
      id = "chat-actions"
      actions = listOf(reply, markRead)
    }

    storage.writeActionGroup(listOf(type))
    val restored = storage.getActionGroup("chat-actions")

    assertEquals(2, restored.size)
    assertEquals("reply", restored[0]!!.id)
    assertEquals("Reply", restored[0]!!.title)
    assertTrue(restored[0]!!.input == true)
    assertEquals("mark-read", restored[1]!!.id)
    assertEquals("Mark Read", restored[1]!!.title)
  }
}
