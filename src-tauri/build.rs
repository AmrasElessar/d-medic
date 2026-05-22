fn main() {
    #[cfg(windows)]
    {
        // Windows uygulama manifestini gömerek manifest seviyesinde
        // requireAdministrator yetkisi talep ediyoruz. Bu sayede Tauri
        // içinden per-komut UAC tetiklenmez, kullanıcı sadece uygulamayı
        // başlatırken bir kez onay verir.
        let mut res = winresource::WindowsResource::new();
        res.set_manifest(include_str!("app.manifest"));
        if let Err(e) = res.compile() {
            // tauri-build kendi kaynak embed'ini yapacağı için sessiz uyarı
            // verip devam ediyoruz; release build'te critical olmamalı.
            eprintln!("winresource manifest embed başarısız: {e}");
        }
    }
    tauri_build::build();
}
