//! Display only: never use these strings for persistence or protocol values.
#[cfg(target_os = "macos")]
use objc2_foundation::{
    NSCalendar, NSCalendarOptions, NSCalendarUnit, NSDate, NSDateFormatter, NSDateFormatterStyle,
    NSLocale, NSNumber, NSNumberFormatter, NSNumberFormatterStyle, NSString,
};
#[cfg(target_os = "macos")]
use std::cell::RefCell;

/// A formatter plus the locale it was built for.
#[cfg(target_os = "macos")]
type LocaleBound<T> = RefCell<Option<(String, objc2::rc::Retained<T>)>>;
/// Date formatters keyed by localized template, with the locale they belong to.
#[cfg(target_os = "macos")]
type DateFormatters = RefCell<(String, Vec<(String, objc2::rc::Retained<NSDateFormatter>)>)>;

/// Run `build` once per thread and reuse it until the system locale changes.
/// Each Foundation formatter costs tens of microseconds to construct, which per
/// visible row is a measurable slice of a frame. Deliberately not a cache
/// framework: no eviction, no locks, one entry per thread.
#[cfg(target_os = "macos")]
fn cached<T>(
    slot: &'static std::thread::LocalKey<LocaleBound<T>>,
    build: impl FnOnce() -> objc2::rc::Retained<T>,
) -> objc2::rc::Retained<T>
where
    T: objc2::Message + 'static,
{
    let locale = NSLocale::currentLocale().localeIdentifier().to_string();
    slot.with(|cell| {
        let mut cached = cell.borrow_mut();
        if cached.as_ref().is_none_or(|(cached, _)| *cached != locale) {
            *cached = Some((locale, build()));
        }
        cached.as_ref().expect("populated above").1.clone()
    })
}

#[cfg(target_os = "macos")]
fn decimal_formatter() -> objc2::rc::Retained<NSNumberFormatter> {
    let formatter = NSNumberFormatter::new();
    formatter.setLocale(Some(&NSLocale::currentLocale()));
    formatter.setNumberStyle(NSNumberFormatterStyle::DecimalStyle);
    formatter
}

#[cfg(target_os = "macos")]
fn currency_formatter() -> objc2::rc::Retained<NSNumberFormatter> {
    let formatter = NSNumberFormatter::new();
    formatter.setLocale(Some(&NSLocale::currentLocale()));
    formatter.setNumberStyle(NSNumberFormatterStyle::CurrencyStyle);
    formatter.setCurrencyCode(Some(&NSString::from_str("USD")));
    formatter
}

#[cfg(target_os = "macos")]
fn percent_formatter() -> objc2::rc::Retained<NSNumberFormatter> {
    let formatter = NSNumberFormatter::new();
    formatter.setLocale(Some(&NSLocale::currentLocale()));
    formatter.setNumberStyle(NSNumberFormatterStyle::PercentStyle);
    formatter
}

thread_local! {
    // One formatter per style, rebuilt when the system locale changes.
    static DECIMAL: LocaleBound<NSNumberFormatter> = const { RefCell::new(None) };
    static CURRENCY: LocaleBound<NSNumberFormatter> = const { RefCell::new(None) };
    static PERCENT: LocaleBound<NSNumberFormatter> = const { RefCell::new(None) };
    // Date formatters are per template; the list stays small (a handful of
    // templates) and is cleared when the locale changes.
    static DATES: DateFormatters = const { RefCell::new((String::new(), Vec::new())) };
}

pub(crate) fn integer(value: i64) -> String {
    #[cfg(target_os = "macos")]
    {
        let formatter = cached(&DECIMAL, decimal_formatter);
        // The formatter is shared, so every property this call depends on is
        // set here rather than inherited from whoever used it last.
        formatter.setMinimumIntegerDigits(0);
        formatter.setMinimumFractionDigits(0);
        formatter.setMaximumFractionDigits(0);
        formatter
            .stringFromNumber(&NSNumber::numberWithLongLong(value))
            .map(|text| text.to_string())
            .unwrap_or_else(|| value.to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        // shortcut: legacy grouping on Windows/Linux, replace with their system APIs when implemented.
        let digits = value.unsigned_abs().to_string();
        let mut out = String::new();
        for (i, ch) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i).is_multiple_of(3) {
                out.push(',');
            }
            out.push(ch);
        }
        if value < 0 { format!("-{out}") } else { out }
    }
}

