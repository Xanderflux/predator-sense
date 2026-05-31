mod app_state;
mod config;
mod hardware;
pub mod i18n;
mod tray;
mod ui;

use gtk4::prelude::*;
use gtk4::{self as gtk, gdk, glib};
use libadwaita as adw;

const APP_ID: &str = "com.predator.sense";
const CSS_THEME: &str = include_str!("../resources/style.css");

fn main() {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|_| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(CSS_THEME);
        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not get default display"),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Set application window icon via icon theme search path
        if let Some(path) = find_icon_path() {
            if let Some(dir) = std::path::Path::new(&path).parent() {
                let theme = gtk::IconTheme::for_display(&gdk::Display::default().unwrap());
                theme.add_search_path(dir.to_str().unwrap_or(""));
            }
        }
    });

    app.connect_activate(|app| {
        config::ensure_dirs();

        // Single instance: if window exists, present it
        if let Some(window) = app.active_window() {
            app_state::set_window_visible(true);
            window.set_visible(true);
            window.present();
            // Force a full redraw after the WM finishes mapping the window.
            // GTK4 + Cinnamon sometimes leaves the surface blank when reshowing
            // a window that was hidden via set_visible(false).
            let win = window.clone();
            glib::idle_add_local_once(move || {
                win.queue_resize();
                win.queue_draw();
                if let Some(child) = win.child() {
                    child.queue_resize();
                    child.queue_draw();
                }
            });
            return;
        }

        ui::window::build(app);
    });

    app.run_with_args::<String>(&[]);
}

fn find_icon_path() -> Option<String> {
    let candidates = [
        "resources/logo-128.png",
        "../resources/logo-128.png",
        "../../resources/logo-128.png",
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for c in &candidates {
                let p = dir.join(c);
                if p.exists() { return Some(p.to_string_lossy().to_string()); }
            }
        }
    }
    let dev = "/opt/predator-sense/resources/logo-128.png";
    if std::path::Path::new(dev).exists() { return Some(dev.to_string()); }
    None
}
