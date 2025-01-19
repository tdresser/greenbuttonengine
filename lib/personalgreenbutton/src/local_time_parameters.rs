use chrono::Datelike;
use chrono::Days;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::TimeDelta;
use chrono::Weekday;
use roxmltree::Node;

use crate::parse_helpers::parse_text_of;
use crate::parse_helpers::strip_espi_prefix;
use anyhow::anyhow;
use anyhow::Result;
use columnar_struct_vec::columnar_struct_vec;

#[derive(Debug)]
#[columnar_struct_vec]
pub struct LocalTimeParameters {
    pub dst_start_rule: u32,
    pub dst_end_rule: u32,
    pub dst_offset: i64,
    pub tz_offset: i64,
}

pub struct LocalTimeParametersSingle {
    pub dst_start_rule: u32,
    pub dst_end_rule: u32,
    pub dst_offset: TimeDelta,
    pub tz_offset: TimeDelta,
}

/*
The operators:

0: DST starts/ends on the Day of the Month
1: DST starts/ends on the Day of the Week that is on or after the Day of the Month
2: DST starts/ends on the first occurrence of the Day of the Week in a month
3: DST starts/ends on the second occurrence of the Day of the Week in a month
4: DST starts/ends on the third occurrence of the Day of the Week in a month
5: DST starts/ends on the forth occurrence of the Day of the Week in a month
6: DST starts/ends on the fifth occurrence of the Day of the Week in a month
7: DST starts/ends on the last occurrence of the Day of the Week in a month  */

fn get_date(
    year: i32,
    day_of_week: Weekday,
    day_of_month: u32,
    operator: u32,
    month: u32,
) -> Option<NaiveDate> {
    return match operator {
        // 0: DST starts/ends on the Day of the Month.
        0 => NaiveDate::from_ymd_opt(year, month, day_of_month),
        // 1: DST starts/ends on the Day of the Week that is on or after the Day of the Month.
        1 => {
            let date = NaiveDate::from_ymd_opt(year, month, day_of_month)?;
            let day_offset = day_of_week.days_since(date.weekday());
            date.checked_add_days(Days::new(day_offset as u64))
        }
        // 7: DST starts/ends on the last occurrence of the Day of the Week in a month.
        7 => {
            let last_day_of_month = if month == 12 {
                NaiveDate::from_ymd_opt(year, month, 31)
            } else {
                NaiveDate::from_ymd_opt(year, month + 1, 1)?.pred_opt()
            };

            // Start from the last day of the month and work backwards to find the last correct weekday.
            for date in last_day_of_month?.iter_days().rev() {
                if date.weekday() == day_of_week {
                    return Some(date);
                }
            }
            return None;
        }
        // 2: DST starts/ends on the first occurrence of the Day of the Week in a month
        // 3: DST starts/ends on the second occurrence of the Day of the Week in a month
        // 4: DST starts/ends on the third occurrence of the Day of the Week in a month
        // 5: DST starts/ends on the forth occurrence of the Day of the Week in a month
        // 6: DST starts/ends on the fifth occurrence of the Day of the Week in a month
        _ => {
            let first_day_of_month = NaiveDate::from_ymd_opt(year, month, 1)?;
            let first_weekday_offset = day_of_week.days_since(first_day_of_month.weekday());
            let first_weekday_date = first_day_of_month + Days::new(first_weekday_offset as u64);
            let nth = operator - 2;
            return Some(first_weekday_date + chrono::Duration::days(7 * nth as i64));
        }
    };
}

fn get_datetime(
    year: i32,
    seconds: u32,
    hours: u32,
    day_of_week: Weekday,
    day_of_month: u32,
    operator: u32,
    month: u32,
) -> Option<NaiveDateTime> {
    let date = get_date(year, day_of_week, day_of_month, operator, month);
    if let Some(date) = date {
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        return date.and_hms_opt(hours, minutes, seconds);
    }
    return None;
}