pub(crate) fn unsigned(value: u64) -> String {
    padded_unsigned(value, 1)
}

pub(crate) fn padded_unsigned(value: u64, digits: usize) -> String {
    #[cfg(target_os = "macos")]
    {
        let formatter = cached(&DECIMAL, decimal_formatter);
        formatter.setMinimumIntegerDigits(digits);
        formatter.setMinimumFractionDigits(0);
        formatter.setMaximumFractionDigits(0);
        formatter
            .stringFromNumber(&NSNumber::numberWithUnsignedLongLong(value))
            .map(|text| text.to_string())
            .unwrap_or_else(|| format!("{value:0digits$}"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        format!("{value:0digits$}")
    }
}

/// Fixed precision belongs to the caller; grouping and decimal signs belong to the system.
pub(crate) fn decimal(value: f64, minimum: usize, maximum: usize) -> String {
    let legacy = format!("{value:.maximum$}");
    #[cfg(target_os = "macos")]
    if value.is_finite() {
        let formatter = cached(&DECIMAL, decimal_formatter);
        formatter.setMinimumIntegerDigits(0);
        formatter.setMinimumFractionDigits(minimum);
        formatter.setMaximumFractionDigits(maximum);
        let rounded = legacy.parse::<f64>().unwrap_or(value);
        if let Some(text) = formatter.stringFromNumber(&NSNumber::numberWithDouble(rounded)) {
            return text.to_string();
        }
    }
    if minimum == 0 && legacy.contains('.') {
        legacy
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_owned()
    } else {
        legacy
    }
}

pub(crate) fn percent(value: f64) -> String {
    #[cfg(target_os = "macos")]
    if value.is_finite() {
        let formatter = cached(&PERCENT, percent_formatter);
        formatter.setMinimumIntegerDigits(0);
        formatter.setMinimumFractionDigits(0);
        formatter.setMaximumFractionDigits(0);
        let rounded = format!("{value:.0}").parse::<f64>().unwrap_or(value);
        if let Some(text) = formatter.stringFromNumber(&NSNumber::numberWithDouble(rounded / 100.))
        {
            return text.to_string();
        }
    }
    format!("{value:.0}%")
}

pub(crate) fn usd(value: f64) -> String {
    #[cfg(target_os = "macos")]
    if value.is_finite() {
        let formatter = cached(&CURRENCY, currency_formatter);
        formatter.setMinimumIntegerDigits(0);
        formatter.setMinimumFractionDigits(2);
        formatter.setMaximumFractionDigits(2);
        // Preserve Rust's existing two-place rounding before localized presentation.
        let rounded = format!("{value:.2}").parse::<f64>().unwrap_or(value);
        if let Some(text) = formatter.stringFromNumber(&NSNumber::numberWithDouble(rounded)) {
            return text.to_string();
        }
    }
    format!("${value:.2}")
}

/// A template selects fields; Foundation selects their order and hour cycle.
pub(crate) fn date(timestamp_ms: i64, template: &str, legacy: &str) -> String {
    use chrono::DateTime;
    let Some(date) = DateTime::from_timestamp_millis(timestamp_ms) else {
        return String::new();
    };
    #[cfg(target_os = "macos")]
    {
        let _ = (date, legacy);
        // Only a few templates are used, so one small per-thread list is enough.
        let locale = NSLocale::currentLocale().localeIdentifier().to_string();
        let formatter = DATES.with(|cell| {
            let mut cell = cell.borrow_mut();
            if cell.0 != locale {
                *cell = (locale, Vec::new());
            }
            let entries = &mut cell.1;
            if let Some(ix) = entries.iter().position(|(key, _)| key == template) {
                return entries[ix].1.clone();
            }
            let formatter = NSDateFormatter::new();
            formatter.setLocale(Some(&NSLocale::currentLocale()));
            if template.is_empty() {
                formatter.setDateStyle(NSDateFormatterStyle::MediumStyle);
                formatter.setTimeStyle(NSDateFormatterStyle::MediumStyle);
            } else {
                formatter.setLocalizedDateFormatFromTemplate(&NSString::from_str(template));
            }
            entries.push((template.to_owned(), formatter.clone()));
            formatter
        });
        formatter
            .stringFromDate(&NSDate::dateWithTimeIntervalSince1970(
                timestamp_ms as f64 / 1000.,
            ))
            .to_string()
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = template;
        date.with_timezone(&chrono::Local)
            .format(legacy)
            .to_string()
    }
}

/// Preserve the pre-existing clock fallback on platforms not integrated in this iteration.
pub(crate) fn clock(timestamp_ms: u64) -> String {
    #[cfg(target_os = "macos")]
    {
        i64::try_from(timestamp_ms)
            .ok()
            .map(|at| date(at, "jm", "%H:%M"))
            .unwrap_or_default()
    }
    #[cfg(not(target_os = "macos"))]
    {
        monocode_core::response::local_clock(timestamp_ms)
    }
}

/// A machine HH:mm field is unchanged; its display follows the system hour cycle.
pub(crate) fn wall_time(value: &str) -> String {
    use chrono::TimeZone;
    chrono::NaiveTime::parse_from_str(value, "%H:%M")
        .ok()
        .and_then(|time| {
            chrono::Local
                .from_local_datetime(&chrono::Local::now().date_naive().and_time(time))
                .earliest()
        })
        .map(|at| date(at.timestamp_millis(), "jm", "%H:%M"))
        .unwrap_or_else(|| value.to_owned())
}

/// Accept a system-formatted clock while preserving the stored HH:mm contract.
pub(crate) fn parse_wall_time(value: &str) -> Option<chrono::NaiveTime> {
    let value = value.trim();
    if let Ok(time) = chrono::NaiveTime::parse_from_str(value, "%H:%M") {
        return Some(time);
    }
    #[cfg(target_os = "macos")]
    {
        let formatter = NSDateFormatter::new();
        formatter.setLocale(Some(&NSLocale::currentLocale()));
        formatter.setLocalizedDateFormatFromTemplate(&NSString::from_str("jm"));
        formatter.setLenient(false);
        parse_formatted_time(value, &formatter)
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
fn parse_formatted_time(value: &str, formatter: &NSDateFormatter) -> Option<chrono::NaiveTime> {
    let input = NSString::from_str(value);
    let mut range = objc2_foundation::NSRange::new(0, input.length());
    // Foundation's non-lenient parser still accepts zero as a 12-hour clock hour.
    unsafe { formatter.getObjectValue_forString_range_error(None, &input, &mut range) }.ok()?;
    if range.location != 0 || range.length != input.length() {
        return None;
    }
    let pattern = formatter.dateFormat().to_string();
    let mut quoted = false;
    let mut hour = None;
    let mut numeric_field = 0;
    let mut previous = '\0';
    for ch in pattern.chars() {
        if ch == '\'' {
            quoted = !quoted;
        } else if !quoted && ch != previous && matches!(ch, 'h' | 'H' | 'K' | 'k' | 'm' | 's') {
            if matches!(ch, 'h' | 'H' | 'K' | 'k') {
                hour = Some((numeric_field, ch));
                break;
            }
            numeric_field += 1;
        }
        previous = ch;
    }
    let (index, cycle) = hour?;
    let number = NSNumberFormatter::new();
    number.setLocale(Some(&formatter.locale()));
    number.setNumberStyle(NSNumberFormatterStyle::DecimalStyle);
    let local_digits: String = (0..10)
        .filter_map(|digit| number.stringFromNumber(&NSNumber::numberWithLongLong(digit)))
        .map(|text| text.to_string())
        .collect();
    let digits = value
        .split(|c: char| !c.is_numeric() && !local_digits.contains(c))
        .filter(|part| !part.is_empty())
        .nth(index)?;
    let value = number
        .numberFromString(&NSString::from_str(digits))?
        .longLongValue();
    let valid = match cycle {
        'h' => (1..=12).contains(&value),
        'K' => (0..=11).contains(&value),
        'H' => (0..=23).contains(&value),
        'k' => (1..=24).contains(&value),
        _ => false,
    };
    if !valid {
        return None;
    }
    let date = formatter.dateFromString(&input)?;
    chrono::DateTime::from_timestamp_millis((date.timeIntervalSince1970() * 1000.) as i64)
        .map(|at| at.with_timezone(&chrono::Local).time())
}

pub(crate) fn day(date: chrono::NaiveDate, template: &str, legacy: &str) -> String {
    use chrono::TimeZone;
    chrono::Local
        .from_local_datetime(&date.and_hms_opt(12, 0, 0).unwrap())
        .earliest()
        .map(|at| self::date(at.timestamp_millis(), template, legacy))
        .unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn calendar_noon(date: chrono::NaiveDate) -> Option<objc2::rc::Retained<NSDate>> {
    use chrono::TimeZone;
    let at = chrono::Local
        .from_local_datetime(&date.and_hms_opt(12, 0, 0)?)
        .earliest()?;
    Some(NSDate::dateWithTimeIntervalSince1970(at.timestamp() as f64))
}

#[cfg(target_os = "macos")]
fn calendar_date(date: &NSDate) -> Option<chrono::NaiveDate> {
    chrono::DateTime::from_timestamp_millis((date.timeIntervalSince1970() * 1000.) as i64)
        .map(|at| at.with_timezone(&chrono::Local).date_naive())
}

#[cfg(target_os = "macos")]
fn month_bounds_with_calendar(
    date: chrono::NaiveDate,
    calendar: &NSCalendar,
) -> Option<(chrono::NaiveDate, chrono::NaiveDate)> {
    let at = calendar_noon(date)?;
    let mut start = None;
    let mut seconds = 0.;
    // The pointers are local and Foundation fills them only on success.
    if !unsafe {
        calendar.rangeOfUnit_startDate_interval_forDate(
            NSCalendarUnit::Month,
            Some(&mut start),
            &mut seconds,
            &at,
        )
    } {
        return None;
    }
    let start = start?;
    let end = NSDate::dateWithTimeIntervalSince1970(start.timeIntervalSince1970() + seconds);
    Some((calendar_date(&start)?, calendar_date(&end)?))
}

pub(crate) fn month_bounds(
    date: chrono::NaiveDate,
) -> Option<(chrono::NaiveDate, chrono::NaiveDate)> {
    #[cfg(target_os = "macos")]
    {
        month_bounds_with_calendar(date, &NSCalendar::currentCalendar())
    }
    #[cfg(not(target_os = "macos"))]
    {
        use chrono::Datelike;
        let start = date.with_day(1)?;
        Some((start, start.checked_add_months(chrono::Months::new(1))?))
    }
}

#[cfg(target_os = "macos")]
fn shift_month_with_calendar(
    date: chrono::NaiveDate,
    amount: i32,
    calendar: &NSCalendar,
) -> Option<chrono::NaiveDate> {
    let at = calendar_noon(date)?;
    let shifted = calendar.dateByAddingUnit_value_toDate_options(
        NSCalendarUnit::Month,
        amount as isize,
        &at,
        NSCalendarOptions::empty(),
    )?;
    calendar_date(&shifted)
}

pub(crate) fn shift_month(date: chrono::NaiveDate, amount: i32) -> Option<chrono::NaiveDate> {
    #[cfg(target_os = "macos")]
    {
        shift_month_with_calendar(date, amount, &NSCalendar::currentCalendar())
    }
    #[cfg(not(target_os = "macos"))]
    {
        if amount < 0 {
            date.checked_sub_months(chrono::Months::new(amount.unsigned_abs()))
        } else {
            date.checked_add_months(chrono::Months::new(amount as u32))
        }
    }
}

pub(crate) fn calendar_day(date: chrono::NaiveDate) -> String {
    #[cfg(target_os = "macos")]
    {
        if let Some(at) = calendar_noon(date) {
            return unsigned(
                NSCalendar::currentCalendar().component_fromDate(NSCalendarUnit::Day, &at) as u64,
            );
        }
    }
    use chrono::Datelike;
    unsigned(date.day() as u64)
}

pub(crate) fn weekday(sunday_index: usize) -> String {
    let sunday = chrono::NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
    day(
        sunday + chrono::Duration::days(sunday_index as i64),
        "EEEE",
        "%A",
    )
}

#[cfg(all(test, target_os = "macos"))]
pub(crate) fn local_date_reference(timestamp_ms: i64, template: &str) -> String {
    use chrono::TimeZone;
    let local_day = chrono::DateTime::from_timestamp_millis(timestamp_ms)
        .unwrap()
        .with_timezone(&chrono::Local)
        .date_naive();
    let noon = chrono::Local
        .from_local_datetime(&local_day.and_hms_opt(12, 0, 0).unwrap())
        .single()
        .unwrap();
    let formatter = NSDateFormatter::new();
    formatter.setLocale(Some(&NSLocale::currentLocale()));
    formatter.setLocalizedDateFormatFromTemplate(&NSString::from_str(template));
    formatter
        .stringFromDate(&NSDate::dateWithTimeIntervalSince1970(
            noon.timestamp() as f64
        ))
        .to_string()
}

#[cfg(all(test, target_os = "macos"))]
pub(crate) fn controlled_test_locale() -> bool {
    let locale = if std::env::var_os("MONOCODE_TEST_12H").is_some() {
        "en_US"
    } else if std::env::var_os("MONOCODE_TEST_ISLAMIC").is_some() {
        "en_US@calendar=islamic"
    } else {
        return false;
    };
    use objc2::{class, msg_send, runtime::AnyObject};
    use objc2_foundation::NSDictionary;
    let key = NSString::from_str("AppleLocale");
    let value = NSString::from_str(locale);
    let domain = NSString::from_str("NSArgumentDomain");
    let values = NSDictionary::from_slices(&[&*key], &[&*value]);
    // This volatile defaults domain exists only in the test process.
    unsafe {
        let defaults: *mut AnyObject = msg_send![class!(NSUserDefaults), standardUserDefaults];
        let _: () = msg_send![defaults, setVolatileDomain: &*values, forName: &*domain];
    }
    assert_eq!(
        NSLocale::currentLocale().localeIdentifier().to_string(),
        locale
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_boundaries() {
        for value in [i64::MIN, -1, 0, 1234, i64::MAX] {
            assert!(!integer(value).is_empty());
        }
        assert_eq!(date(i64::MAX, "yMd", "%F"), "");
        #[cfg(target_os = "macos")]
        {
            assert_eq!(clock(u64::MAX), "");
            assert_eq!(clock(0), date(0, "jm", "%H:%M"));
        }
        assert_eq!(wall_time("invalid"), "invalid");
        assert!(!wall_time("09:30").is_empty());
        assert!(!date(0, "yMd", "%F").is_empty());
        assert_eq!(usd(f64::NAN), "$NaN");
        assert_eq!(usd(f64::INFINITY), "$inf");
        for value in [0., -1.125, 1234.567] {
            assert!(!usd(value).is_empty());
        }
    }
    #[test]
    #[cfg(target_os = "macos")]
    fn calendar_day_and_weekday_follow_system_without_shifting_business_date() {
        use chrono::TimeZone;
        let business_day = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let noon = chrono::Local
            .from_local_datetime(&business_day.and_hms_opt(12, 0, 0).unwrap())
            .single()
            .unwrap();
        assert_eq!(
            day(business_day, "yMMMd", "%F"),
            local_date_reference(noon.timestamp_millis(), "yMMMd")
        );
        let formatter = NSDateFormatter::new();
        formatter.setLocale(Some(&NSLocale::currentLocale()));
        let weekdays = formatter.weekdaySymbols();
        for index in 0..7 {
            assert_eq!(weekday(index), weekdays.objectAtIndex(index).to_string());
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn wall_clock_uses_system_hour_cycle() {
        use chrono::TimeZone;
        let local = chrono::Local
            .from_local_datetime(
                &chrono::Local::now()
                    .date_naive()
                    .and_hms_opt(9, 30, 0)
                    .unwrap(),
            )
            .earliest()
            .unwrap();
        let formatter = NSDateFormatter::new();
        formatter.setLocale(Some(&NSLocale::currentLocale()));
        formatter.setLocalizedDateFormatFromTemplate(&NSString::from_str("jm"));
        let expected = formatter
            .stringFromDate(&NSDate::dateWithTimeIntervalSince1970(
                local.timestamp() as f64
            ))
            .to_string();
        assert_eq!(wall_time("09:30"), expected);
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn localized_time_input_round_trips_and_rejects_invalid_values() {
        use chrono::TimeZone;
        for raw in ["09:30", "23:45"] {
            let expected = chrono::NaiveTime::parse_from_str(raw, "%H:%M").unwrap();
            assert_eq!(parse_wall_time(&wall_time(raw)), Some(expected));
            assert_eq!(parse_wall_time(raw), Some(expected));
        }
        let formatter = NSDateFormatter::new();
        formatter.setLocale(Some(&NSLocale::localeWithLocaleIdentifier(
            &NSString::from_str("en_US"),
        )));
        formatter.setLocalizedDateFormatFromTemplate(&NSString::from_str("hmma"));
        formatter.setLenient(false);
        assert_eq!(
            parse_formatted_time("9:30 PM", &formatter),
            chrono::NaiveTime::from_hms_opt(21, 30, 0)
        );
        assert_eq!(
            parse_formatted_time("09:30 pm", &formatter),
            chrono::NaiveTime::from_hms_opt(21, 30, 0)
        );
        for value in ["0:30 PM", "00:30 AM", "13:30 PM", "9:30 PM junk"] {
            assert!(parse_formatted_time(value, &formatter).is_none(), "{value}");
        }
        for (value, hour) in [("12:30 AM", 0), ("12:30 PM", 12)] {
            assert_eq!(
                parse_formatted_time(value, &formatter),
                chrono::NaiveTime::from_hms_opt(hour, 30, 0)
            );
        }
        let arabic = NSDateFormatter::new();
        arabic.setLocale(Some(&NSLocale::localeWithLocaleIdentifier(
            &NSString::from_str("ar_EG"),
        )));
        arabic.setLocalizedDateFormatFromTemplate(&NSString::from_str("hmma"));
        arabic.setLenient(false);
        let at = chrono::Local
            .from_local_datetime(
                &chrono::Local::now()
                    .date_naive()
                    .and_hms_opt(21, 30, 0)
                    .unwrap(),
            )
            .earliest()
            .unwrap();
        let text = arabic
            .stringFromDate(&NSDate::dateWithTimeIntervalSince1970(at.timestamp() as f64))
            .to_string();
        assert_eq!(
            parse_formatted_time(&text, &arabic),
            chrono::NaiveTime::from_hms_opt(21, 30, 0),
            "{text}"
        );
        let mut replaced = false;
        let invalid: String = text
            .chars()
            .map(|ch| {
                if !replaced && ch.is_numeric() {
                    replaced = true;
                    '٠'
                } else {
                    ch
                }
            })
            .collect();
        assert!(replaced);
        assert!(
            parse_formatted_time(&invalid, &arabic).is_none(),
            "{invalid}"
        );
        let han = NSDateFormatter::new();
        han.setLocale(Some(&NSLocale::localeWithLocaleIdentifier(
            &NSString::from_str("zh_CN@numbers=hanidec"),
        )));
        han.setLocalizedDateFormatFromTemplate(&NSString::from_str("hmma"));
        han.setLenient(false);
        let text = han
            .stringFromDate(&NSDate::dateWithTimeIntervalSince1970(at.timestamp() as f64))
            .to_string();
        assert_eq!(
            parse_formatted_time(&text, &han),
            chrono::NaiveTime::from_hms_opt(21, 30, 0),
            "{text}"
        );
        assert!(parse_wall_time("25:00").is_none());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn islamic_calendar_grid_selection_and_month_navigation() {
        use chrono::TimeZone;
        let calendar = NSCalendar::calendarWithIdentifier(&NSString::from_str("islamic")).unwrap();
        let selected = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let (start, end) = month_bounds_with_calendar(selected, &calendar).unwrap();
        assert_eq!(
            start,
            chrono::NaiveDate::from_ymd_opt(2025, 12, 21).unwrap()
        );
        assert_eq!(end, chrono::NaiveDate::from_ymd_opt(2026, 1, 19).unwrap());
        let day =
            calendar.component_fromDate(NSCalendarUnit::Day, &calendar_noon(selected).unwrap());
        assert_eq!(day, 12);
        let clicked = start + chrono::Duration::days(day as i64 - 1);
        assert_eq!(clicked, selected);
        let saved = chrono::Local
            .from_local_datetime(&clicked.and_hms_opt(9, 30, 0).unwrap())
            .earliest()
            .unwrap();
        assert_eq!(saved.date_naive(), selected);
        let next = month_bounds_with_calendar(end, &calendar).unwrap();
        assert_eq!(next.0, end);
        assert_eq!(
            month_bounds_with_calendar(start - chrono::Duration::days(1), &calendar)
                .unwrap()
                .1,
            start
        );
        assert_eq!(
            shift_month_with_calendar(selected, 1, &calendar).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 30).unwrap()
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn display_numbers_match_foundation_preferences_and_usd() {
        let formatter = NSNumberFormatter::new();
        formatter.setLocale(Some(&NSLocale::currentLocale()));
        formatter.setNumberStyle(NSNumberFormatterStyle::DecimalStyle);
        formatter.setMinimumFractionDigits(2);
        formatter.setMaximumFractionDigits(2);
        assert_eq!(
            decimal(1234.567, 2, 2),
            formatter
                .stringFromNumber(&NSNumber::numberWithDouble(1234.57))
                .unwrap()
                .to_string()
        );
        formatter.setNumberStyle(NSNumberFormatterStyle::PercentStyle);
        formatter.setMinimumFractionDigits(0);
        formatter.setMaximumFractionDigits(0);
        assert_eq!(
            percent(25.),
            formatter
                .stringFromNumber(&NSNumber::numberWithDouble(0.25))
                .unwrap()
                .to_string()
        );
        formatter.setNumberStyle(NSNumberFormatterStyle::CurrencyStyle);
        formatter.setCurrencyCode(Some(&NSString::from_str("USD")));
        formatter.setMinimumFractionDigits(2);
        formatter.setMaximumFractionDigits(2);
        assert_eq!(
            usd(1234.567),
            formatter
                .stringFromNumber(&NSNumber::numberWithDouble(1234.57))
                .unwrap()
                .to_string()
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn reused_formatters_do_not_leak_settings_between_calls() {
        // The decimal formatter is shared by integer, padded_unsigned and
        // decimal; each call must set the properties it depends on.
        let padded = padded_unsigned(5, 2);
        assert_eq!(
            padded,
            cached_reference(|f| {
                f.setMinimumIntegerDigits(2);
                f.setMinimumFractionDigits(0);
                f.setMaximumFractionDigits(0);
                f.stringFromNumber(&NSNumber::numberWithUnsignedLongLong(5))
                    .unwrap()
                    .to_string()
            }),
            "padded form"
        );
        // After a padded call, an unpadded one must not keep the two digits.
        let plain = integer(5);
        assert_ne!(plain, padded);
        assert_eq!(
            plain,
            cached_reference(|f| {
                f.setMinimumIntegerDigits(0);
                f.setMinimumFractionDigits(0);
                f.setMaximumFractionDigits(0);
                f.stringFromNumber(&NSNumber::numberWithLongLong(5))
                    .unwrap()
                    .to_string()
            })
        );
        // And decimals must not inherit the integer-only fraction limit.
        let fraction = decimal(1.5, 0, 1);
        assert_eq!(
            fraction,
            cached_reference(|f| {
                f.setMinimumIntegerDigits(0);
                f.setMinimumFractionDigits(0);
                f.setMaximumFractionDigits(1);
                f.stringFromNumber(&NSNumber::numberWithDouble(1.5))
                    .unwrap()
                    .to_string()
            })
        );
        assert_ne!(decimal(1.5, 0, 1), "2");
        // Repeated calls must stay stable rather than drift with shared state.
        for _ in 0..3 {
            assert_eq!(integer(1234), integer(1234));
            assert_eq!(percent(25.), percent(25.));
            assert_eq!(usd(1.125), usd(1.125));
        }
        assert_eq!(
            unsigned(0),
            cached_reference(|f| {
                f.setMinimumIntegerDigits(1);
                f.setMinimumFractionDigits(0);
                f.setMaximumFractionDigits(0);
                f.stringFromNumber(&NSNumber::numberWithUnsignedLongLong(0))
                    .unwrap()
                    .to_string()
            })
        );
        assert_eq!(clock(0), date(0, "jm", "%H:%M"));
        assert_eq!(
            day(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                "yMMMd",
                "%F"
            ),
            day(
                chrono::NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                "yMMMd",
                "%F"
            )
        );
    }

    /// A freshly built date formatter for the same timestamp, never the cache.
    #[cfg(target_os = "macos")]
    fn fresh_date_reference(timestamp_ms: i64, template: &str) -> String {
        let formatter = NSDateFormatter::new();
        formatter.setLocale(Some(&NSLocale::currentLocale()));
        formatter.setLocalizedDateFormatFromTemplate(&NSString::from_str(template));
        formatter
            .stringFromDate(&NSDate::dateWithTimeIntervalSince1970(
                timestamp_ms as f64 / 1000.,
            ))
            .to_string()
    }

    /// A standalone formatter for expected values, never the cached one.
    #[cfg(target_os = "macos")]
    fn cached_reference(f: impl FnOnce(&NSNumberFormatter) -> String) -> String {
        let formatter = NSNumberFormatter::new();
        formatter.setLocale(Some(&NSLocale::currentLocale()));
        formatter.setNumberStyle(NSNumberFormatterStyle::DecimalStyle);
        f(&formatter)
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn reused_date_formatters_follow_each_template() {
        // Two templates on one shared list must not return each other's output.
        let day_first = date(1_767_310_200_000, "yMd", "%F");
        let time_first = date(1_767_310_200_000, "jm", "%H:%M");
        assert_ne!(day_first, time_first);
        assert_eq!(day_first, date(1_767_310_200_000, "yMd", "%F"));
        assert_eq!(time_first, date(1_767_310_200_000, "jm", "%H:%M"));
        assert_eq!(
            time_first,
            fresh_date_reference(1_767_310_200_000, "jm"),
            "the cached formatter matches a freshly built one"
        );
        assert_eq!(
            day_first,
            fresh_date_reference(1_767_310_200_000, "yMd"),
            "and so does the other template"
        );
        // The empty template uses the medium date/time styles rather than a
        // localized template.
        assert!(!date(0, "", "%c").is_empty());
        assert_eq!(date(0, "", "%c"), date(0, "", "%c"));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn system_numbers_round_trip_without_currency_or_precision_changes() {
        let parser = NSNumberFormatter::new();
        parser.setLocale(Some(&NSLocale::currentLocale()));
        parser.setNumberStyle(NSNumberFormatterStyle::DecimalStyle);
        for value in [i64::MIN, -1, 0, 1234567, i64::MAX] {
            let text = integer(value);
            let number = parser.numberFromString(&NSString::from_str(&text)).unwrap();
            assert_eq!(number.longLongValue(), value, "{text}");
        }
        // Parsing via NSNumberFormatter loses precision above i64::MAX; compare rendered digits.
        for value in [0, 1234567, u64::MAX] {
            let text = unsigned(value);
            let digits = value
                .to_string()
                .chars()
                .map(|digit| unsigned(digit.to_digit(10).unwrap() as u64))
                .collect::<String>();
            assert_eq!(
                text.replace(&parser.groupingSeparator().to_string(), ""),
                digits
            );
        }
        for (value, minimum, maximum) in [(0.25, 0, 2), (12345.625, 2, 2), (-1.125, 0, 2)] {
            let text = decimal(value, minimum, maximum);
            let number = parser.numberFromString(&NSString::from_str(&text)).unwrap();
            assert_eq!(
                number.doubleValue(),
                format!("{value:.maximum$}").parse::<f64>().unwrap()
            );
        }
        parser.setNumberStyle(NSNumberFormatterStyle::PercentStyle);
        for value in [-12.5, 0., 12.5, 99.9, 100.] {
            let text = percent(value);
            let number = parser.numberFromString(&NSString::from_str(&text)).unwrap();
            assert_eq!(
                number.doubleValue(),
                format!("{value:.0}").parse::<f64>().unwrap() / 100.,
                "{text}"
            );
        }
        parser.setNumberStyle(NSNumberFormatterStyle::CurrencyStyle);
        parser.setCurrencyCode(Some(&NSString::from_str("USD")));
        for value in [0., -1.125, 1.125, 1.135, 1234.567, 999999.995] {
            let text = usd(value);
            let number = parser.numberFromString(&NSString::from_str(&text)).unwrap();
            assert_eq!(
                number.doubleValue(),
                format!("{value:.2}").parse::<f64>().unwrap(),
                "{text}"
            );
        }
        assert_eq!(crate::i18n::text("settings-sounds"), "Sounds");
    }
}
