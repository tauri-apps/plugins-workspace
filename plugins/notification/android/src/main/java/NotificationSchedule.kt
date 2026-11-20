// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.notification

import android.annotation.SuppressLint
import android.text.format.DateUtils
import com.fasterxml.jackson.annotation.JsonFormat
import com.fasterxml.jackson.annotation.JsonProperty
import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.core.JsonProcessingException
import com.fasterxml.jackson.databind.DeserializationContext
import com.fasterxml.jackson.databind.JsonDeserializer
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.SerializerProvider
import com.fasterxml.jackson.databind.annotation.JsonDeserialize
import com.fasterxml.jackson.databind.annotation.JsonSerialize
import com.fasterxml.jackson.databind.deser.std.StdDeserializer
import com.fasterxml.jackson.databind.ser.std.StdSerializer
import java.io.IOException
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.TimeZone

const val JS_DATE_FORMAT = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"

enum class NotificationInterval {
  @JsonProperty("year")
  Year,
  @JsonProperty("month")
  Month,
  @JsonProperty("twoWeeks")
  TwoWeeks,
  @JsonProperty("week")
  Week,
  @JsonProperty("day")
  Day,
  @JsonProperty("hour")
  Hour,
  @JsonProperty("minute")
  Minute,
  @JsonProperty("second")
  Second
}

fun getIntervalTime(interval: NotificationInterval, count: Int): Long {
  return when (interval) {
    // This case is just approximation as not all years have the same number of days
    NotificationInterval.Year -> count * DateUtils.WEEK_IN_MILLIS * 52
    // This case is just approximation as months have different number of days
    NotificationInterval.Month -> count * 30 * DateUtils.DAY_IN_MILLIS
    NotificationInterval.TwoWeeks -> count * 2 * DateUtils.WEEK_IN_MILLIS
    NotificationInterval.Week -> count * DateUtils.WEEK_IN_MILLIS
    NotificationInterval.Day -> count * DateUtils.DAY_IN_MILLIS
    NotificationInterval.Hour -> count * DateUtils.HOUR_IN_MILLIS
    NotificationInterval.Minute -> count * DateUtils.MINUTE_IN_MILLIS
    NotificationInterval.Second -> count * DateUtils.SECOND_IN_MILLIS
  }
}

@JsonDeserialize(using = NotificationScheduleDeserializer::class)
@JsonSerialize(using = NotificationScheduleSerializer::class)
sealed class NotificationSchedule {
  // At specific moment of time (with repeating option)
  @JsonDeserialize
  class At: NotificationSchedule() {
    @JsonFormat(shape = JsonFormat.Shape.STRING, pattern = JS_DATE_FORMAT)
    lateinit var date: Date
    var repeating: Boolean = false
    var allowWhileIdle: Boolean = false
  }
  @JsonDeserialize
  class Interval: NotificationSchedule() {
    lateinit var interval: DateMatch
    var allowWhileIdle: Boolean = false
  }
  @JsonDeserialize
  class Every: NotificationSchedule() {
    lateinit var interval: NotificationInterval
    var count: Int = 0
    var allowWhileIdle: Boolean = false
  }

  fun isRemovable(): Boolean {
    return when (this) {
      is At -> !repeating
      else -> false
    }
  }

  fun allowWhileIdle(): Boolean {
    return when (this) {
      is At -> allowWhileIdle
      is Interval -> allowWhileIdle
      is Every -> allowWhileIdle
      else -> false
    }
  }
}

