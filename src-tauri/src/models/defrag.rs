//! Defrag motoru veri modelleri — frontend ile paylaşılan sözleşme.
//!
//! Akış: [`VolumeInfo`] listele → [`FragmentationReport`] analiz et →
//! [`ClusterMap`] görselleştir → [`DefragMode`] seç → taşıma sırasında
//! [`DefragProgress`] event'leri akar.

use serde::{Deserialize, Serialize};

/// Bir birim (sürücü harfi) hakkında geometri + kullanım özeti.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    /// "C", "D" — harf (iki nokta veya backslash olmadan).
    pub letter: String,
    /// "NTFS", "ReFS", "FAT32" ... Defrag yalnızca NTFS'te tam desteklenir.
    pub file_system: String,
    /// Disk türü — SSD'de full defrag yapılmaz (yalnız analiz/TRIM).
    pub media_type: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub cluster_bytes: u32,
    pub total_clusters: u64,
    pub free_clusters: u64,
    /// Bu birimde defrag taşıması güvenli/desteklenen mi (NTFS + non-SSD).
    pub defrag_supported: bool,
}

/// Tek bir parçalı dosya — "en parçalı N dosya" listesinde gösterilir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileFrag {
    pub path: String,
    /// Extent (parça) sayısı. 1 = bütünleşik.
    pub fragments: u32,
    pub size_bytes: u64,
}

/// Birim genelinde parçalanma analizi.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FragmentationReport {
    pub letter: String,
    /// 0-100 — parçalı dosyaların / parçalanmış kümelerin oranı.
    pub fragmentation_percent: f32,
    pub total_files: u64,
    pub fragmented_files: u64,
    /// En çok parçalanmış dosyalar (azalan, kapaklı liste).
    pub most_fragmented: Vec<FileFrag>,
    /// Analiz ne kadar sürdü (ms) — UI teşhis.
    pub elapsed_ms: u64,
    /// Tam defrag mı yoksa sadece analiz mi önerilir (SSD vs HDD + eşik).
    pub recommendation: String,
}

/// Cluster haritası hücre durumu. Birim bitmap'i UI grid'ine downsample edilir;
/// her hücre binlerce kümeyi temsil eder, baskın duruma göre renklenir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CellState {
    /// Çoğunlukla boş.
    Free,
    /// Dolu, bütünleşik veriler.
    Used,
    /// Bu bölgede parçalı dosya kümeleri var.
    Fragmented,
    /// Taşınamaz bölge (MFT zone, pagefile, $BadClus).
    Unmovable,
    /// Şu an aktif taşınıyor (canlı tick sırasında).
    Moving,
}

/// UI için downsample edilmiş cluster haritası.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClusterMap {
    pub letter: String,
    pub cols: u32,
    pub rows: u32,
    /// row-major, cols*rows uzunluğunda hücre durumları.
    pub cells: Vec<CellState>,
    /// Bir hücrenin temsil ettiği küme sayısı (UI tooltip).
    pub clusters_per_cell: u64,
}

/// Taşıma modu — kullanıcı seçer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefragMode {
    /// Yalnız analiz — hiçbir küme taşınmaz.
    AnalyzeOnly,
    /// Hızlı: yalnız parçalı dosyaları bütünleştir.
    Quick,
    /// Tam: parça birleştirme + boş alan konsolidasyonu.
    Full,
    /// Yalnız boş alanı diskin sonuna doğru toparla.
    FreeSpaceConsolidate,
}

/// Defrag sırasında yayılan ilerleme event'i (`defrag-progress`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefragProgress {
    pub job_id: String,
    /// "analyzing" | "moving" | "consolidating" | "done" | "cancelled" | "error"
    pub phase: String,
    /// İşlenmekte olan dosya (taşıma fazında).
    pub current_file: Option<String>,
    /// Şimdiye dek taşınan küme sayısı.
    pub clusters_moved: u64,
    /// Hedeflenen toplam taşınacak küme sayısı (tahmini).
    pub clusters_total: u64,
    pub files_processed: u64,
    pub percent: f32,
}
