// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.content.Context
import android.content.ContextWrapper
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.json.JSONObject
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.util.Date

@RunWith(AndroidJUnit4::class)
class NotificationStorageTest {
  // keep the test's records away from the app's own store
  private val context: Context = object : ContextWrapper(
    InstrumentationRegistry.getInstrumentation().targetContext
  ) {
    override fun getSharedPreferences(name: String, mode: Int) =
      super.getSharedPreferences("test-$name", mode)
  }

  private fun preferences() = context.getSharedPreferences("NOTIFICATION_STORE", Context.MODE_PRIVATE)

  private fun storage() = NotificationStorage(context, NotificationStorage.defaultMapper())

  // what `batch` receives from Rust, including keys the Kotlin model does not have (`silent`)
  private val batchJson = """{"notifications":[
    {"id":7,"channelId":"reminders","title":"Reminder","body":"Saved body","silent":false,
     "groupSummary":false,"ongoing":false,"autoCancel":true,"attachments":[],
     "extra":{"nested":{"items":[1,true,null,"text"]}},
     "schedule":{"at":{"date":"2033-05-18T03:33:20.000Z","repeating":false,"allowWhileIdle":true}}},
    {"id":8,"title":"Daily","extra":{},"schedule":{"every":{"interval":"day","count":2,"allowWhileIdle":true}}},
    {"id":9,"title":"Evening","schedule":{"interval":{"interval":{"hour":20,"minute":0,"second":0},"allowWhileIdle":true}}}
  ]}"""

  // what `batch` does before storing: parse with the plugin's mapper, keep each raw notification
  private fun receiveBatch(): List<Notification> {
    val args = NotificationStorage.defaultMapper().readValue(batchJson, BatchArgs::class.java)
    val raw = JSONObject(batchJson).getJSONArray("notifications")
    args.notifications.forEachIndexed { i, n -> n.sourceJson = raw.getJSONObject(i).toString() }
    return args.notifications
  }

  @Before
  fun clearStore() {
    assertTrue(preferences().edit().clear().commit())
  }

  @Test
  fun storesTheReceivedJsonAndReadsItBackWithTheReceiverMapper() {
    storage().appendNotifications(receiveBatch())

    val raw = JSONObject(preferences().getString("7", null)!!)
    assertEquals("Reminder", raw.getString("title"))
    assertEquals(
      "text",
      raw.getJSONObject("extra").getJSONObject("nested").getJSONArray("items").getString(3)
    )

    val restored = storage().getSavedNotification("7")!!
    assertEquals("Reminder", restored.title)
    assertEquals("reminders", restored.channelId)
    assertTrue(restored.isAutoCancel)
    val at = restored.schedule as NotificationSchedule.At
    assertEquals(2000000000000L, at.date.time)
    assertTrue(at.allowWhileIdle)
    // action intents carry this
    assertEquals(raw.toString(), JSONObject(restored.sourceJson!!).toString())

    val every = storage().getSavedNotification("8")!!.schedule as NotificationSchedule.Every
    assertEquals(NotificationInterval.Day, every.interval)
    assertEquals(2, every.count)
    assertTrue(every.allowWhileIdle)

    val interval = storage().getSavedNotification("9")!!.schedule as NotificationSchedule.Interval
    assertEquals(20, interval.interval.hour)
    assertTrue(interval.allowWhileIdle)

    val pending = Notification.buildNotificationPendingList(storage().getSavedNotifications())
    assertEquals(setOf(7, 8, 9), pending.map { it.id }.toSet())
  }

  @Test
  fun resavingKeepsAChangedSchedule() {
    storage().appendNotifications(receiveBatch())
    // what the restore receiver does with an overdue notification
    val restored = storage().getSavedNotification("7")!!
    (restored.schedule as NotificationSchedule.At).date = Date(2100000000000L)
    storage().appendNotifications(listOf(restored))

    val again = storage().getSavedNotification("7")!!
    val at = again.schedule as NotificationSchedule.At
    assertEquals(2100000000000L, at.date.time)
    assertTrue(at.allowWhileIdle)
    assertEquals("Reminder", again.title)
    val raw = JSONObject(preferences().getString("7", null)!!)
    assertTrue(raw.getJSONObject("extra").has("nested"))
  }

  @Test
  fun skipsNullAndUnreadableRecords() {
    preferences().edit()
      .putString("1", "null")
      .putString("2", "not-json")
      .putInt("3", 42)
      .putString("4", """{"id":4,"title":"Healthy","schedule":{"unknown":{}}}""")
      .commit()
    storage().appendNotifications(receiveBatch().take(1))

    for (id in listOf("1", "2", "3", "4")) {
      assertNull(storage().getSavedNotification(id))
    }
    val pending = Notification.buildNotificationPendingList(storage().getSavedNotifications())
    assertEquals(listOf(7), pending.map { it.id })
  }

  @Test
  fun persistsOnlyScheduledNotificationsWithSourceJson() {
    val (scheduled, unscheduled) = receiveBatch().take(2)
    unscheduled.schedule = null
    scheduled.sourceJson = null
    storage().appendNotifications(listOf(scheduled, unscheduled))
    assertTrue(preferences().all.isEmpty())
  }
}
