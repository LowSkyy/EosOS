fn main() {
    // Manifesto proprio: quello di Tauri piu' la compatibilita' con Windows 8+ (finestre figlie «a strati», vedi lights.rs).
    let windows = tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest"));
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows)).expect("tauri build");
}
