pub mod creds;
pub mod parsers;
pub mod poll;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri_nspanel::{
    tauri_panel, CollectionBehavior, ManagerExt, PanelLevel, StyleMask, WebviewWindowExt,
};
use tauri_plugin_window_state::StateFlags;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};

tauri_panel! {
    panel!(ManaPanel {
        config: {
            can_become_key_window: false,
            can_become_main_window: false,
            is_floating_panel: true
        }
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(StateFlags::POSITION)
                .build(),
        )
        .manage(poll::Snapshots::default())
        .setup(|app| {
            // Menu-bar-only app: no Dock icon, never activates as a regular app.
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app.get_webview_window("main").unwrap();

            // Glass blur behind the webview. Active state is required: a
            // non-activating panel is never key, and FollowsWindowActiveState
            // would render the material permanently dim.
            apply_vibrancy(
                &window,
                NSVisualEffectMaterial::HudWindow,
                Some(NSVisualEffectState::Active),
                Some(14.0),
            )?;

            // Non-activating floating panel: hovers over every window and
            // fullscreen Space without ever stealing keyboard focus.
            let panel = window.to_panel::<ManaPanel>()?;
            panel.set_level(PanelLevel::Floating.value());
            panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
            panel.set_collection_behavior(
                CollectionBehavior::new()
                    .can_join_all_spaces()
                    .full_screen_auxiliary()
                    .stationary()
                    .into(),
            );
            panel.show(); // orderFrontRegardless — no activation

            let toggle = MenuItem::with_id(app, "toggle", "Show / Hide", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit mana", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "toggle" => {
                        if let (Some(win), Ok(panel)) =
                            (app.get_webview_window("main"), app.get_webview_panel("main"))
                        {
                            if win.is_visible().unwrap_or(true) {
                                panel.hide();
                            } else {
                                panel.show();
                            }
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            poll::spawn_pollers(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![poll::get_snapshots])
        .run(tauri::generate_context!())
        .expect("error while running mana");
}
