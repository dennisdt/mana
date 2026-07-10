pub mod creds;
pub mod parsers;
pub mod poll;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(poll::Snapshots::default())
        .setup(|app| {
            poll::spawn_pollers(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![poll::get_snapshots])
        .run(tauri::generate_context!())
        .expect("error while running mana");
}
