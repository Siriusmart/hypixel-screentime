use std::{collections::HashMap, fmt::Display, sync::OnceLock};

use chrono::{DateTime, Datelike, NaiveTime, Timelike, Utc};

use crate::{Record, Storage, web::HEART};

pub struct Mermaid {
    weekly: Weekly,
    daily: Daily,
    timeofday: TimeOfDay,
}

// reversed order
pub struct Daily(pub [u64; 7]);
pub struct Weekly(pub [u64; 10]);
pub struct TimeOfDay(pub [u64; 24]);

pub static mut MERMAID_DATA: OnceLock<HashMap<String, Mermaid>> = OnceLock::new();

impl Mermaid {
    pub fn html(username: &str) -> String {
        match Self::get().get(username) {
            Some(user) => format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{username} - Hypixel Screentime</title>
</head>
<body>
<script type="module">
  import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';
  mermaid.initialize({{ startOnLoad: true, theme: "dark" }});
</script>
<style>
body {{
    background: #333333;
    text-align: center;
    color: #eceff4;
    font-family: Arial, Helvetica, sans-serif;
}}

.green {{
    color: #a6e3a1 !important;
}}

table {{
    margin: auto;
}}

table * * {{
    padding: 10px;
}}

#home {{
    text-decoration: none;
    padding: 20px;
    position: absolute;
    top: 0px;
    left: 0px;
}}

#home {{
    color: #eceff4;
    transition: 100ms;
}}

#home:hover {{
    transition: 100ms;
    color: #ee99a0;
}}
</style>
<a id="home" href="/">&gt;Back Home</a>
<h1{}>{username}{}</h1>
<pre class="mermaid">
  {}
</pre>
<pre class="mermaid">
  {}
</pre>
<pre class="mermaid">
  {}
</pre>

<h2>Login Records</h2>
<table>
<tr>
  <th>Login</th>
  <th>Logout</th>
</tr>
{}
</table>
<br>
<br>
<footer>
  <p>Hypixel Screentime by <i><b>Sirius</b></i> | <span style="border-bottom: 2px solid #a6e3a1;"><a class="green" target="_blank" href="https://github.com/siriusmart/hypixel-screentime" style="text-decoration: none;">Written with {HEART} in Rust</a></span></p>
</footer>
</body>
</html>"#,
                if Storage::is_online(username) {
                    " class=\"green\""
                } else {
                    ""
                },
                if Storage::is_online(username) {
                    " (online)"
                } else {
                    ""
                },
                user.timeofday,
                user.daily,
                user.weekly,
                Storage::print_log(username)
            ),
            None => "No such user".to_string(),
        }
    }

    pub fn get() -> &'static HashMap<String, Self> {
        unsafe { MERMAID_DATA.get_or_init(Self::init) }
    }

    pub fn update() {
        unsafe { *MERMAID_DATA.get_mut().unwrap() = Self::init() }
    }

    pub fn init() -> HashMap<String, Mermaid> {
        Storage::copy()
            .users
            .iter()
            .map(|(name, entry)| (name.clone(), Self::build(entry)))
            .collect()
    }

    pub unsafe fn build_all() {
        if MERMAID_DATA.get().is_some() {
            *MERMAID_DATA.get_mut().unwrap() = Self::init()
        } else {
            MERMAID_DATA.set(Self::init());
        }
    }

    pub fn build(records: &Vec<Record>) -> Self {
        let now = chrono::Utc::now();

        let mut daily = [0; 7];
        let mut weekly = [0; 10];
        let mut timeofday = [0; 24];

        records.iter().for_each(|record| {
            if record.end.is_none() {
                return;
            }

            let mut push = |start: DateTime<Utc>, end: DateTime<Utc>| {
                let duration = end.signed_duration_since(start).num_seconds() as u64;
                daily[start.weekday().num_days_from_monday() as usize] += duration;
                weekly[(now - start).num_weeks().min(6) as usize] += duration;

                if start.hour() == end.hour() {
                    timeofday[start.hour() as usize] += duration;
                } else {
                    let start_roundup = start.with_minute(0).unwrap().with_second(0).unwrap()
                        + chrono::Duration::hours(1);
                    let end_rounddown = end.with_minute(0).unwrap().with_second(0).unwrap();

                    timeofday[start.hour() as usize] +=
                        start_roundup.signed_duration_since(start).num_seconds() as u64;
                    timeofday[end.hour() as usize] +=
                        end.signed_duration_since(end_rounddown).num_seconds() as u64;

                    for hour in start_roundup.hour()..end_rounddown.hour() {
                        timeofday[hour as usize] += 3600;
                    }
                }
            };

            let start = chrono::DateTime::from_timestamp_millis(record.beginning as i64).unwrap();
            let end = chrono::DateTime::from_timestamp_millis(
                record.end.unwrap_or(now.timestamp_millis() as u64) as i64,
            )
            .unwrap();

            if start.date_naive() == end.date_naive() {
                push(start, end);
            } else {
                let start_roundup = start
                    .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
                    .unwrap()
                    + chrono::Duration::days(1);
                let end_rounddown = end.with_minute(0).unwrap().with_second(0).unwrap();

                push(start, start_roundup);
                push(end_rounddown, end);

                for day in 0..end_rounddown.num_days_from_ce() - start_roundup.num_days_from_ce() {
                    push(
                        start_roundup + chrono::Duration::days(day as i64),
                        start_roundup + chrono::Duration::days(day as i64 + 1),
                    )
                }
            }
        });

        Self {
            weekly: Weekly(weekly),
            daily: Daily(daily),
            timeofday: TimeOfDay(timeofday),
        }
    }
}

