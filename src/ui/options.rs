use iced::widget::{button, column, container, row, text, toggler};
use iced::{Element, Length};

use crate::app::{Message, TsState};

pub fn view(state: &TsState) -> Element<'_, Message> {
    let prefs = match &state.prefs {
        None => {
            return container(
                column![
                    text("Options not loaded.").size(13),
                    button(text("Load options").size(13))
                        .on_press(Message::LoadPrefs)
                        .style(iced::widget::button::secondary),
                ]
                .spacing(8),
            )
            .padding(16)
            .into();
        }
        Some(p) => p,
    };

    let reload_btn = button(text("Reload").size(12))
        .on_press(Message::LoadPrefs)
        .style(iced::widget::button::secondary);

    container(
        column![
            toggle_row(
                "Accept DNS",
                "Use DNS settings pushed by the Tailscale admin",
                prefs.accept_dns,
                Message::SetOption("accept-dns".into(), !prefs.accept_dns),
            ),
            toggle_row(
                "Accept Routes",
                "Accept subnet routes advertised by other nodes",
                prefs.accept_routes,
                Message::SetOption("accept-routes".into(), !prefs.accept_routes),
            ),
            toggle_row(
                "Shields Up",
                "Block all incoming connections",
                prefs.shields_up,
                Message::SetOption("shields-up".into(), !prefs.shields_up),
            ),
            toggle_row(
                "SSH Server",
                "Allow Tailscale SSH access to this machine",
                prefs.run_ssh,
                Message::SetOption("ssh".into(), !prefs.run_ssh),
            ),
            toggle_row(
                "LAN Access via Exit Node",
                "Allow local network access when using an exit node",
                prefs.exit_node_allow_lan_access,
                Message::SetOption(
                    "exit-node-allow-lan-access".into(),
                    !prefs.exit_node_allow_lan_access,
                ),
            ),
            toggle_row(
                "Advertise as Exit Node",
                "Offer this machine as an exit node for the tailnet",
                prefs.advertise_exit_node,
                Message::SetOption(
                    "advertise-exit-node".into(),
                    !prefs.advertise_exit_node,
                ),
            ),
            row![
                iced::widget::Space::new().width(Length::Fill),
                reload_btn,
            ],
        ]
        .spacing(4)
        .padding(16),
    )
    .into()
}

fn toggle_row<'a>(
    label: &'a str,
    description: &'a str,
    value: bool,
    on_toggle: Message,
) -> Element<'a, Message> {
    row![
        column![
            text(label).size(13),
            text(description)
                .size(11)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        ]
        .spacing(2)
        .width(Length::Fill),
        toggler(value).on_toggle(move |_| on_toggle.clone()),
    ]
    .spacing(12)
    .padding([8, 0])
    .into()
}
