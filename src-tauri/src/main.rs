// D-Medic her zaman GUI olarak çalışır — debug build dahil console penceresi
// açılmaz. Geliştirici log'ları %APPDATA%\D-Medic\logs\dmedic.<tarih>.log
// dosyasından (Settings > Loglar > "Klasörü Aç") okur veya
// `cargo run 2>&1 | cat` ile terminal'den çalıştırarak stderr'i toplar.
#![windows_subsystem = "windows"]

fn main() {
    d_medic_lib::run();
}