/*
https://www.greenbuttonalliance.org/daylight-savings-time
The rule encoding:
Bits  0 - 11: seconds 0 - 3599
Bits 12 - 16: hours 0 - 23
Bits 17 - 19: day of the week 0 = not applicable, 1 - 7 (Monday = 1)
Bits:20 - 24: day of the month 0 = not applicable, 1 - 31
Bits: 25 - 27: operator  (detailed below)
Bits: 28 - 31: month 1 - 12 */
// Bit twiddling inspired by https://github.com/VerdigrisTech/green-button-data/blob/master/lib/green-button-data/dst.rb .
pub fn get_date_from_dst_rule(rule: u32, year: i32) -> Result<Option<NaiveDateTime>> {
    if rule == 0xFFFFFFFF {
        return Ok(None);
    }

    let seconds = rule & 0x00000fff;
    let hours = (rule & 0x0001f000) >> 12;
    let day_of_week = Weekday::try_from((((rule & 0x000e0000) >> 17) as u8 + 1) % 7)?;
    let day_of_month = (rule & 0x01f00000) >> 20;
    let operator = (rule & 0x0e000000) >> 25;
    let month = (rule & 0xf0000000) >> 28;

    if !(seconds <= 3599 && hours <= 23 && day_of_month <= 31 && operator <= 7 && month <= 12) {
        return Err(anyhow!("Invalid dst rule in LocalTimeParameters"));
    }

    return Ok(get_datetime(
        year,
        seconds,
        hours,
        day_of_week,
        day_of_month,
        operator,
        month,
    ));
}

pub fn parse_local_time_parameters(
    mut local_time_parameters: LocalTimeParameters,
    node: Node,
) -> Result<LocalTimeParameters> {
    let mut ltp = local_time_parameters.start_push();
    for child in node.children() {
        match strip_espi_prefix(child.tag_name().name()) {
            "dstStartRule" => {
                let text: String = parse_text_of(child)?;
                ltp.dst_start_rule(u32::from_str_radix(&text, 16)?);
            }
            "dstEndRule" => {
                let text: String = parse_text_of(child)?;
                ltp.dst_end_rule(u32::from_str_radix(&text, 16)?);
            }
            "dstOffset" => {
                ltp.dst_offset(parse_text_of(child)?);
            }
            "tzOffset" => {
                ltp.tz_offset(parse_text_of(child)?);
            }
            _ => {
                if child.tag_name().name().len() > 0 {
                    return Err(anyhow!("Unmatched tag name: {:?}", child.tag_name().name()));
                }
            }
        }
    }
    local_time_parameters = ltp.finalize_push()?;
    return Ok(local_time_parameters);
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};

    use crate::local_time_parameters::{get_date, get_datetime};

    use super::get_date_from_dst_rule;

    // 0: DST starts/ends on the Day of the Month
    #[test]
    fn operator0() {
        let date = get_date(2025, chrono::Weekday::Tue /* ignored */, 18, 0, 6).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 6, 18).unwrap());
    }

    // 1: DST starts/ends on the Day of the Week that is on or after the Day of the Month.
    #[test]
    fn operator1() {
        let date = get_date(2025, chrono::Weekday::Tue, 14, 1, 2).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 2, 18).unwrap());
    }

    #[test]
    fn operator1_equals() {
        let date = get_date(2025, chrono::Weekday::Tue, 11, 1, 2).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 2, 11).unwrap());
    }

    // 7: DST starts/ends on the last occurrence of the Day of the Week in a month.
    #[test]
    fn operator7() {
        // Last occurance of Tuesday in February.
        let date = get_date(2025, chrono::Weekday::Tue, 1 /* ignored */, 7, 2).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 2, 25).unwrap());
    }

    #[test]
    fn operator7_december() {
        // Last occurance of Tuesday in December.
        let date = get_date(2025, chrono::Weekday::Tue, 1 /* ignored */, 7, 12).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 12, 30).unwrap());
    }

    // 4: DST starts/ends on the third occurrence of the Day of the Week in a month
    #[test]
    fn operator4() {
        // Third Tuesday of the month.
        let date = get_date(2025, chrono::Weekday::Tue, 1 /* ignored */, 4, 2).unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2025, 2, 18).unwrap());
    }

    #[test]
    fn get_datetime_test() {
        let date = get_datetime(
            2025,
            3012,
            2,
            chrono::Weekday::Tue, /* ignored */
            18,
            0,
            6,
        )
        .unwrap();
        assert_eq!(
            date,
            NaiveDateTime::parse_from_str("2025-06-18 02:50:12", "%Y-%m-%d %H:%M:%S").unwrap()
        );
    }

    #[test]
    fn documentation_ex1() {
        // https://www.greenbuttonalliance.org/daylight-savings-time
        let rule = u32::from_str_radix("360E2000", 16).unwrap();
        assert_eq!(
            get_date_from_dst_rule(rule, 2020).unwrap().unwrap(),
            NaiveDateTime::parse_from_str("2020-03-10 02:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
    }
}
