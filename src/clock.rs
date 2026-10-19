use std::time::Duration;

use amane::Service;

pub struct Clock {
    pub time: String,   // 17:52
    pub date: String,   // Sun 18 Oct
    pub long_date: String, // Sunday 18 October
    pub year: i32,
    pub month: u32,
    pub day: u32,
    seconds: u32,
}

fn read() -> Clock {
    // one `date` call, so every field comes from the same instant
    let line = amane::output("date '+%H:%M|%a %d %b|%Y|%m|%d|%A %d %B|%S'");

    let mut parts = line.trim().split('|');

    let time = parts.next().unwrap_or("").to_string();
    let date = parts.next().unwrap_or("").to_string();
    let year = parts.next().and_then(|v| v.parse().ok()).unwrap_or(2026);
    let month = parts.next().and_then(|v| v.parse().ok()).unwrap_or(1);
    let day = parts.next().and_then(|v| v.parse().ok()).unwrap_or(1);
    let long_date = parts.next().unwrap_or("").to_string();
    let seconds = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);

    Clock { time, date, long_date, year, month, day, seconds }
}

impl Service for Clock {
    fn new() -> Self {
        read()
    }

    /*
     * `date` runs before the lock is taken (`update()` holds it, and every bar reads the clock), and only about
     * once at each minute change instead of every second. it wakes a little more often than that so a clock that
     * was asleep (suspend doesn't advance the timer) corrects itself within seconds of waking
     */
    fn listen() {
        loop {
            let now = read();

            let seconds_left = 60u64.saturating_sub(u64::from(now.seconds)).clamp(1, 60);

            let changed = {
                let current = Self::read();

                now.time != current.time || now.day != current.day
            };

            if changed {
                *Self::write() = now;
            }

            std::thread::sleep(Duration::from_secs(seconds_left.min(10)));
        }
    }
}
