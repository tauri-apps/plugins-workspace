// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.content.Context
import com.fasterxml.jackson.annotation.JsonAutoDetect
import com.fasterxml.jackson.annotation.PropertyAccessor
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.ObjectMapper
import app.tauri.plugin.JSObject
import org.json.JSONObject
import org.json.JSONException
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import java.util.Date

abstract class NotificationStorageTest {
  protected abstract val context: Context

  private fun preferences() = context.getSharedPreferences("NOTIFICATION_STORE", Context.MODE_PRIVATE)

  protected open fun configuredMapper() = ObjectMapper()
    .disable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
    .enable(DeserializationFeature.FAIL_ON_NULL_FOR_PRIMITIVES)
    .setVisibility(PropertyAccessor.FIELD, JsonAutoDetect.Visibility.ANY)

  @Before
  fun clearTestStore() {
    assertTrue(preferences().edit().clear().commit())
  }

  private fun notification(id: Int = 7039373): Notification {
    return Notification().apply {
      this.id = id
      title = "Reminder"
      body = "Saved body"
      channelId = "reminders"
      isGroupSummary = true
      isOngoing = true
      isAutoCancel = true
      usesChronometer = true
      chronometerCountDown = true
      visibility = -1
      number = 7
      `when` = 123456789L
      inboxLines = listOf("first", "second")
      extra = JSObject("""{"nested":{"items":[1,true,null,"text"]},"flag":false}""")
      attachments = listOf(NotificationAttachment().apply {
        this.id = "picture"
        url = "asset://notification.png"
        options = JSONObject("""{"nested":{"width":320},"flags":[true,null]}""")
      })
      schedule = NotificationSchedule.At().apply {
        date = Date(2000000000000L)
        repeating = true
        allowWhileIdle = true
      }
    }
  }

  @Test
  fun storesTheModelWithoutSourceJsonAndRestoresWithBootMapper() {
    val original = notification()
    assertNull(original.sourceJson)
    NotificationStorage(context, configuredMapper()).appendNotifications(listOf(original))
    val raw = preferences().getString(original.id.toString(), null)
    assertNotNull(raw)
    assertNotEquals("null", raw)
    assertFalse(JSONObject(raw!!).has("sourceJson"))

    val storage = NotificationStorage(context, ObjectMapper())
    val restored = storage.getSavedNotification(original.id.toString())!!
    assertEquals(original.id, restored.id)
    assertEquals("Reminder", restored.title)
    assertEquals("Saved body", restored.body)
    assertEquals("reminders", restored.channelId)
    assertTrue(restored.isGroupSummary)
    assertTrue(restored.isOngoing)
    assertTrue(restored.isAutoCancel)
    assertTrue(restored.usesChronometer)
    assertTrue(restored.chronometerCountDown)
    assertEquals(-1, restored.visibility)
    assertEquals(7, restored.number)
    assertEquals(123456789L, restored.`when`)
    assertEquals(listOf("first", "second"), restored.inboxLines)
    val schedule = restored.schedule as NotificationSchedule.At
    assertEquals(2000000000000L, schedule.date.time)
    assertTrue(schedule.repeating)
    assertTrue(schedule.allowWhileIdle)
    val mapper = ObjectMapper()
    assertEquals(mapper.readTree(original.extra.toString()), mapper.readTree(restored.extra.toString()))
    assertEquals(mapper.readTree(original.attachments!![0].options.toString()),
      mapper.readTree(restored.attachments!![0].options.toString()))
    try {
      restored.attachments!![0].options!!.getString("missing")
      fail("Attachment options must retain JSONObject lookup behavior")
    } catch (_: JSONException) { }
    assertEquals(raw, restored.sourceJson)
    val pending = Notification.buildNotificationPendingList(storage.getSavedNotifications())
    assertEquals(1, pending.size)
    assertEquals(original.id, pending[0].id)
    assertEquals("Reminder", pending[0].title)
  }

  @Test
  fun preservesEveryAndIntervalSchedulesForMultipleNotifications() {
    val every = notification(21).apply {
      schedule = NotificationSchedule.Every().apply {
        interval = NotificationInterval.Day
        count = 2
        allowWhileIdle = true
      }
    }
    val interval = notification(22).apply {
      schedule = NotificationSchedule.Interval().apply {
        interval = DateMatch().apply { hour = 20; minute = 0; second = 0 }
        allowWhileIdle = true
      }
    }
    NotificationStorage(context, ObjectMapper()).appendNotifications(listOf(every, interval))
    val storage = NotificationStorage(context, configuredMapper())
    val first = storage.getSavedNotification("21")!!.schedule as NotificationSchedule.Every
    assertEquals(NotificationInterval.Day, first.interval)
    assertEquals(2, first.count)
    assertTrue(first.allowWhileIdle)
    val second = storage.getSavedNotification("22")!!.schedule as NotificationSchedule.Interval
    assertEquals(20, second.interval.hour)
    assertEquals(0, second.interval.minute)
    assertEquals(0, second.interval.second)
    assertTrue(second.allowWhileIdle)
    assertEquals(setOf(21, 22), storage.getSavedNotifications().map { it.id }.toSet())
  }

