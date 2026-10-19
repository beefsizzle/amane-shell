use std::time::{Duration, Instant};

use amane::{Audio, Service};

/*
 * the speaker and microphone levels as the user sees them. setting a level asks PulseAudio, and the Audio service
 * only shows the new one once the sound server has announced it back. so a level that was just dragged or scrolled to
 * is shown straight away, for a moment, instead of waiting. this also makes fast scrolling add up: every notch starts
 * from the level just set, not from the last one the sound server got round to announcing
 */
#[derive(Clone, Copy)]
pub enum Level {
    Output,
    Input,
}

// how long a level that was just set wins over what the sound server says
const CONFIRM_WITHIN: Duration = Duration::from_millis(600);

// a scroll notch, in percent
const NOTCH: i32 = 5;

#[derive(Default)]
pub struct Pending {
    set: [Option<(u8, Instant)>; 2], // by `Level`: the percent just set, and when
    expiring: [bool; 2],             // a thread will clear it once it's old, see `expire`
}

impl Service for Pending {
    fn new() -> Self {
        Self::default()
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

impl Level {
    /// what to show: the level just set while it's fresh, otherwise `actual`, what the sound server says. the caller
    /// passes that in because views already hold the Audio service, and reading it twice on one thread can deadlock
    pub fn shown(self, actual: u8) -> u8 {
        Pending::read().set[self as usize].map_or(actual, |(percent, _)| percent)
    }

    pub fn set(self, percent: u8) {
        let percent = percent.min(100);

        {
            let mut pending = Pending::write();

            pending.set[self as usize] = Some((percent, Instant::now()));

            if !pending.expiring[self as usize] {
                pending.expiring[self as usize] = true;

                let level = self;

                std::thread::spawn(move || expire(level));
            }
        }

        match self {
            Level::Output => Audio::set_volume(percent),
            Level::Input => Audio::set_microphone_volume(percent),
        }
    }

    /// moves the level by whole wheel notches, up is positive
    pub fn step(self, notches: i32) {
        if notches == 0 {
            return;
        }

        let actual = {
            let audio = Audio::read();

            match self {
                Level::Output => audio.volume(),
                Level::Input => audio.microphone_volume(),
            }
        };

        let next = i32::from(self.shown(actual)) + notches * NOTCH;

        self.set(next.clamp(0, 100) as u8);
    }
}

/*
 * clears a level once nothing has set it for CONFIRM_WITHIN, so the views redraw with what the sound server says.
 * without this, a level the sound server never took (one it clamps, or refuses) would stay on screen until something
 * else redrew. one thread per level, however many drag events there are
 */
fn expire(level: Level) {
    loop {
        let left = match Pending::read().set[level as usize] {
            Some((_, at)) => CONFIRM_WITHIN.saturating_sub(at.elapsed()),
            None => Duration::ZERO,
        };

        if !left.is_zero() {
            std::thread::sleep(left);

            continue;
        }

        let mut pending = Pending::write();

        // set again just now: keep waiting
        if pending.set[level as usize].is_some_and(|(_, at)| at.elapsed() < CONFIRM_WITHIN) {
            continue;
        }

        pending.set[level as usize] = None;
        pending.expiring[level as usize] = false;

        return;
    }
}

// for sliders, which take plain functions
pub fn drag_output(value: f32) {
    Level::Output.set((value * 100.0).round() as u8);
}

pub fn drag_input(value: f32) {
    Level::Input.set((value * 100.0).round() as u8);
}

pub fn scroll_output(notches: i32) {
    Level::Output.step(notches);
}

pub fn scroll_input(notches: i32) {
    Level::Input.step(notches);
}
