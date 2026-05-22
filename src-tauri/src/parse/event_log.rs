//! Windows Event Log analizi. BSOD stop code'larını yakalama amaçlı.

use memchr::memmem;

#[derive(Debug, Clone, Copy)]
pub enum BsodKind {
    MemoryManagement,
    CriticalProcessDied,
    DriverIrqlNotLessOrEqual,
    UnexpectedKernelModeTrap,
    NtfsFileSystem,
    PageFaultInNonpagedArea,
    Unknown,
}

impl BsodKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MemoryManagement => "MEMORY_MANAGEMENT",
            Self::CriticalProcessDied => "CRITICAL_PROCESS_DIED",
            Self::DriverIrqlNotLessOrEqual => "DRIVER_IRQL_NOT_LESS_OR_EQUAL",
            Self::UnexpectedKernelModeTrap => "UNEXPECTED_KERNEL_MODE_TRAP",
            Self::NtfsFileSystem => "NTFS_FILE_SYSTEM",
            Self::PageFaultInNonpagedArea => "PAGE_FAULT_IN_NONPAGED_AREA",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// XML event log buffer'ı içinde bilinen stop code'larını ara.
pub fn detect_bsod_kind(buffer: &[u8]) -> Option<BsodKind> {
    let patterns: &[(&[u8], BsodKind)] = &[
        (b"MEMORY_MANAGEMENT", BsodKind::MemoryManagement),
        (b"CRITICAL_PROCESS_DIED", BsodKind::CriticalProcessDied),
        (b"DRIVER_IRQL_NOT_LESS_OR_EQUAL", BsodKind::DriverIrqlNotLessOrEqual),
        (b"UNEXPECTED_KERNEL_MODE_TRAP", BsodKind::UnexpectedKernelModeTrap),
        (b"NTFS_FILE_SYSTEM", BsodKind::NtfsFileSystem),
        (b"PAGE_FAULT_IN_NONPAGED_AREA", BsodKind::PageFaultInNonpagedArea),
    ];
    for (pat, kind) in patterns {
        if memmem::find(buffer, pat).is_some() {
            return Some(*kind);
        }
    }
    None
}
