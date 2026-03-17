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
        install().expect("install failed");
        println!("\nYou can now run 'tuxscale' from the command line.");
        std::process::exit(0);
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

pub fn install() -> std::io::Result<()> {
    use std::fs;
    use std::path::PathBuf;

    let current_exe = std::env::current_exe()?;
    let home = std::env::var("HOME").expect("HOME not set");

    let local_bin = PathBuf::from(&home).join(".local/bin");
    let desktop_dir = PathBuf::from(&home).join(".local/share/applications");
    let icon_dir = PathBuf::from(&home).join(".local/share/icons");

    fs::create_dir_all(&local_bin)?;
    fs::create_dir_all(&desktop_dir)?;
    fs::create_dir_all(&icon_dir)?;

    // Symlink binary into ~/.local/bin
    let bin_link = local_bin.join("tuxscale");
    if bin_link.exists() || bin_link.symlink_metadata().is_ok() {
        fs::remove_file(&bin_link)?;
    }
    std::os::unix::fs::symlink(&current_exe, &bin_link)?;

    // Write icon PNG
    let icon_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/icon.png"));
    fs::write(icon_dir.join("tuxscale.png"), icon_bytes)?;

    // Write .desktop entry
    let desktop = format!(
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
    fs::write(desktop_dir.join("tuxscale.desktop"), desktop)?;

    println!("Tuxscale installed:");
    println!("  Binary → {}", bin_link.display());

    Ok(())
}

/// Returns true if `tuxscale` is already reachable on PATH.
pub fn is_on_path() -> bool {
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|dir| std::path::Path::new(dir).join("tuxscale").exists())
}