impl Display for Weekly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first_fetch =
            chrono::DateTime::from_timestamp(Storage::copy().first_fetch as i64, 0).unwrap();
        let weekly = self
            .0
            .iter()
            .rev()
            .map(|time| {
                *time as f32 / ((chrono::Utc::now() - first_fetch).num_weeks() + 1) as f32 / 3600.
            })
            .collect::<Vec<_>>();
        f.write_fmt(format_args!(
            r#"xychart-beta
     title "Weekly login"
     x-axis [-9, -8, -7, -6, -5, -4, -3, -2, -1, 0]
     y-axis "Avg. hours online"
     bar {}
"#,
            serde_json::to_string(&weekly).unwrap()
        ))
    }
}

impl Display for TimeOfDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first_fetch =
            chrono::DateTime::from_timestamp(Storage::copy().first_fetch as i64, 0).unwrap();
        let timeofday = self
            .0
            .iter()
            .map(|time| {
                *time as f32 / ((chrono::Utc::now() - first_fetch).num_days() + 1) as f32 / 60.
            })
            .collect::<Vec<_>>();
        f.write_fmt(format_args!(r#"xychart-beta
    title "Time of day (GMT)"
    x-axis [00, 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
    y-axis "Avg. minutes online"
    bar {}
"#, serde_json::to_string(&timeofday).unwrap()))
    }
}

impl Display for Daily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first_fetch =
            chrono::DateTime::from_timestamp(Storage::copy().first_fetch as i64, 0).unwrap();
        let daily = self
            .0
            .iter()
            .map(|time| {
                *time as f32 / ((chrono::Utc::now() - first_fetch).num_weeks() + 1) as f32 / 3600.
            })
            .collect::<Vec<_>>();
        f.write_fmt(format_args!(
            r#"xychart-beta
     title "Day of week"
     x-axis [1, 2, 3, 4, 5, 6, 7]
     y-axis "Avg. hours online"
     bar {}
"#,
            serde_json::to_string(&daily).unwrap()
        ))
    }
}

// xychart-beta
//     title "Weekly login"
//     x-axis [-10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 0]
//     y-axis "Hours online"
//     bar [0.1, 0.2]

// xychart-beta
//     title "Time of day (GMT + 1)"
//     x-axis [00, 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
//     y-axis "Hours online"
//     bar [0.1, 0.2]

// xychart-beta
//     title "Day of week"
//     x-axis [1, 2, 3, 4, 5, 6, 7]
//     y-axis "Hours online"
//     bar [0.1, 0.2]
