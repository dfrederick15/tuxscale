use tray_icon::{
    TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuId, MenuItem},
};

const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

pub struct Tray {
    // Kept alive for process lifetime; must not be dropped.
    pub _icon: TrayIcon,
    pub show_id: MenuId,
    pub quit_id: MenuId,
}

pub fn init() -> Tray {
    let icon = load_icon();

    let show_item = MenuItem::new("Show / Hide", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    let show_id = show_item.id().clone();
    let quit_id = quit_item.id().clone();

    let menu = Menu::new();
    menu.append(&show_item).unwrap();
    menu.append(&quit_item).unwrap();

    let icon_handle = TrayIconBuilder::new()
        .with_icon(icon)
        .with_tooltip("Tuxscale")
        .with_menu(Box::new(menu))
        .build()
        .expect("failed to build tray icon");

    Tray { _icon: icon_handle, show_id, quit_id }
}

fn load_icon() -> tray_icon::Icon {
    let img = image::load_from_memory(ICON_PNG)
        .expect("invalid icon PNG")
        .into_rgba8();
    let (w, h) = img.dimensions();
    tray_icon::Icon::from_rgba(img.into_raw(), w, h).expect("failed to create tray icon")
}

pub fn try_recv() -> Option<MenuId> {
    MenuEvent::receiver().try_recv().ok().map(|e| e.id)
}
