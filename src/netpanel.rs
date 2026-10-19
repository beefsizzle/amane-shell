use amane::{Center, Color, Column, Horizontal, LayerWindow, Link, Monitor, Network, Parent, Rectangle, Row, Service, Start, Widget, children};

use crate::netinfo::Netinfo;
use crate::panels::{self, Which};
use crate::sys::{Sys, format_rate};
use crate::theme::{Theme, alpha, theme};
use crate::ui::{bold, icon, label, section};

const WIDTH: f32 = 372.0;
const INNER: f32 = WIDTH - 28.0;

fn bytes(count: u64) -> String {
    let value = count as f64;

    if value >= 1e12 {
        format!("{:.2} TB", value / 1e12)
    } else if value >= 1e9 {
        format!("{:.1} GB", value / 1e9)
    } else {
        format!("{:.0} MB", value / 1e6)
    }
}

// the last minute and a half of traffic as thin bars. the scale is at least 100 KB/s, so an idle line stays flat
fn sparkline(t: &Theme, width: f32, height: f32, data: &[f32], color: Color) -> Row {
    let top = data.iter().copied().fold(100_000.0, f32::max);

    let count = data.len().max(2);
    let bar = 3.0;
    let gap = ((width - bar * count as f32) / (count - 1) as f32).max(0.0);

    let mut bars: Vec<Box<dyn Widget>> = Vec::new();

    for (index, value) in data.iter().enumerate() {
        let newest = index == data.len() - 1;

        let fill = Rectangle::new()
            .width(bar)
            .height((height * value / top).max(1.0))
            .fill(if newest { color } else { alpha(color, 0.55) });

        bars.push(Box::new(Rectangle::new().width(bar).height(height).fill(alpha(t.fg, 0.04)).align_child(Center, amane::End).child(fill)));
    }

    Row::new(bars).gap(gap)
}

fn speed_card(t: &Theme, caption: &str, glyph: &str, rate: f64, history: &[f32]) -> Rectangle {
    Rectangle::new()
        .width((INNER - 10.0) / 2.0)
        .height(88.0)
        .fill(t.surface)
        .border(1.0, t.border)
        .padding(12.0)
        .align_child(Start, Start)
        .child(
            Column::new(children![
                Row::new(children![icon(glyph, 11.0, t.accent), section(caption, t)]).gap(7.0).align(Center),
                bold(format_rate(rate), 18.0, t.fg),
                sparkline(t, (INNER - 10.0) / 2.0 - 24.0, 26.0, history, t.accent),
            ])
            .gap(5.0),
        )
}

const DETAIL_ROW: f32 = 18.0;

fn detail(t: &Theme, name: &str, value: String) -> Box<dyn Widget> {
    Box::new(
        Rectangle::new().width(INNER - 24.0).height(DETAIL_ROW).align_child(Start, Center).child(
            Row::new(children![
                label(name, 12.0, t.dim),
                Rectangle::new().width(Parent).height(1.0),
                label(value, 12.0, t.fg),
            ])
            .align(Center)
            .width(INNER - 24.0),
        ),
    )
}

