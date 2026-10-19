use amane::{Center, Column, Horizontal, Key, Keyboard, LayerWindow, Lock, Monitor, Parent, Rectangle, Row, Start, Widget};

use crate::panels::{self, Which};
use crate::theme::{Theme, alpha, mix, theme};
use crate::ui::{hover, icon, label, set_hover};

/*
 * the power dropdown under the power button. every entry has a number key, the same one every time,
 * so 3 is always sleep. bound to Alt+X in niri.
 */
const WIDTH: f32 = 250.0;

// the commands behind the entries, shared with the dashboard's buttons and the `sleep` ipc call
pub const SUSPEND: &str = "systemctl suspend";
pub const LOG_OUT: &str = "niri msg action quit --skip-confirmation";
pub const REBOOT: &str = "systemctl reboot";
pub const POWER_OFF: &str = "systemctl poweroff";
const ROW: f32 = 40.0;

struct Entry {
    key: char,
    glyph: &'static str,
    name: &'static str,
    danger: bool,
    run: fn(),
}

const ENTRIES: [Entry; 5] = [
    Entry { key: '1', glyph: "\u{f023}", name: "Lock", danger: false, run: lock },
    Entry { key: '2', glyph: "\u{f08b}", name: "Log out", danger: false, run: log_out },
    Entry { key: '3', glyph: "\u{f186}", name: "Sleep", danger: false, run: sleep },
    Entry { key: '4', glyph: "\u{f021}", name: "Restart", danger: true, run: restart },
    Entry { key: '5', glyph: "\u{f011}", name: "Shut down", danger: true, run: shut_down },
];

fn lock() {
    panels::close();

    Lock::start();
}

fn log_out() {
    panels::close();

    amane::spawn(LOG_OUT);
}

fn sleep() {
    panels::close();

    amane::spawn(SUSPEND);
}

fn restart() {
    panels::close();

    amane::spawn(REBOOT);
}

fn shut_down() {
    panels::close();

    amane::spawn(POWER_OFF);
}

fn on_key(key: Key) {
    match key {
        Key::Escape => panels::close(),
        Key::Character(letter) => {
            if let Some(entry) = ENTRIES.iter().find(|entry| entry.key == letter) {
                (entry.run)();
            }
        }
        _ => {}
    }
}

fn row(t: &Theme, entry: &Entry) -> Rectangle {
    let id = format!("power-{}", entry.key);
    let hot = hover(&id);

    let tint = if entry.danger { t.red } else { t.fg };

    let run = entry.run;

    Rectangle::new()
        .width(WIDTH - 28.0)
        .height(ROW)
        .fill(mix(alpha(t.surface, 0.0), t.surface_hi, hot))
        .padding(amane::Padding { top: 0.0, right: 12.0, bottom: 0.0, left: 12.0 })
        .align_child(Start, Center)
        .cursor(amane::Pointer)
        .on_hover(move |inside| set_hover(&id, inside))
        .on_click(move |_| run())
        .child(
            Row::new(vec![
                Box::new(Rectangle::new().width(16.0).height(16.0).align_child(Center, Center).child(icon(entry.glyph, 14.0, tint))) as Box<dyn Widget>,
                Box::new(label(entry.name, 13.5, tint)),
                Box::new(Rectangle::new().width(Parent).height(1.0)),
                Box::new(
                    Rectangle::new()
                        .width(22.0)
                        .height(22.0)
                        .border(1.0, alpha(t.fg, 0.28))
                        .align_child(Center, Center)
                        .child(label(entry.key.to_string(), 11.5, t.dim)),
                ),
            ])
            .gap(12.0)
            .align(Center)
            .width(WIDTH - 28.0 - 24.0),
        )
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    if !panels::shown(Which::Power, monitor) {
        return panels::hidden("amane-panel");
    }

    let t = theme();

    let rows: Vec<Box<dyn Widget>> = ENTRIES.iter().map(|entry| Box::new(row(&t, entry)) as Box<dyn Widget>).collect();

    let height = 28.0 + ROW * ENTRIES.len() as f32 + 2.0 * (ENTRIES.len() as f32 - 1.0) + 8.0 + 16.0;

    panels::window_at(Which::Power, monitor, WIDTH, height, Horizontal::Right, crate::bar::right_edge(crate::bar::Slot::Power))
        .keyboard(Keyboard::Exclusive)
        .on_key(on_key)
        .child(
            panels::body(&t).padding(14.0).child(
                Column::new(vec![
                    Box::new(Column::new(rows).gap(2.0)) as Box<dyn Widget>,
                    Box::new(label("press a number, esc to cancel", 10.5, t.faint)),
                ])
                .gap(8.0)
                .align(Center),
            ),
        )
}
