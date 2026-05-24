/** Backend `commands::system::list_disks` dönüş tipi — wmi::PhysicalDiskInfo. */
export interface PhysicalDiskInfo {
  friendly_name: string;
  /** "HDD" | "SSD" | "SCM" | "Unknown" */
  media_type: string;
  size_gb: number;
  device_id: string;
}

/** Backend `commands::system::system_stats` dönüş tipi — wmi::SystemSnapshot. */
export interface SystemStats {
  total_ram_gb: number;
  free_ram_gb: number;
  cpu_name: string;
  primary_disk_type: string;
  primary_disk_size_gb: number;
  primary_disk_free_gb: number;
  vbs_running: boolean;
  efi_size_mb: number;
  efi_free_mb: number;
  uwp_package_count: number;
  installed_app_count: number;
}
