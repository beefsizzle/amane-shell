use std::time::{Duration, Instant};

use amane::{Network, Service, TextInput};

/*
 * the Wi-Fi part of the network dropdown: which network asks for its password, which one is being joined, and
 * which one just failed. NetworkManager does the joining (through amane's `Network`), this only keeps what the
 * panel needs to say about it
 */
#[derive(Default)]
pub struct Wifi {
    pub prompt: Option<String>,  // the secured network whose password field is open
    pub password: String,        // what's typed in it so far
    pub joining: Option<String>, // the network being joined from the panel
    pub failed: Option<Failure>, // the last network that didn't join
}

#[derive(Clone, PartialEq)]
pub struct Failure {
    pub ssid: String,
    pub wrong_password: bool, // a password typed in the panel, as opposed to a saved one that stopped working
}

impl Service for Wifi {
    fn new() -> Self {
        Self::default()
    }

    fn interval() -> Duration {
        Duration::from_secs(3600)
    }
}

pub const PASSWORD_FIELD: &str = "wifi-password";

// the bar and panel glyph for a signal strength, 0 to 100
pub fn glyph(strength: u8) -> &'static str {
    match strength {
        0..=19 => "\u{f092f}",  // outline, no bars
        20..=44 => "\u{f091f}", // one bar
        45..=69 => "\u{f0922}",
        70..=84 => "\u{f0925}",
        _ => "\u{f0928}", // full
    }
}

/// the panel opened: a fresh look at the networks around, and no leftovers from last time
pub fn opened() {
    Network::scan();

    let mut wifi = Wifi::write();

    wifi.prompt = None;
    wifi.password.clear();

    if wifi.joining.is_none() {
        wifi.failed = None;
    }
}

/// opens the password field under a secured network, or closes it when it's already open there
pub fn ask(ssid: &str) {
    let mut wifi = Wifi::write();

    if wifi.prompt.as_deref() == Some(ssid) {
        wifi.prompt = None;
    } else {
        wifi.prompt = Some(ssid.to_string());
        wifi.failed = None;
    }

    wifi.password.clear();

    drop(wifi);

    TextInput::set_text(PASSWORD_FIELD, "");
}

pub fn cancel() {
    let mut wifi = Wifi::write();

    wifi.prompt = None;
    wifi.password.clear();

    drop(wifi);

    TextInput::set_text(PASSWORD_FIELD, "");
}

/// joins with what's typed in the password field
pub fn submit() {
    let (ssid, password) = {
        let wifi = Wifi::read();

        let Some(ssid) = wifi.prompt.clone() else { return };

        (ssid, wifi.password.clone())
    };

    // WPA needs at least eight characters, NetworkManager would refuse it anyway
    if password.chars().count() < 8 {
        return;
    }

    join(&ssid, Some(password));
}

/// joins a network: a saved or open one with `None`, a new secured one with its password
pub fn join(ssid: &str, password: Option<String>) {
    {
        let mut wifi = Wifi::write();

        if wifi.joining.is_some() {
            return;
        }

        wifi.prompt = None;
        wifi.password.clear();
        wifi.joining = Some(ssid.to_string());
        wifi.failed = None;
    }

    TextInput::set_text(PASSWORD_FIELD, "");

    let typed = password.is_some();

    Network::connect(ssid, password.as_deref());

    let ssid = ssid.to_string();

    std::thread::spawn(move || watch(ssid, typed));
}

/// forgets a saved network, password and all
pub fn forget(ssid: &str) {
    Network::forget(ssid);

    let mut wifi = Wifi::write();

    if wifi.failed.as_ref().is_some_and(|failure| failure.ssid == ssid) {
        wifi.failed = None;
    }
}

#[derive(Debug, PartialEq)]
enum Step {
    Wait,
    Joined,
    Failed,
}

// NetworkManager starts working on it within a few seconds, and a WPA handshake is over well within the limit
const START_WITHIN: Duration = Duration::from_secs(10);
const GIVE_UP_AFTER: Duration = Duration::from_secs(45);

/*
 * where a join stands, from what `Network` shows: on the network and done joining is success, having been busy
 * joining and stopped without getting there is failure. the network that was up before (if any) is still
 * `current` until NetworkManager starts, so nothing counts before it's been seen busy, or the limit passed
 */
fn step(target: &str, current: &str, connecting: bool, seen_connecting: bool, elapsed: Duration) -> Step {
    if current == target && !connecting {
        return Step::Joined;
    }

    if seen_connecting && !connecting {
        return Step::Failed;
    }

    if elapsed > GIVE_UP_AFTER || (!seen_connecting && elapsed > START_WITHIN) {
        return Step::Failed;
    }

    Step::Wait
}

// follows a join to its end. a password typed in the panel that didn't work leaves a profile with that password
// behind, which would be tried again on the next click, so it's forgotten and the field opens again
fn watch(ssid: String, typed: bool) {
    let started = Instant::now();

    let mut seen_connecting = false;

    loop {
        std::thread::sleep(Duration::from_millis(400));

        let (current, connecting) = {
            let network = Network::read();

            (network.ssid().to_string(), network.connecting())
        };

        seen_connecting |= connecting;

        match step(&ssid, &current, connecting, seen_connecting, started.elapsed()) {
            Step::Wait => continue,
            Step::Joined => {
                Wifi::write().joining = None;

                return;
            }
            Step::Failed => {
                if typed {
                    Network::forget(&ssid);
                }

                let mut wifi = Wifi::write();

                wifi.joining = None;
                wifi.failed = Some(Failure { ssid: ssid.clone(), wrong_password: typed });

                if typed {
                    wifi.prompt = Some(ssid);
                    wifi.password.clear();

                    drop(wifi);

                    TextInput::set_text(PASSWORD_FIELD, "");
                }

                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Step, glyph, step};

    const SOON: Duration = Duration::from_secs(2);

    #[test]
    fn being_on_the_network_and_done_is_joined() {
        assert_eq!(step("home", "home", false, true, SOON), Step::Joined);

        // a saved network can come up between two looks
        assert_eq!(step("home", "home", false, false, SOON), Step::Joined);
    }

    #[test]
    fn the_old_network_is_not_a_failure_before_networkmanager_starts() {
        assert_eq!(step("cafe", "home", false, false, SOON), Step::Wait);
        assert_eq!(step("cafe", "home", true, true, SOON), Step::Wait);
    }

    #[test]
    fn stopping_without_getting_there_is_a_failure() {
        assert_eq!(step("cafe", "", false, true, SOON), Step::Failed);
        assert_eq!(step("cafe", "home", false, true, SOON), Step::Failed);
    }

    #[test]
    fn a_join_that_never_starts_or_never_ends_gives_up() {
        assert_eq!(step("cafe", "home", false, false, Duration::from_secs(11)), Step::Failed);
        assert_eq!(step("cafe", "", true, true, Duration::from_secs(46)), Step::Failed);
    }

    #[test]
    fn more_signal_draws_more_bars() {
        assert_eq!(glyph(5), "\u{f092f}");
        assert_eq!(glyph(30), "\u{f091f}");
        assert_eq!(glyph(60), "\u{f0922}");
        assert_eq!(glyph(80), "\u{f0925}");
        assert_eq!(glyph(100), "\u{f0928}");
    }
}
