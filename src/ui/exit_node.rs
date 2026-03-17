use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

use crate::app::{Message, TsState};

pub fn view(state: &TsState) -> Element<'_, Message> {
    let nodes = match &state.exit_nodes {
        Some(n) => n.clone(),
        None => {
            return container(
                column![
                    text("Exit nodes not loaded.").size(13),
                    button(text("Load exit nodes").size(13))
                        .on_press(Message::LoadExitNodes)
                        .style(button::secondary),
                ]
                .spacing(8),
            )
            .padding(16)
            .into();
        }
    };

    if nodes.is_empty() {
        return container(
            text("No exit nodes available on your tailnet.")
                .size(13)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.5)),
        )
        .padding(16)
        .into();
    }

    let clear_btn = button(text("Clear exit node").size(12))
        .on_press(Message::ClearExitNode)
        .style(button::danger);

    let node_rows: Vec<Element<Message>> = nodes
        .iter()
        .map(|node| {
            let label = if node.active {
                format!("★ {} ({})", node.name, node.ip)
            } else {
                format!("  {} ({})", node.name, node.ip)
            };

            let use_btn = button(text(if node.active { "Active" } else { "Use" }).size(12))
                .on_press_maybe(if node.active {
                    None
                } else {
                    Some(Message::SetExitNode(node.name.clone()))
                })
                .style(if node.active {
                    button::success
                } else {
                    button::secondary
                });

            row![
                text(label).size(13).width(Length::Fill),
                use_btn,
            ]
            .spacing(8)
            .padding([6, 8])
            .into()
        })
        .collect();

    column![
        column(node_rows).spacing(2),
        clear_btn,
    ]
    .spacing(12)
    .padding(8)
    .into()
}
