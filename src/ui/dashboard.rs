use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::{Message, TsState};

pub fn view(state: &TsState) -> Element<'_, Message> {
    let (status_label, status_color) = match state.backend_state.as_str() {
        "Running" => ("Connected", iced::Color::from_rgb(0.2, 0.75, 0.3)),
        "Stopped" => ("Disconnected", iced::Color::from_rgb(0.7, 0.2, 0.2)),
        s => (s, iced::Color::from_rgb(0.6, 0.6, 0.1)),
    };

    let status_badge = container(
        text(status_label)
            .size(14)
            .color(iced::Color::WHITE),
    )
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(status_color)),
        border: iced::Border {
            radius: 6.0.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .padding([4, 12]);

    let connect_btn = if state.backend_state == "Running" {
        button(text("Disconnect").size(14))
            .on_press(Message::Disconnect)
            .style(button::danger)
    } else {
        button(text("Connect").size(14))
            .on_press(Message::Connect)
            .style(button::success)
    };

    let ip_display = if let Some(node) = &state.self_node {
        let ip = node.primary_ip();
        column![
            text("Your Tailscale IP").size(12).color(iced::Color::from_rgb(0.6, 0.6, 0.6)),
            text(ip).size(22),
        ]
        .spacing(2)
    } else {
        column![text("—").size(22)]
    };

    let header = row![
        ip_display,
        iced::widget::Space::new().width(Length::Fill),
        status_badge,
        connect_btn,
    ]
    .spacing(12)
    .align_y(Alignment::Center);

    let refresh_btn = button(text("Refresh").size(12))
        .on_press(Message::Refresh)
        .style(button::secondary);

    let error_row: Element<Message> = if let Some(err) = &state.last_error {
        text(err).size(12).color(iced::Color::from_rgb(0.8, 0.2, 0.2)).into()
    } else {
        iced::widget::Space::new().into()
    };

    container(
        column![header, error_row, refresh_btn]
            .spacing(10)
            .padding(16),
    )
    .style(|theme| {
        let palette = iced::Theme::palette(theme);
        container::Style {
            background: Some(iced::Background::Color(palette.background)),
            border: iced::Border {
                color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                width: 1.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    })
    .width(Length::Fill)
    .into()
}
