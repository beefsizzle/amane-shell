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
        let pending = Pending::read().set[self as usize];

        pending.filter(|(_, at)| at.elapsed() < CONFIRM_WITHIN).map_or(actual, |(percent, _)| percent)
    }

    pub fn set(self, percent: u8) {
        let percent = percent.min(100);

        Pending::write().set[self as usize] = Some((percent, Instant::now()));

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