pub fn view(monitor: &Monitor) -> LayerWindow {
    if !panels::shown(Which::Network, monitor) {
        return panels::hidden("amane-panel");
    }

    let t = theme();
    let network = Network::read();
    let info = Netinfo::read();
    let sys = Sys::read();

    let (glyph, title) = match network.link() {
        Link::Wired => ("\u{f0200}", String::from("Wired connection")),
        Link::Wifi => ("\u{f1eb}", format!("Wi-Fi \u{b7} {}", network.ssid())),
        Link::Offline => ("\u{f05aa}", String::from("Offline")),
        _ => ("\u{f0200}", String::from("Connected")),
    };

    let subtitle = if info.interface.is_empty() { String::from("No active interface") } else { format!("{} \u{b7} {}", info.connection, info.interface) };

    let header = Row::new(children![
        Rectangle::new()
            .width(40.0)
            .height(40.0)
            .fill(t.accent_soft)
            .align_child(Center, Center)
            .child(icon(glyph, 18.0, if network.connected() { t.accent } else { t.faint })),
        Column::new(children![bold(title, 14.0, t.fg), label(subtitle, 11.5, t.dim)]).gap(2.0),
    ])
    .gap(12.0)
    .align(Center);

    let speeds = Row::new(children![
        speed_card(&t, "Download", "\u{f063}", sys.rx, &sys.rx_history),
        speed_card(&t, "Upload", "\u{f062}", sys.tx, &sys.tx_history),
    ])
    .gap(10.0);

    let mut details: Vec<Box<dyn Widget>> = Vec::new();

    if !info.ip.is_empty() {
        details.push(detail(&t, "IP address", info.ip.clone()));
    }

    if !info.ip6.is_empty() {
        details.push(detail(&t, "IPv6", info.ip6.clone()));
    }

    if !info.gateway.is_empty() {
        details.push(detail(&t, "Gateway", info.gateway.clone()));
    }

    if !info.dns.is_empty() {
        details.push(detail(&t, "DNS", info.dns.iter().take(2).cloned().collect::<Vec<_>>().join(", ")));
    }

    if !info.mac.is_empty() {
        details.push(detail(&t, "MAC address", info.mac.to_lowercase()));
    }

    if network.link() == Link::Wifi {
        details.push(detail(&t, "Signal", format!("{}%", network.strength())));
    } else if !info.speed.is_empty() {
        details.push(detail(&t, "Link speed", info.speed.clone()));
    }

    if let Some(ping) = info.ping {
        details.push(detail(&t, "Latency to gateway", format!("{ping:.1} ms")));
    }

    details.push(detail(&t, "Received", bytes(sys.rx_total)));
    details.push(detail(&t, "Sent", bytes(sys.tx_total)));

    let detail_height = details.len() as f32 * DETAIL_ROW + (details.len() as f32 - 1.0) * 6.0 + 24.0;

    let card = Rectangle::new()
        .width(INNER)
        .height(detail_height)
        .fill(t.surface)
        .border(1.0, t.border)
        .padding(12.0)
        .child(Column::new(details).gap(6.0));

    let mut sections: Vec<Box<dyn Widget>> = vec![Box::new(header), Box::new(speeds), Box::new(card)];

    let mut height = 28.0 + 40.0 + 14.0 + 88.0 + 14.0 + detail_height;

    // wi-fi networks in range, for machines that have a radio
    if network.wifi_enabled() && !network.access_points().is_empty() {
        let mut points: Vec<&amane::AccessPoint> = network.access_points().iter().collect();

        points.sort_by(|a, b| b.active().cmp(&a.active()).then(b.strength().cmp(&a.strength())));

        let mut rows: Vec<Box<dyn Widget>> = Vec::new();

        for point in points.iter().take(5) {
            let ssid = point.ssid().to_string();
            let active = point.active();
            let open = !point.secured() || point.saved();

            rows.push(Box::new(
                Rectangle::new()
                    .width(INNER)
                    .height(30.0)
                    .padding(amane::Padding { top: 0.0, right: 10.0, bottom: 0.0, left: 10.0 })
                    .align_child(Start, Center)
                    .cursor(amane::Pointer)
                    .on_click(move |_| {
                        if active {
                            Network::disconnect();
                        } else if open {
                            Network::connect(&ssid, None);
                        }
                    })
                    .child(
                        Row::new(children![
                            icon("\u{f1eb}", 12.0, if active { t.accent } else { t.dim }),
                            label(point.ssid(), 12.5, if active { t.fg } else { t.dim }),
                            Rectangle::new().width(Parent).height(1.0),
                            icon(if point.secured() { "\u{f023}" } else { "" }, 10.0, t.faint),
                            label(format!("{}%", point.strength()), 11.0, t.faint),
                        ])
                        .gap(10.0)
                        .align(Center)
                        .width(INNER - 20.0),
                    ),
            ));
        }

        let count = rows.len() as f32;

        sections.push(Box::new(section("Wi-Fi networks", &t)));
        sections.push(Box::new(Column::new(rows).gap(2.0)));

        height += 14.0 + 14.0 + 14.0 + count * 30.0 + (count - 1.0) * 2.0;
    }

    panels::window_at(Which::Network, monitor, WIDTH, height, Horizontal::Right, crate::bar::right_edge(crate::bar::Slot::Network))
        .child(panels::body(&t).padding(14.0).child(Column::new(sections).gap(14.0)))
}
