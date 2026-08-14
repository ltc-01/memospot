//! System tray icon and menu.
//!
//! The tray icon is created at application startup and lets the user restore
//! the main window when it is minimized to the tray on close, or quit the
//! application.

use crate::fl;
use crate::window::Window;
use log::debug;
use tauri::menu::{Menu, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Runtime};

/// Tray icon identifier, used to look up the tray icon at runtime.
pub(crate) const TRAY_ID: &str = "memospot-tray";

const MENU_OPEN: &str = "tray-open";
const MENU_QUIT: &str = "tray-quit";

/// Show and focus the main window.
///
/// Used to restore the window from the system tray.
pub(crate) fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window(Window::Main.into()) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Create the system tray icon with a menu to restore or quit the app.
pub(crate) fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let Some(icon) = app.default_window_icon() else {
        debug!("no default window icon available; skipping system tray creation");
        return Ok(());
    };

    let open = MenuItemBuilder::with_id(MENU_OPEN, fl!("tray-open")).build(app)?;
    let quit = MenuItemBuilder::with_id(MENU_QUIT, fl!("appmenu-quit")).build(app)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon.clone())
        .menu(&menu)
        // On Windows the menu opens on right-click, so a left-click restores
        // the main window directly. On macOS and Linux the menu opens on
        // left-click instead (macOS), or click events are not delivered (Linux).
        .show_menu_on_left_click(cfg!(not(target_os = "windows")))
        .on_menu_event(|app, event| match event.id().0.as_str() {
            MENU_OPEN => show_main_window(app),
            MENU_QUIT => {
                debug!("quitting from system tray");
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    debug!("system tray icon created");
    Ok(())
}
