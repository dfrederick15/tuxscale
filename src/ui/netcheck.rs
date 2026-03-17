use iced::widget::{button, column, container, row, text};
use iced::Element;

use crate::app::{Message, TsState};

pub fn view(state: &TsState) -> Element<'_, Message> {
    let run_btn = button(text(if state.netcheck_running { "Running…" } else { "Run Netcheck" }).size(13))
        .on_press_maybe(if state.netcheck_running {
            None
        } else {
            Some(Message::RunNetcheck)
        })
        .style(button::secondary);

    let report: Element<Message> = match &state.netcheck_report {
        None => text("Press Run Netcheck to analyse your network.")
            .size(13)
            .color(iced::Color::from_rgb(0.5, 0.5, 0.5))
            .into(),
        Some(r) => {
            let mut latencies: Vec<(String, u64)> = r
                .region_latency
                .as_ref()
                .map(|m| m.iter().map(|(k, v)| (k.clone(), *v)).collect())
                .unwrap_or_default();
            latencies.sort_by_key(|a| a.1);

            let lat_rows: Vec<Element<Message>> = latencies
                .iter()
                .take(8)
                .map(|(region, ns)| {
                    let ms = *ns as f64 / 1_000_000.0;
                    row![
                        text(format!("Region {region}")).size(12).width(120),
                        text(format!("{ms:.1} ms")).size(12),
                    ]
                    .spacing(8)
                    .into()
                })
                .collect();

            column![
                row![
                    stat("UDP", bool_str(r.udp).to_string()),
                    stat("IPv4", bool_str(r.ipv4).to_string()),
                    stat("IPv6", bool_str(r.ipv6).to_string()),
                    stat("Preferred DERP", r.preferred_derp.to_string()),
                ]
                .spacing(16),
                text("Region latencies (closest first):").size(12),
                column(lat_rows).spacing(2),
            ]
            .spacing(10)
            .into()
        }
    };

    container(
        column![run_btn, report].spacing(12).padding(16),
    )
    .into()
}

fn bool_str(b: bool) -> &'static str {
    if b { "✓" } else { "✗" }
}

fn stat(label: &str, value: String) -> Element<'_, Message> {
    column![
        text(label.to_string()).size(11).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
        text(value).size(14),
    ]
    .spacing(2)
    .into()
}
