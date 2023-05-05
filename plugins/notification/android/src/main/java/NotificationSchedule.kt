package app.tauri.notification

import android.annotation.SuppressLint
import android.text.format.DateUtils
import app.tauri.plugin.JSObject
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.TimeZone

const val JS_DATE_FORMAT = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"

enum class NotificationInterval {
  Year, Month, TwoWeeks, Week, Day, Hour, Minute, Second
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

sealed class ScheduleKind {
  // At specific moment of time (with repeating option)
  class At(var date: Date, val repeating: Boolean): ScheduleKind()
  class Interval(val interval: DateMatch): ScheduleKind()
  class Every(val interval: NotificationInterval, val count: Int): ScheduleKind()
}

@SuppressLint("SimpleDateFormat")
class NotificationSchedule(val scheduleObj: JSObject) {
  val kind: ScheduleKind
  // Schedule this notification to fire even if app is idled (Doze)
  var whileIdle: Boolean = false

  init {
    val payload = scheduleObj.getJSObject("data", JSObject())

    when (val scheduleKind = scheduleObj.getString("kind", "")) {
      "At" -> {
        val dateString = payload.getString("date")
        if (dateString.isNotEmpty()) {
          val sdf = SimpleDateFormat(JS_DATE_FORMAT)
          sdf.timeZone = TimeZone.getTimeZone("UTC")
          val at = sdf.parse(dateString)
          if (at == null) {
            throw Exception("could not parse `at` date")
          } else {
            kind = ScheduleKind.At(at, payload.getBoolean("repeating"))
          }
        } else {
          throw Exception("`at` date cannot be empty")
        }
      }
      "Interval" -> {
        val dateMatch = onFromJson(payload)
        kind = ScheduleKind.Interval(dateMatch)
      }
      "Every" -> {
        val interval = NotificationInterval.valueOf(payload.getString("interval"))
        kind = ScheduleKind.Every(interval, payload.getInteger("count", 1))
      }
      else -> {
        throw Exception("Unknown schedule kind $scheduleKind")
      }
    }
    whileIdle = scheduleObj.getBoolean("allowWhileIdle", false)
  }

  private fun onFromJson(onJson: JSObject): DateMatch {
    val match = DateMatch()
    match.year = onJson.getInteger("year")
    match.month = onJson.getInteger("month")
    match.day = onJson.getInteger("day")
    match.weekday = onJson.getInteger("weekday")
    match.hour = onJson.getInteger("hour")
    match.minute = onJson.getInteger("minute")
    match.second = onJson.getInteger("second")
    return match
  }

  fun isRemovable(): Boolean {
    return when (kind) {
      is ScheduleKind.At -> !kind.repeating
      else -> false
    }
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