internal class NotificationScheduleSerializer @JvmOverloads constructor(t: Class<NotificationSchedule>? = null) :
  StdSerializer<NotificationSchedule>(t) {
  @SuppressLint("SimpleDateFormat")
  @Throws(IOException::class, JsonProcessingException::class)
  override fun serialize(
    value: NotificationSchedule, jgen: JsonGenerator, provider: SerializerProvider
  ) {
    jgen.writeStartObject()
    when (value) {
      is NotificationSchedule.At -> {
        jgen.writeObjectFieldStart("at")

        val sdf = SimpleDateFormat(JS_DATE_FORMAT)
        sdf.timeZone = TimeZone.getTimeZone("UTC")
        jgen.writeStringField("date", sdf.format(value.date))
        jgen.writeBooleanField("repeating", value.repeating)

        jgen.writeEndObject()
      }
      is NotificationSchedule.Interval -> {
        jgen.writeObjectFieldStart("interval")

        jgen.writeObjectField("interval", value.interval)

        jgen.writeEndObject()
      }
      is NotificationSchedule.Every -> {
        jgen.writeObjectFieldStart("every")

        jgen.writeObjectField("interval", value.interval)
        jgen.writeNumberField("count", value.count)

        jgen.writeEndObject()
      }
    }

    jgen.writeEndObject()
  }
}

internal class NotificationScheduleDeserializer: JsonDeserializer<NotificationSchedule>() {
  override fun deserialize(
    jsonParser: JsonParser,
    deserializationContext: DeserializationContext
  ): NotificationSchedule {
    val node: JsonNode = jsonParser.codec.readTree(jsonParser)
    node.get("at")?.let {
      return jsonParser.codec.treeToValue(it, NotificationSchedule.At::class.java)
    }
    node.get("interval")?.let {
      return jsonParser.codec.treeToValue(it, NotificationSchedule.Interval::class.java)
    }
    node.get("every")?.let {
      return jsonParser.codec.treeToValue(it, NotificationSchedule.Every::class.java)
    }
    throw Error("unknown schedule kind $node")
  }
}

class DateMatch {
  var year: Int? = null
  var month: Int? = null
  var day: Int? = null
  var weekday: Int? = null
  var hour: Int? = null
  var minute: Int? = null
  var second: Int? = null

  // Unit used to save the last used unit for a trigger.
  // One of the Calendar constants values
  var unit: Int? = -1

  /**
   * Gets a calendar instance pointing to the specified date.
   *
   * @param date The date to point.
   */
  private fun buildCalendar(date: Date): Calendar {
    val cal: Calendar = Calendar.getInstance()
    cal.time = date
    cal.set(Calendar.MILLISECOND, 0)
    return cal
  }

  /**
   * Calculates next trigger date for
   *
   * @param date base date used to calculate trigger
   * @return next trigger timestamp
   */
  fun nextTrigger(date: Date): Long {
    val current: Calendar = buildCalendar(date)
    val next: Calendar = buildNextTriggerTime(date)
    return postponeTriggerIfNeeded(current, next)
  }

  /**
   * Postpone trigger if first schedule matches the past
   */
  private fun postponeTriggerIfNeeded(current: Calendar, next: Calendar): Long {
    if (next.timeInMillis <= current.timeInMillis && unit != -1) {
      var incrementUnit = -1
      if (unit == Calendar.YEAR || unit == Calendar.MONTH) {
        incrementUnit = Calendar.YEAR
      } else if (unit == Calendar.DAY_OF_MONTH) {
        incrementUnit = Calendar.MONTH
      } else if (unit == Calendar.DAY_OF_WEEK) {
        incrementUnit = Calendar.WEEK_OF_MONTH
      } else if (unit == Calendar.HOUR_OF_DAY) {
        incrementUnit = Calendar.DAY_OF_MONTH
      } else if (unit == Calendar.MINUTE) {
        incrementUnit = Calendar.HOUR_OF_DAY
      } else if (unit == Calendar.SECOND) {
        incrementUnit = Calendar.MINUTE
      }
      if (incrementUnit != -1) {
        next.set(incrementUnit, next.get(incrementUnit) + 1)
      }
    }
    return next.timeInMillis
  }

