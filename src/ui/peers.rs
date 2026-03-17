use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

use crate::app::{Message, TsState};
use crate::tailscale::PeerStatus;

pub fn view(state: &TsState) -> Element<'_, Message> {
    let peers = match &state.peers {
        Some(p) if !p.is_empty() => p,
        _ => {
            return container(
                text("No peers — connect to your tailnet first.")
                    .size(13)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
            )
            .padding(16)
            .into();
        }
    };

    let rows: Vec<Element<Message>> = peers
        .iter()
        .map(|peer| peer_row(peer, &state.ping_output, &state.pinging))
        .collect();

    let header = row![
        text("Hostname").size(12).width(180),
        text("IP").size(12).width(140),
        text("OS").size(12).width(80),
        text("Status").size(12).width(80),
        iced::widget::Space::new().width(Length::Fill),
    ]
    .spacing(8)
    .padding([4, 8]);

    let list = scrollable(
        column(rows).spacing(4).padding([0, 8]),
    )
    .height(Length::Fill);

    column![header, list].spacing(4).into()
}

fn peer_row<'a>(
    peer: &'a PeerStatus,
    ping_output: &'a std::collections::HashMap<String, String>,
    pinging: &'a Option<String>,
) -> Element<'a, Message> {
    let is_pinging = pinging.as_deref() == Some(peer.primary_ip());

    let online_color = if peer.online {
        iced::Color::from_rgb(0.2, 0.75, 0.3)
    } else {
        iced::Color::from_rgb(0.6, 0.6, 0.6)
    };

    let ping_label = if is_pinging {
        "Pinging…"
    } else {
        "Ping"
    };

    let ping_btn = button(text(ping_label).size(11))
        .on_press_maybe(if is_pinging {
            None
        } else {
            Some(Message::PingPeer(peer.primary_ip().to_string()))
        })
        .style(button::secondary);

    let copy_btn = button(text("Copy IP").size(11))
        .on_press(Message::CopyIp(peer.primary_ip().to_string()))
        .style(button::secondary);

    let ping_result: Element<Message> = if let Some(out) = ping_output.get(peer.primary_ip()) {
        text(out.trim()).size(10).color(iced::Color::from_rgb(0.4, 0.4, 0.4)).into()
    } else {
        iced::widget::Space::new().width(Length::Fill).into()
    };

    let main_row = row![
        text(peer.short_name()).size(13).width(180),
        text(peer.primary_ip()).size(13).width(140),
        text(&peer.os).size(13).width(80),
        text(if peer.online { "online" } else { "offline" })
            .size(12)
            .color(online_color)
            .width(80),
        iced::widget::Space::new().width(Length::Fill),
        ping_btn,
        copy_btn,
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    column![main_row, ping_result]
        .spacing(2)
        .padding([6, 8])
        .into()
}
