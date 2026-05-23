fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let mut attrs = tauri_build::Attributes::new();

    #[cfg(windows)]
    {
        // Dev profilinde requireAdministrator → asInvoker'a indir; cargo run
        // elevated olmayan terminalden requireAdministrator exe spawn edemez
        // (os error 740). Release build aynen UAC manifest'iyle çıkar.
        let raw = include_str!("app.manifest");
        let manifest_owned;
        let manifest_ref: &str = if std::env::var("PROFILE").as_deref() == Ok("debug") {
            manifest_owned = raw.replace(
                r#"level="requireAdministrator""#,
                r#"level="asInvoker""#,
            );
            &manifest_owned
        } else {
            raw
        };

        let win = tauri_build::WindowsAttributes::new().app_manifest(manifest_ref);
        attrs = attrs.windows_attributes(win);
    }

    tauri_build::try_build(attrs).expect("tauri-build başarısız");
}