  private fun buildNextTriggerTime(date: Date): Calendar {
    val next: Calendar = buildCalendar(date)
    if (year != null) {
      next.set(Calendar.YEAR, year ?: 0)
      if (unit == -1) unit = Calendar.YEAR
    }
    if (month != null) {
      next.set(Calendar.MONTH, month ?: 0)
      if (unit == -1) unit = Calendar.MONTH
    }
    if (day != null) {
      next.set(Calendar.DAY_OF_MONTH, day ?: 0)
      if (unit == -1) unit = Calendar.DAY_OF_MONTH
    }
    if (weekday != null) {
      next.set(Calendar.DAY_OF_WEEK, weekday ?: 0)
      if (unit == -1) unit = Calendar.DAY_OF_WEEK
    }
    if (hour != null) {
      next.set(Calendar.HOUR_OF_DAY, hour ?: 0)
      if (unit == -1) unit = Calendar.HOUR_OF_DAY
    }
    if (minute != null) {
      next.set(Calendar.MINUTE, minute ?: 0)
      if (unit == -1) unit = Calendar.MINUTE
    }
    if (second != null) {
      next.set(Calendar.SECOND, second ?: 0)
      if (unit == -1) unit = Calendar.SECOND
    }
    return next
  }

  override fun toString(): String {
    return "DateMatch{" +
            "year=" +
            year +
            ", month=" +
            month +
            ", day=" +
            day +
            ", weekday=" +
            weekday +
            ", hour=" +
            hour +
            ", minute=" +
            minute +
            ", second=" +
            second +
            '}'
  }

  override fun equals(other: Any?): Boolean {
    if (this === other) return true
    if (other == null || javaClass != other.javaClass) return false
    val dateMatch = other as DateMatch
    if (if (year != null) year != dateMatch.year else dateMatch.year != null) return false
    if (if (month != null) month != dateMatch.month else dateMatch.month != null) return false
    if (if (day != null) day != dateMatch.day else dateMatch.day != null) return false
    if (if (weekday != null) weekday != dateMatch.weekday else dateMatch.weekday != null) return false
    if (if (hour != null) hour != dateMatch.hour else dateMatch.hour != null) return false
    if (if (minute != null) minute != dateMatch.minute else dateMatch.minute != null) return false
    return if (second != null) second == dateMatch.second else dateMatch.second == null
  }

  override fun hashCode(): Int {
    var result = if (year != null) year.hashCode() else 0
    result = 31 * result + if (month != null) month.hashCode() else 0
    result = 31 * result + if (day != null) day.hashCode() else 0
    result = 31 * result + if (weekday != null) weekday.hashCode() else 0
    result = 31 * result + if (hour != null) hour.hashCode() else 0
    result = 31 * result + if (minute != null) minute.hashCode() else 0
    result += 31 + if (second != null) second.hashCode() else 0
    return result
  }

  /**
   * Transform DateMatch object to CronString
   *
   * @return
   */
  fun toMatchString(): String {
    val matchString = year.toString() +
            separator +
            month +
            separator +
            day +
            separator +
            weekday +
            separator +
            hour +
            separator +
            minute +
            separator +
            second +
            separator +
            unit
    return matchString.replace("null", "*")
  }

  companion object {
    private const val separator = " "

    /**
     * Create DateMatch object from stored string
     *
     * @param matchString
     * @return
     */
    fun fromMatchString(matchString: String): DateMatch {
      val date = DateMatch()
      val split = matchString.split(separator.toRegex()).dropLastWhile { it.isEmpty() }
        .toTypedArray()
      if (split.size == 7) {
        date.year = getValueFromCronElement(split[0])
        date.month = getValueFromCronElement(split[1])
        date.day = getValueFromCronElement(split[2])
        date.weekday = getValueFromCronElement(split[3])
        date.hour = getValueFromCronElement(split[4])
        date.minute = getValueFromCronElement(split[5])
        date.unit = getValueFromCronElement(split[6])
      }
      if (split.size == 8) {
        date.year = getValueFromCronElement(split[0])
        date.month = getValueFromCronElement(split[1])
        date.day = getValueFromCronElement(split[2])
        date.weekday = getValueFromCronElement(split[3])
        date.hour = getValueFromCronElement(split[4])
        date.minute = getValueFromCronElement(split[5])
        date.second = getValueFromCronElement(split[6])
        date.unit = getValueFromCronElement(split[7])
      }
      return date
    }

    private fun getValueFromCronElement(token: String): Int? {
      return try {
        token.toInt()
      } catch (e: NumberFormatException) {
        null
      }
    }
  }
}