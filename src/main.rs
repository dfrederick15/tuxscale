mod app;
mod tailscale;
mod tray;
mod ui;

use app::App;
use std::sync::OnceLock;
use tray_icon::menu::MenuId;

/// The tray menu item IDs, set once at startup. MenuId is Send+Sync.
pub static SHOW_ID: OnceLock<MenuId> = OnceLock::new();
pub static QUIT_ID: OnceLock<MenuId> = OnceLock::new();

fn main() -> iced::Result {
    // Vulkan is unavailable in this environment; fall back to OpenGL.
    if std::env::var("WGPU_BACKEND").is_err() {
        unsafe { std::env::set_var("WGPU_BACKEND", "gl") };
    }

    // Tray must live for the duration of the process.
    // TrayIcon is !Send so it stays here on the main thread.
    let t = tray::init();
    SHOW_ID.set(t.show_id.clone()).ok();
    QUIT_ID.set(t.quit_id.clone()).ok();
    let _tray = t._icon; // keep alive

    iced::application(App::new, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .title(|_: &App| "Tuxscale".to_string())
        .window(iced::window::Settings {
            size: iced::Size::new(720.0, 520.0),
            min_size: Some(iced::Size::new(500.0, 380.0)),
            exit_on_close_request: false, // hide to tray instead
            ..Default::default()
        })
        .run()
}
