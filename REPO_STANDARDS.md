# D-Medic · Repo Standards

> **Hedef konum:** `C:\Projeler\dmedic\REPO_STANDARDS.md` (lokal klasör adı `dmedic`, remote `d-medic`)
> Reponun köküne kopyalayıp commit'leyin. Sonraki düzenlemeler bu dosyaya bağlı kalmalı — değişiklik gerekirse burayı da güncelleyin (drift önleme).
>
> **Snapshot:** 2026-05-26 (dmedic D Brand ailesine geç-katılım, d-terminal kanonik şablonuna align)
> **Bu repo Tauri tarafında MIT — D Brand açık-kaynak ailesindendir** (d-terminal / d-transfer / d-space ile aynı seri).

---

## 1. Locked GitHub metadata

| Alan | Değer |
|---|---|
| **Owner/Repo** | `AmrasElessar/d-medic` |
| **Visibility** | public |
| **Default branch** | `main` (2026-05-26'da `master` → `main` rename edildi, D Brand ailesi ile align) |
| **License (SPDX)** | `MIT` |
| **Description** | `Diagnose-first Windows 11 system optimizer — scans, prioritizes findings, applies only with consent. Revo-style uninstaller + custom NTFS defrag engine. Tauri v2 + Vue 3 + Rust.` |
| **Homepage** | `https://github.com/sponsors/AmrasElessar` |
| **Topics (19)** | `cleaner, defrag, defragmentation, desktop-app, diagnostics, ntfs, open-source, optimization, performance, rust, system-optimization, tauri, tauri-v2, uninstaller, vue, windows, windows-11, windows-rs, wmi` |

Değişiklik yaparsanız bu tabloyu güncelleyin + `gh repo edit` / `gh api PUT topics` ile remote'a yansıtın.

---

## 2. README iskeleti (D Brand template — kaynak: d-terminal)

D-Medic d-terminal'in kanonik README iskeletini takip eder.

### 2.1 Bölüm sırası (kanonik)

1. **Header** — center-aligned div: logo + başlık + İngilizce tagline + TR/EN alt-tagline + bilingual notice
2. **🎬 Demo** — screenshot grid + demo video / "coming soon" placeholder (`docs/media/`)
3. **Badge row** — License (MIT mavi) → Status → Platform (Windows 11 22H2+) → Tech (Tauri v2, Vue 3, Rust)
4. **🛡 Güvenlik / Security badges** — UAC: requireAdministrator → Network: 0 dependency / offline → Rollback: 3 layers → Diagnostics: native WMI (windows-rs)
5. **📌 Kısaca** (TR) + collapsible `🇬🇧 At a glance` (EN)
6. **🆕 Yenilikler / What's done so far**
7. **🎯 Vizyon / Vision** — "Diagnose-first, local, zero-network, three-layer rollback"
8. **✨ Öne Çıkan Özellikler / Key Features** — alt başlıklarla (`### 🔍 Tanı`, `### 🛠 Optimizasyon`, `### ⏮ Rollback`...)
9. **🛠️ Teknoloji / Tech Stack** — Tauri v2, Vue 3, Rust, windows-rs, WMI, Tailwind
10. **🗺️ Yol Haritası / Roadmap** — pre-alpha → alpha → beta → 1.0 checklist
11. **📥 Kurulum / Installation** + **🚀 İlk Adımlar / Quick Start** (release varsa)
12. **🛡️ Güvenlik Tarama / Security Scan Results** (release varsa — VirusTotal, signing, dependency audit)
13. **🤝 Katkı / Contributing** — issues + PR akışı
14. **🎨 D Brand Ailesi / D Brand Family** — diğer D Brand repo linkleri (7+)
15. **💖 Sponsorlar / Sponsors**
16. **❤️ Destekle / Support** — Star, Sponsor
17. **📜 Lisans / License** — MIT açıklaması

Pre-alpha status'a göre 11-12 düşürülebilir; **sıralama bozulmaz**.

### 2.2 Header pattern

```markdown
<div align="center">

<img src="src-tauri/icons/icon.png" width="128" alt="D-Medic logo" />

# D-Medic

**Tanı koyan Windows optimizasyon aracı / Diagnose-first Windows optimizer**

*Önce dinler, sonra konuşur — sistemi tarar, her bulguyu önceliklendirir, onayınla uygular*
*Scans first, then acts — prioritizes every finding and applies only with your consent*

🌐 **TR · EN** — Bu README iki dillidir / This README is bilingual (English collapsibles below each section)

</div>
```

### 2.3 Badge row

```
[License: MIT]                       (mavi)
[Status: pre-alpha]                  (turuncu)
[Platform: Windows 11 22H2+ · x64]   (mavi)
[Tauri v2]                           (cyan #24C8DB)
[Vue 3]                              (yeşil #4FC08D)
[Rust stable]                        (kırmızı #CE412B)

— Security row —
[UAC: requireAdministrator]
[Network: 0 dependency · offline]
[Rollback: 3 katman · layers]
[Diagnostics: native WMI (windows-rs)]
```

### 2.4 Bilingual yapı

- Ana akış TR + `<details><summary>🇬🇧 At a glance (English)</summary>` ile EN
- Güvenlik blok'u (UAC + offline + rollback) hem TR hem EN'de görünür kalır — premium-ish güvenlik kontratı

---

## 3. Tech stack & status

- **Status:** pre-alpha (release yok, Play Store / Store yok)
- **Platform:** Windows 11 22H2+ (x64), 2–8 GB RAM hedeflenir, HDD ve SSD
- **Privilege:** Manifest seviyesinde tek seferlik `requireAdministrator` UAC
- **Frontend:** Vue 3 + Vite + TypeScript + Tailwind CSS + PostCSS
- **Backend:** Rust (Tauri v2 — `src-tauri/`)
- **Native bindings:** `windows-rs`, WMI
- **Pkg manager:** pnpm (`pnpm-lock.yaml`)
- **Binary footprint:** ~10 MB hedefi (Electron'a karşı avantaj)
- **Network:** **0 runtime network dependency** — fully offline marka taahhüdü
- **Rollback:** 3 katman (snapshot/transaction/manuel-export) — yeni write operasyonu eklendiğinde hangi katmanın koruduğu README'de + kodda belirtilmeli

---

## 4. Lisans

- **MIT** — açık kaynak. README + `LICENSE` dosyası tutarlı (MIT).
- D Brand açık-kaynak ailesi: d-terminal (GPL-3.0), d-transfer, d-space, d-medic gibi.
- MIT seçimi kişisel kullanım + topluluk katkısı dengeyi destekler.

---

## 5. Commit mesaj stili

Conventional commits:

- `feat(readme): ...`, `fix(readme): ...`, `docs(readme): ...`
- `chore: ...` — config / dependency bump / `.idea/` & `.claude/` rebase
- `feat(<area>): ...` — kod (defrag, uninstaller, scan, ui, rollback, ...)
- `feat(tauri): ...` — Rust/Tauri tarafı

Dil: TR veya EN.

---

## 6. Dosya hijyeni

- Adı `:` veya `\` içeren dosyalar **commit'lenmez**.
- **Zorunlu:** `README.md`, `LICENSE` (MIT), `.github/FUNDING.yml` (Sponsors aktif)
- **`.gitignore`'a alınmalı:** `.idea/`, `.claude/`, `node_modules/`, `dist/`, `src-tauri/target/`, `*.log`, `.env`, `.env.*`
- **Repo köğünden taşınmalı/silinmeli:**
  - `D-Medic-Teknik-Dokuman-v1.1.docx` → `docs/` altına taşı, veya markdown'a çevir
  - `dmedic_logo.png` veya `yeni_logo.png` → kanonik birini seç, diğerini sil. Logoyu `src-tauri/icons/` altına standardize et.
  - `yedek.zip` benzeri yedek artıkları varsa → silinmeli (git zaten geçmişi tutar)
- Push öncesi `git status` kontrol.

---

## 7. Repo-spesifik notlar

- **Diagnose-first marka konumu** — README'deki "scans, prioritizes findings, applies only with consent" claim'i markanın özü. Kör optimizasyon yapan PR'lar **reddedilir**.
- **3-katman rollback kontratı** — README'de iddia edilen rollback layer sayısı (3) kod ile **senkron tutulmalı**. Yeni bir write operasyonu hangi katmanla korunuyorsa açıkça belirtilmeli (issue + PR description).
- **Native WMI via `windows-rs`** — `wmi` crate alternatifi yerine `windows-rs` doğrudan binding kullanılır. WMI üzerinden geçmeyen tanı API'lerinde de bu tercih korunmalı (PowerShell shell-out **yasak** — performans + güvenilirlik).
- **`requireAdministrator` manifest** — single-shot UAC. Per-action UAC anti-pattern; PR'larda bu kalıp reddedilir.
- **Zero-network claim** — runtime'da hiçbir host'a outbound bağlantı **yok**. Telemetry/analytics/auto-update bile **yok**. PR'da network çağrısı varsa kullanıcıya açık opt-in olmalı + README'de belirtilmeli.
- **Rust unsafe** — `windows-rs` FFI dışında `unsafe` blok kullanımı PR review'da açıkça gerekçelendirilmeli (`# Safety` doc comment).
- **Tauri v2 CSP** — `src-tauri/tauri.conf.json`'da CSP `'unsafe-inline'` / `'unsafe-eval'` içermemeli; gerekirse nonce kullan.
- **Branch adı** — default branch `main` (2026-05-26'da `master` → `main` rename edildi, D Brand ailesi ile align).
- **Binary boyut bütçesi:** ~10 MB. PR'da bu eşik aşılıyorsa gerekçelendirme + bundle analiz şart.
- **`D-Medic-Teknik-Dokuman-v1.1.docx`** — `.docx` kaynak kod repo'sunda olmamalı; markdown'a çevir veya `docs/` altına taşı.
- **Logo ikilemi** — `dmedic_logo.png` ile `yeni_logo.png` repo kökünde. Kanonik = `src-tauri/icons/icon.png` (Tauri build için). README + brand'de kullanılan dosya tek olmalı.
- **Pre-alpha disiplin** — şu an test yok. Alpha'ya geçişten önce: (1) `src-tauri` Rust unit testleri, (2) frontend Vitest, (3) Playwright Windows E2E iskeleti zorunlu.
