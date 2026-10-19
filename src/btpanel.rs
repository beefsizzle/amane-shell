use amane::{Center, Column, Horizontal, LayerWindow, Monitor, Parent, Rectangle, Row, ScrollArea, Service, Start, Widget, children};

use crate::btstate::{self, BtState, Device};

use crate::panels::{self, Which};
use crate::theme::{Theme, alpha, mix, theme};
use crate::ui::{bold, chip, hover, icon, label, round_button, set_hover};

/*
 * the Bluetooth dropdown: switch the adapter, see what's around while it's open (scanning only runs then, see
 * `panels::changed`), and pair, connect, disconnect or forget a device
 */
const NOTE_HEIGHT: f32 = 90.0;
const WIDTH: f32 = 372.0;
const INNER: f32 = WIDTH - 28.0;
const ROW: f32 = 46.0;
const MAX_ROWS: usize = 7;

// BlueZ's icon names are freedesktop names; these are Nerd Font glyphs
fn glyph(name: &str) -> &'static str {
    match name {
        "audio-headphones" | "audio-headset" => "\u{f025}",
        "audio-card" | "audio-speakers" => "\u{f028}",
        "input-keyboard" => "\u{f11c}",
        "input-mouse" => "\u{f037d}",
        "input-gaming" => "\u{f11b}",
        "phone" => "\u{f10b}",
        "computer" => "\u{f109}",
        _ => "\u{f293}",
    }
}

// BlueZ names a device after its address when it never told it a name
fn named(device: &Device) -> bool {
    !device.name.is_empty() && device.name.replace('-', ":") != device.address
}

fn row(t: &Theme, index: usize, device: &Device) -> Rectangle {
    let id = format!("bt-{index}");
    let hot = hover(&id);

    let path = device.path.clone();
    let (connected, paired) = (device.connected, device.paired);

    let status = if connected {
        match device.battery {
            Some(percent) => format!("Connected \u{b7} {percent}%"),
            None => String::from("Connected"),
        }
    } else if paired {
        String::from("Paired")
    } else {
        String::from("Available to pair")
    };

    let action = if connected { "Disconnect" } else if paired { "Connect" } else { "Pair" };

    let target = path.clone();

    let forget: Box<dyn Widget> = if paired {
        Box::new(round_button(&format!("bt-x-{index}"), t, "\u{f1f8}", t.faint, move |_| btstate::forget(&path)))
    } else {
        Box::new(Rectangle::new().width(1.0).height(1.0))
    };

    Rectangle::new()
        .width(INNER)
        .height(ROW)
        .fill(if connected { alpha(t.accent, 0.14) } else { mix(alpha(t.surface, 0.0), t.surface_hi, hot) })
        .padding(amane::Padding { top: 0.0, right: 8.0, bottom: 0.0, left: 12.0 })
        .align_child(Start, Center)
        .cursor(amane::Pointer)
        .on_hover(move |inside| set_hover(&id, inside))
        .on_click(move |_| {
            if connected {
                btstate::disconnect(&target);
            } else if paired {
                btstate::connect(&target);
            } else {
                btstate::pair(&target);
            }
        })
        .child(
            Row::new(vec![
                Box::new(Rectangle::new().width(22.0).height(22.0).align_child(Center, Center).child(icon(glyph(&device.icon), 15.0, if connected { t.accent } else { t.dim }))) as Box<dyn Widget>,
                Box::new(
                    Column::new(children![
                        Rectangle::new().width(INNER - 196.0).height(17.0).child(bold(if device.name.is_empty() { &device.address } else { &device.name }, 12.5, t.fg).elide()),
                        label(status, 11.0, if connected { t.accent } else { t.dim }),
                    ])
                    .gap(1.0),
                ),
                Box::new(Rectangle::new().width(Parent).height(1.0)),
                Box::new(label(action, 11.0, t.faint)),
                forget,
            ])
            .gap(10.0)
            .align(Center)
            .width(INNER - 20.0),
        )
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    if !panels::shown(Which::Bluetooth, monitor) {
        return panels::hidden("amane-panel");
    }

    let t = theme();
    let bluetooth = BtState::read();

    let header = Row::new(children![
        icon("\u{f293}", 16.0, t.accent),
        bold("Bluetooth", 15.0, t.fg),
        Rectangle::new().width(Parent).height(1.0),
        label(if bluetooth.scanning { "Searching\u{2026}" } else { "" }, 11.0, t.faint),
        chip(
            "bt-power",
            &t,
            52.0,
            label(if bluetooth.powered { "On" } else { "Off" }, 11.5, if bluetooth.powered { t.accent } else { t.dim }),
        )
        .on_click(|_| {
            let on = BtState::read().powered;

            btstate::set_powered(!on);
        }),
    ])
    .gap(10.0)
    .align(Center)
    .width(INNER);

    let devices: Vec<&Device> = bluetooth.devices.iter().filter(|device| device.paired || named(device)).collect();

    // the list, or a line saying why there isn't one
    let note = |text: &str| -> (Box<dyn Widget>, f32) {
        let body = Rectangle::new().width(INNER).height(NOTE_HEIGHT).align_child(Center, Center).child(label(text, 12.5, t.faint));

        (Box::new(body), NOTE_HEIGHT)
    };

    let (body, list_height) = if bluetooth.adapter.is_none() {
        note("No Bluetooth adapter found")
    } else if !bluetooth.powered {
        note("Bluetooth is off")
    } else if devices.is_empty() {
        note("Looking for devices\u{2026}")
    } else {
        let rows: Vec<Box<dyn Widget>> = devices.iter().enumerate().map(|(index, device)| Box::new(row(&t, index, device)) as Box<dyn Widget>).collect();

        let shown = rows.len().min(MAX_ROWS) as f32;

        let list_height = shown * ROW + (shown - 1.0) * 2.0;

        let list = Rectangle::new().width(INNER).height(list_height).clip().child(ScrollArea::new("bluetooth-list", Column::new(rows).gap(2.0)));

        (Box::new(list) as Box<dyn Widget>, list_height)
    };

    let height = 28.0 + 30.0 + 14.0 + list_height;

    panels::window_at(Which::Bluetooth, monitor, WIDTH, height, Horizontal::Right, crate::bar::right_edge(crate::bar::Slot::Bluetooth))
        .child(panels::body(&t).padding(14.0).child(Column::new(vec![Box::new(header) as Box<dyn Widget>, body]).gap(14.0)))
}
