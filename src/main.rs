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
    if std::env::args().any(|a| a == "--install") {
        return install();
    }

    // Vulkan is unavailable in this environment; fall back to OpenGL.
    if std::env::var("WGPU_BACKEND").is_err() {
        unsafe { std::env::set_var("WGPU_BACKEND", "gl") };
    }

    // GTK must be initialised before the tray icon is created.
    gtk::init().expect("failed to initialize GTK");

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

fn install() -> iced::Result {
    use std::fs;
    use std::path::PathBuf;

    let current_exe = std::env::current_exe().expect("cannot determine current executable path");

    let home = std::env::var("HOME").expect("HOME not set");
    let local_bin = PathBuf::from(&home).join(".local/bin");
    let desktop_dir = PathBuf::from(&home).join(".local/share/applications");
    let icon_dir = PathBuf::from(&home).join(".local/share/icons");

    fs::create_dir_all(&local_bin).ok();
    fs::create_dir_all(&desktop_dir).ok();
    fs::create_dir_all(&icon_dir).ok();

    // Symlink binary into ~/.local/bin
    let bin_link = local_bin.join("tuxscale");
    if bin_link.exists() || bin_link.symlink_metadata().is_ok() {
        fs::remove_file(&bin_link).ok();
    }
    std::os::unix::fs::symlink(&current_exe, &bin_link)
        .expect("failed to create symlink in ~/.local/bin");

    // Write icon PNG
    let icon_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/icon.png"));
    let icon_path = icon_dir.join("tuxscale.png");
    fs::write(&icon_path, icon_bytes).expect("failed to write icon");

    // Write .desktop entry
    let desktop_contents = format!(
        "[Desktop Entry]\n\
         Name=Tuxscale\n\
         Comment=A native Linux GUI for the Tailscale VPN client\n\
         Exec={exe}\n\
         Icon=tuxscale\n\
         Terminal=false\n\
         Type=Application\n\
         Categories=Network;VPN;System;\n\
         Keywords=tailscale;vpn;network;tray;\n\
         StartupNotify=true\n\
         StartupWMClass=tuxscale\n",
        exe = current_exe.display()
    );
    let desktop_path = desktop_dir.join("tuxscale.desktop");
    fs::write(&desktop_path, desktop_contents).expect("failed to write .desktop file");

    println!("Tuxscale installed:");
    println!("  Binary  → {}", bin_link.display());
    println!("  Icon    → {}", icon_path.display());
    println!("  Menu    → {}", desktop_path.display());
    println!("\nYou can now run 'tuxscale' from the command line.");

    std::process::exit(0);
}
