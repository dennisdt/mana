pub mod activity;
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

/// Corner radius (points) of the macOS vibrancy blur layer. Must stay equal
/// to `--hud-radius` in `src/styles.css`, or the blur's rounded corner and
/// the CSS border corner visibly diverge.
const HUD_CORNER_RADIUS: f64 = 8.0;

fn tray_template_icon() -> tauri::Result<tauri::image::Image<'static>> {
    tauri::image::Image::from_bytes(include_bytes!("../icons/tray-template.png"))
        .map(tauri::image::Image::to_owned)
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
                Some(HUD_CORNER_RADIUS),
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
            let quit = MenuItem::with_id(app, "quit", "Quit Mana", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &quit])?;
            TrayIconBuilder::new()
                .icon(tray_template_icon()?)
                .icon_as_template(true)
                .tooltip("Mana")
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
            activity::spawn_activity_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![poll::get_snapshots, activity::get_activity])
        .run(tauri::generate_context!())
        .expect("error while running mana");
}

#[cfg(test)]
mod tests {
    use super::{tray_template_icon, HUD_CORNER_RADIUS};

    #[test]
    fn vibrancy_radius_matches_css_hud_radius() {
        let css = include_str!("../../src/styles.css");
        let expected = format!("--hud-radius: {HUD_CORNER_RADIUS}px;");
        assert!(
            css.contains(&expected),
            "--hud-radius in src/styles.css must equal HUD_CORNER_RADIUS ({HUD_CORNER_RADIUS}px)"
        );
    }

    #[test]
    fn tray_template_asset_has_expected_geometry() {
        let image = tray_template_icon().expect("tray template PNG should decode");
        assert_eq!(image.width(), 36);
        assert_eq!(image.height(), 36);

        let rgba = image.rgba();
        let alpha_at = |x: usize, y: usize| rgba[(y * 36 + x) * 4 + 3];
        for (x, y) in [(0, 0), (35, 0), (0, 35), (35, 35)] {
            assert_eq!(alpha_at(x, y), 0, "corner ({x}, {y}) must be transparent");
        }

        let opaque_pixels = rgba.chunks_exact(4).filter(|pixel| pixel[3] > 0).count();
        assert!(opaque_pixels > 0, "tray icon must contain visible pixels");
        assert!(
            opaque_pixels < 36 * 36 * 70 / 100,
            "tray icon must leave most of the canvas transparent"
        );

        let visible = (0..36).flat_map(|y| {
            (0..36)
                .filter(move |&x| alpha_at(x, y) > 0)
                .map(move |x| (x, y))
        });
        let (min_x, max_x, min_y, max_y) = visible.fold(
            (usize::MAX, 0, usize::MAX, 0),
            |(min_x, max_x, min_y, max_y), (x, y)| {
                (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
            },
        );
        let center_x = (min_x + max_x) as f32 / 2.0;
        let center_y = (min_y + max_y) as f32 / 2.0;
        assert!((center_x - 17.5).abs() <= 2.0);
        assert!((center_y - 17.5).abs() <= 2.0);
    }
}
