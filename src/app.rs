use std::collections::HashMap;

use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Subscription, Task, Theme, time, window};

use crate::tailscale::{self, ExitNode, NetcheckReport, PeerStatus, Prefs, Status};
use crate::tray;
use crate::ui;

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone)]
pub struct TsState {
    pub backend_state: String,
    pub self_node: Option<PeerStatus>,
    pub peers: Option<Vec<PeerStatus>>,
    pub exit_nodes: Option<Vec<ExitNode>>,
    pub ping_output: HashMap<String, String>,
    pub pinging: Option<String>,
    pub netcheck_report: Option<NetcheckReport>,
    pub netcheck_running: bool,
    pub prefs: Option<Prefs>,
    pub last_error: Option<String>,
}

// ── App ───────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub enum Tab {
    #[default]
    Peers,
    ExitNodes,
    Netcheck,
    Options,
}

pub struct App {
    state: TsState,
    tab: Tab,
    window_visible: bool,
    window_id: Option<window::Id>,
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    // lifecycle
    Refresh,
    StatusLoaded(Result<Status, String>),
    // connection
    Connect,
    Disconnect,
    ConnectDone(Result<(), String>),
    // peers
    PingPeer(String),
    PingDone(String, Result<String, String>),
    CopyIp(String),
    // exit nodes
    LoadExitNodes,
    ExitNodesLoaded(Result<Vec<ExitNode>, String>),
    SetExitNode(String),
    ClearExitNode,
    ExitNodeSet(Result<(), String>),
    // netcheck
    RunNetcheck,
    NetcheckDone(Result<NetcheckReport, String>),
    // options
    LoadPrefs,
    PrefsLoaded(Result<Prefs, String>),
    SetOption(String, bool),
    OptionSet(Result<(), String>),
    // tabs
    SetTab(Tab),
    // window
    GotWindowId(Option<window::Id>),
    WindowCloseRequest(window::Id),
    // tray
    TrayEvent,
    // timer
    Tick,
}

// ── iced::Application impl ────────────────────────────────────────────────────

