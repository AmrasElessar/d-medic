# D-Medic

Windows 11 sistem tanılama & optimizasyon toolkit'i. **Önce tanı, sonra tedavi.**

- **Platform**: Windows 11 (22H2+)
- **Stack**: Tauri 2 + Vue 3 + TypeScript + Rust
- **Lisans**: MIT
- **Hedef**: 2-8 GB RAM, HDD/SSD, Win11 kurulu sistemler

## Felsefe

D-Medic kör optimizasyon yapmaz. PC'yi baştan aşağı tarar, her bulguyu **OTOMATİK / KILAVUZ / MÜMKÜN DEĞİL** olarak etiketler, öncelik + tahmini kazanım ile sıralar ve kullanıcının onayı ile uygular. Tüm otomatik işlemler geri alınabilir.

## Geliştirme

### Önkoşullar

- Node.js 20+ ve [pnpm](https://pnpm.io/)
- Rust 1.75+ (`rustup default stable`)
- Visual Studio Build Tools 2022 (Windows SDK + C++ workload)
- WebView2 Runtime (Win11'de gelir)

### Komutlar

```bash
pnpm install              # bağımlılıkları kur
pnpm tauri:dev            # geliştirme — admin UAC promptu açılır
pnpm tauri:build          # NSIS installer üret (target/release/bundle/)
```

D-Medic baştan sona admin yetkisiyle çalışır. `tauri:dev` ilk başlatmada Windows UAC penceresi açar — onayladıktan sonra dev sunucusu yükselmiş bağlamda çalışır.

### Proje Yapısı

```
src/                  # Vue 3 frontend
src-tauri/            # Rust backend + Tauri config
src-tauri/resources/  # Bundle edilen statik (kılavuz JSON'ları)
```

Detaylı mimari ve 28 tanılama kalemi için bkz. `D-Medic-Teknik-Dokuman-v1.1.docx`.

## Sürüm Notu

`v0.1.0` — geliştirme başlangıcı. Faz 1: tanılama motoru + rollback altyapısı.
