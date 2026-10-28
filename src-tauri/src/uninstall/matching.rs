//! Kalıntı taraması için ad eşleştirme kuralları (saf fonksiyonlar).
//!
//! Klasör / registry anahtarı adı, kaldırılan programın ürün anahtarıyla
//! karşılaştırılır. Ortak alanlar ([`safety::is_protected_name`]) ve yayıncı
//! klasörleri asla eşleşme sayılmaz — örn. "Microsoft Edge" kaldırılınca
//! `Microsoft` klasörü/anahtarı aday olmaz.

use crate::safety::{self, norm_key};

/// Ad/yayıncıdan üretilen eşleştirme anahtarları.
#[derive(Debug, Clone)]
pub struct Tokens {
    /// Alfanümerik-küçük harf ürün anahtarı (örn. "AcmeApp 3.2" → "acmeapp").
    /// Ürün adı korumalı bir ada indirgeniyorsa boştur (eşleşme üretmez).
    pub product_key: String,
    /// Yayıncı anahtarı (varsa).
    pub publisher_key: Option<String>,
}

/// Eşleşme gücü — çağıran bunu güven seviyesine çevirir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strength {
    /// Ad, ürün anahtarıyla birebir aynı.
    Exact,
    /// Ad, ürün anahtarını içeriyor (örn. "AcmeAppData").
    Strong,
    /// Ad, ürün anahtarının büyük bir ön eki (örn. "Acme" ↔ "acmeapp").
    Weak,
}

/// "3.2", "23.01", "v1.2.3" gibi sürüm sözcükleri.
fn is_version_word(w: &str) -> bool {
    let w = w.trim_start_matches(['v', 'V']);
    w.starts_with(|c: char| c.is_ascii_digit())
        && w.contains('.')
        && w.chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
}

/// Ürün adından sürüm/parantez gürültüsünü ayıklayıp anahtar üret.
/// "Acme App 3.2 (x64)" → "acmeapp", "7-Zip 23.01 (x64)" → "7zip".
pub fn product_token(name: &str) -> String {
    let cut = name.split(['(', '[']).next().unwrap_or(name);
    let mut words: Vec<&str> = Vec::new();
    for (i, w) in cut.split_whitespace().enumerate() {
        if i > 0 && is_version_word(w) {
            break;
        }
        words.push(w);
    }
    let key = norm_key(&words.join(" "));
    if key.len() >= 3 {
        key
    } else {
        norm_key(cut)
    }
}

pub fn build_tokens(name: &str, publisher: Option<&str>) -> Tokens {
    let publisher_key = publisher.map(norm_key).filter(|k| k.len() >= 4);
    let mut product_key = product_token(name);
    if safety::is_protected_name(&product_key) {
        product_key.clear();
    }
    Tokens {
        product_key,
        publisher_key,
    }
}

/// Bir klasör/anahtar adının ürün anahtarıyla eşleşme gücü.
pub fn match_strength(name: &str, tokens: &Tokens) -> Option<Strength> {
    let key = norm_key(name);
    let product = &tokens.product_key;
    if key.is_empty() || product.len() < 3 || safety::is_protected_name(name) {
        return None;
    }
    // Yayıncı klasörü (ürün adıyla aynı değilse) birden çok ürünü barındırır.
    if tokens.publisher_key.as_deref() == Some(key.as_str()) && key != *product {
        return None;
    }
    if key == *product {
        return Some(Strength::Exact);
    }
    if product.len() >= 4 && key.contains(product.as_str()) {
        return Some(Strength::Strong);
    }
    // Ters yön yalnız ön ek ve ürün anahtarının en az 2/3'ü kadar uzunsa:
    // "Google" ↔ "googlechrome" gibi ortak yayıncı adları elenir.
    if key.len() >= 5 && product.starts_with(key.as_str()) && key.len() * 3 >= product.len() * 2 {
        return Some(Strength::Weak);
    }
    None
}

pub fn is_publisher_dir(name: &str, tokens: &Tokens) -> bool {
    if safety::is_protected_name(name) {
        return false;
    }
    match &tokens.publisher_key {
        Some(pk) => {
            let k = norm_key(name);
            !k.is_empty() && (k == *pk || (pk.len() >= 5 && k.contains(pk.as_str())))
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_token_strips_versions() {
        assert_eq!(product_token("Acme App 3.2 (x64)"), "acmeapp");
        assert_eq!(product_token("7-Zip 23.01 (x64)"), "7zip");
        assert_eq!(product_token("Python 3.12.0 (64-bit)"), "python");
        assert_eq!(product_token("Visual Studio 2019"), "visualstudio2019");
        assert_eq!(product_token("Notepad++ v8.6.2"), "notepad");
        assert_eq!(
            product_token("Mozilla Firefox (x64 en-US)"),
            "mozillafirefox"
        );
    }

    #[test]
    fn microsoft_products_never_match_shared_roots() {
        let t = build_tokens("Microsoft Edge", Some("Microsoft Corporation"));
        assert_eq!(match_strength("Microsoft", &t), None);
        assert_eq!(match_strength("Windows", &t), None);
        assert_eq!(match_strength("Microsoft Edge", &t), Some(Strength::Exact));
        assert!(!is_publisher_dir("Microsoft", &t));
    }

    #[test]
    fn generic_product_names_produce_no_matches() {
        let t = build_tokens("Microsoft 365", Some("Microsoft Corporation"));
        assert_eq!(t.product_key, "microsoft365");
        let t = build_tokens("Windows 1.0", None);
        assert!(t.product_key.is_empty());
        assert_eq!(match_strength("Windows", &t), None);
    }

    #[test]
    fn shared_publisher_dirs_are_not_leftovers() {
        let t = build_tokens("Google Chrome", Some("Google"));
        assert_eq!(match_strength("Google", &t), None);
        assert!(is_publisher_dir("Google", &t));
        assert_eq!(match_strength("Chrome", &t), None);
        assert_eq!(match_strength("GoogleChrome", &t), Some(Strength::Exact));

        let t = build_tokens("Mozilla Firefox", Some("Mozilla"));
        assert_eq!(match_strength("Mozilla", &t), None);
        assert_eq!(match_strength("Mozilla Firefox", &t), Some(Strength::Exact));
    }

    #[test]
    fn product_equal_to_publisher_still_matches() {
        let t = build_tokens("Spotify", Some("Spotify"));
        assert_eq!(match_strength("Spotify", &t), Some(Strength::Exact));
    }

    #[test]
    fn strengths() {
        let t = build_tokens("AcmeApp", None);
        assert_eq!(match_strength("acme-app", &t), Some(Strength::Exact));
        assert_eq!(match_strength("AcmeAppData", &t), Some(Strength::Strong));
        let t = build_tokens("Acme Studio", None);
        assert_eq!(match_strength("AcmeStud", &t), Some(Strength::Weak));
        assert_eq!(match_strength("Acmes", &t), None);
        assert_eq!(match_strength("Unrelated", &t), None);
    }
}
