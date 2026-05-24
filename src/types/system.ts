/** Backend `commands::system::system_stats` dönüş tipi — wmi::SystemSnapshot. */
export interface SystemStats {
  total_ram_gb: number;
  free_ram_gb: number;
  cpu_name: string;
  primary_disk_type: string;
  primary_disk_size_gb: number;
  primary_disk_free_gb: number;
  vbs_running: boolean;
}