  @Test
  fun persistsDeserializedBatchNotifications() {
    val mapper = configuredMapper()
    val args = mapper.readValue("""{"notifications":[{"id":8,"title":"Batch reminder","extra":{"nested":[1,true,null]},"schedule":{"every":{"interval":"day","count":2,"allowWhileIdle":true}}}]}""", BatchArgs::class.java)
    assertNull(args.notifications.single().sourceJson)
    NotificationStorage(context, mapper).appendNotifications(args.notifications)
    val restored = NotificationStorage(context, ObjectMapper()).getSavedNotification("8")!!
    assertEquals("Batch reminder", restored.title)
    assertEquals(mapper.readTree("""{"nested":[1,true,null]}"""), mapper.readTree(restored.extra.toString()))
    assertTrue((restored.schedule as NotificationSchedule.Every).allowWhileIdle)
  }

  @Test
  fun legacyNullDoesNotBreakTheHealthyPendingList() {
    preferences().edit().putString("1", "null")
      .putString("4", """{"id":4,"title":"Healthy","schedule":{"every":{"interval":"day","count":2}}}""")
      .commit()
    val storage = NotificationStorage(context, ObjectMapper())
    val pending = Notification.buildNotificationPendingList(storage.getSavedNotifications())
    assertEquals(4, pending.single().id)
    assertEquals("Healthy", pending.single().title)
    assertEquals(setOf("1", "4"), storage.getSavedNotificationIds().toSet())
  }

  @Test
  fun resavingUsesChangedFieldsRatherThanStaleSourceJson() {
    val storage = NotificationStorage(context, ObjectMapper())
    storage.appendNotifications(listOf(notification()))
    val restored = storage.getSavedNotification("7039373")!!
    assertNotNull(restored.sourceJson)
    restored.title = "Updated"
    (restored.schedule as NotificationSchedule.At).date = Date(2100000000000L)
    storage.appendNotifications(listOf(restored))
    val again = NotificationStorage(context, ObjectMapper()).getSavedNotification("7039373")!!
    assertEquals("Updated", again.title)
    assertEquals(2100000000000L, (again.schedule as NotificationSchedule.At).date.time)
    assertFalse(JSONObject(preferences().getString("7039373", null)!!).has("sourceJson"))
  }

  @Test
  fun preservesNullOptionalJsonFields() {
    val notification = notification().apply {
      extra = null
      attachments!![0].options = null
    }
    NotificationStorage(context, ObjectMapper()).appendNotifications(listOf(notification))
    val restored = NotificationStorage(context, ObjectMapper()).getSavedNotification("7039373")!!
    assertNull(restored.extra)
    assertNull(restored.attachments!![0].options)
  }

  @Test
  fun skipsLegacyNullAndMalformedRecordsWithoutDeletingTheirIds() {
    preferences().edit()
      .putString("1", "null")
      .putString("2", "not-json")
      .putInt("3", 42)
      .putString("4", """{"id":4,"title":"Healthy","schedule":{"every":{"interval":"day","count":2}}}""")
      .putString("5", """{"id":5,"schedule":{"unknown":{}}}""")
      .commit()
    val storage = NotificationStorage(context, ObjectMapper())
    assertNull(storage.getSavedNotification("1"))
    assertNull(storage.getSavedNotification("2"))
    assertNull(storage.getSavedNotification("3"))
    assertEquals(listOf(4), storage.getSavedNotifications().map { it.id })
    assertEquals(4, Notification.buildNotificationPendingList(storage.getSavedNotifications()).single().id)
    assertNull(storage.getSavedNotification("5"))
    assertEquals(setOf("1", "2", "3", "4", "5"), storage.getSavedNotificationIds().toSet())
  }

  @Test
  fun doesNotPersistUnscheduledNotifications() {
    NotificationStorage(context, ObjectMapper()).appendNotifications(listOf(notification().apply { schedule = null }))
    assertTrue(preferences().all.isEmpty())
  }

  @Test
  fun leavesCallerMapperConfigurationUnchanged() {
    val mapper = ObjectMapper()
    assertTrue(mapper.isEnabled(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES))
    NotificationStorage(context, mapper)
    assertTrue(mapper.isEnabled(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES))
  }
}
