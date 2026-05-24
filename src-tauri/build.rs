fn main() {
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=.git/HEAD");

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
        // D-Medic her zaman yönetici yetkisiyle çalışır — dev ve release ayrımı yok.
        // pnpm tauri:dev exe'yi spawn ettiğinde Windows UAC promptu çıkar; kullanıcı
        // Evet derse pencere yetkili, Hayır derse spawn başarısız (os error 740).
        let manifest = include_str!("app.manifest");
        let win = tauri_build::WindowsAttributes::new().app_manifest(manifest);
        attrs = attrs.windows_attributes(win);
    }

    tauri_build::try_build(attrs).expect("tauri-build başarısız");
}