impl App {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: TsState::default(),
                tab: Tab::default(),
                window_visible: true,
                window_id: None,
            },
            Task::batch([
                Task::perform(tailscale::status(), Message::StatusLoaded),
                // Grab the main window ID once the event loop starts.
                window::latest().map(|id| Message::GotWindowId(id)),
            ]),
        )
    }

    pub fn title(&self) -> String {
        "Tailscale".to_string()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick | Message::Refresh => {
                Task::perform(tailscale::status(), Message::StatusLoaded)
            }

            Message::StatusLoaded(Ok(status)) => {
                self.state.backend_state = status.backend_state.clone();
                self.state.self_node = status.self_node;
                self.state.peers = status.peer.map(|map| {
                    let mut peers: Vec<PeerStatus> = map.into_values().collect();
                    peers.sort_by(|a, b| {
                        b.online.cmp(&a.online).then(a.host_name.cmp(&b.host_name))
                    });
                    peers
                });
                self.state.last_error = None;
                Task::none()
            }

            Message::StatusLoaded(Err(e)) => {
                self.state.last_error = Some(e);
                Task::none()
            }

            Message::Connect => {
                Task::perform(tailscale::connect(), Message::ConnectDone)
            }

            Message::Disconnect => {
                Task::perform(tailscale::disconnect(), Message::ConnectDone)
            }

            Message::ConnectDone(result) => {
                if let Err(e) = result {
                    self.state.last_error = Some(e);
                }
                Task::perform(tailscale::status(), Message::StatusLoaded)
            }

            Message::PingPeer(ip) => {
                self.state.pinging = Some(ip.clone());
                let ip_clone = ip.clone();
                Task::perform(
                    async move { tailscale::ping(&ip_clone).await },
                    move |r| Message::PingDone(ip.clone(), r),
                )
            }

            Message::PingDone(ip, result) => {
                self.state.pinging = None;
                match result {
                    Ok(out) => {
                        let summary = out
                            .lines()
                            .filter(|l| l.contains("ms") || l.contains("pong"))
                            .take(3)
                            .collect::<Vec<_>>()
                            .join(" | ");
                        self.state.ping_output.insert(ip, summary);
                    }
                    Err(e) => {
                        self.state.ping_output.insert(ip, format!("Error: {e}"));
                    }
                }
                Task::none()
            }

            Message::CopyIp(ip) => {
                // iced 0.14 clipboard write
                iced::clipboard::write(ip)
            }

            Message::LoadExitNodes => {
                Task::perform(tailscale::exit_nodes(), Message::ExitNodesLoaded)
            }

            Message::ExitNodesLoaded(result) => {
                match result {
                    Ok(nodes) => self.state.exit_nodes = Some(nodes),
                    Err(e) => self.state.last_error = Some(e),
                }
                Task::none()
            }

            Message::SetExitNode(name) => {
                Task::perform(
                    async move { tailscale::set_exit_node(&name).await },
                    Message::ExitNodeSet,
                )
            }

            Message::ClearExitNode => {
                Task::perform(tailscale::clear_exit_node(), Message::ExitNodeSet)
            }

            Message::ExitNodeSet(result) => {
                if let Err(e) = result {
                    self.state.last_error = Some(e);
                }
                Task::perform(tailscale::exit_nodes(), Message::ExitNodesLoaded)
            }

            Message::RunNetcheck => {
                self.state.netcheck_running = true;
                Task::perform(tailscale::netcheck(), Message::NetcheckDone)
            }

            Message::NetcheckDone(result) => {
                self.state.netcheck_running = false;
                match result {
                    Ok(report) => self.state.netcheck_report = Some(report),
                    Err(e) => self.state.last_error = Some(e),
                }
                Task::none()
            }

            Message::LoadPrefs => {
                Task::perform(tailscale::prefs(), Message::PrefsLoaded)
            }

            Message::PrefsLoaded(result) => {
                match result {
                    Ok(p) => self.state.prefs = Some(p),
                    Err(e) => self.state.last_error = Some(e),
                }
                Task::none()
            }

            Message::SetOption(flag, value) => {
                Task::perform(
                    async move { tailscale::set_bool(&flag, value).await },
                    Message::OptionSet,
                )
            }

            Message::OptionSet(result) => {
                if let Err(e) = result {
                    self.state.last_error = Some(e);
                }
                Task::perform(tailscale::prefs(), Message::PrefsLoaded)
            }

            Message::SetTab(tab) => {
                let load = matches!(tab, Tab::Options) && self.state.prefs.is_none();
                self.tab = tab;
                if load {
                    Task::perform(tailscale::prefs(), Message::PrefsLoaded)
                } else {
                    Task::none()
                }
            }

            Message::GotWindowId(id) => {
                self.window_id = id;
                Task::none()
            }

            Message::WindowCloseRequest(id) => {
                // Hide to tray instead of closing
                self.window_visible = false;
                window::set_mode(id, window::Mode::Hidden)
            }

            Message::TrayEvent => {
                while let Some(id) = tray::try_recv() {
                    if Some(&id) == crate::QUIT_ID.get() {
                        return iced::exit();
                    }
                    if Some(&id) == crate::SHOW_ID.get() {
                        if let Some(win_id) = self.window_id {
                            let mode = if self.window_visible {
                                self.window_visible = false;
                                window::Mode::Hidden
                            } else {
                                self.window_visible = true;
                                window::Mode::Windowed
                            };
                            return window::set_mode(win_id, mode);
                        }
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let tab_bar = row![
            tab_btn("Peers", matches!(self.tab, Tab::Peers), Message::SetTab(Tab::Peers)),
            tab_btn("Exit Nodes", matches!(self.tab, Tab::ExitNodes), Message::SetTab(Tab::ExitNodes)),
            tab_btn("Netcheck", matches!(self.tab, Tab::Netcheck), Message::SetTab(Tab::Netcheck)),
            tab_btn("Options", matches!(self.tab, Tab::Options), Message::SetTab(Tab::Options)),
        ]
        .spacing(4);

        let content: Element<Message> = match self.tab {
            Tab::Peers => ui::peers::view(&self.state),
            Tab::ExitNodes => ui::exit_node::view(&self.state),
            Tab::Netcheck => ui::netcheck::view(&self.state),
            Tab::Options => ui::options::view(&self.state),
        };

        container(
            column![
                ui::dashboard::view(&self.state),
                tab_bar,
                container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(|theme| {
                        let palette = iced::Theme::palette(theme);
                        iced::widget::container::Style {
                            background: Some(iced::Background::Color(palette.background)),
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.85, 0.85, 0.85),
                                width: 1.0,
                                radius: 8.0.into(),
                            },
                            ..Default::default()
                        }
                    }),
            ]
            .spacing(8)
            .padding(12),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            time::every(std::time::Duration::from_secs(5)).map(|_| Message::Tick),
            // Poll tray menu events at ~10 fps
            time::every(std::time::Duration::from_millis(100)).map(|_| Message::TrayEvent),
            // Intercept close button → hide to tray
            window::close_requests().map(Message::WindowCloseRequest),
        ])
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }
}

fn tab_btn(label: &str, active: bool, msg: Message) -> Element<'_, Message> {
    let btn = button(text(label).size(13)).on_press(msg);
    if active {
        btn.style(button::primary)
    } else {
        btn.style(button::secondary)
    }
    .into()
}
