fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=PROFILE");

    // Build metadata — AboutView'da göstermek için env var olarak inject.
    let git_rev = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=D_MEDIC_GIT_REV={git_rev}");

    let build_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    println!("cargo:rustc-env=D_MEDIC_BUILD_DATE={build_date}");

    let mut attrs = tauri_build::Attributes::new();

    #[cfg(windows)]
    {
        // Release build → requireAdministrator (kullanıcı exe'yi çift tıkladığında
        // Windows UAC promptu çıkar). Debug build → asInvoker (cargo elevated
        // olmayan terminalden requireAdministrator exe spawn edemez, os error 740).
        // Dev'de admin davranışını test etmek için terminali "Yönetici olarak
        // çalıştır" ile aç; release exe her durumda admin yetkili çalışır.
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
