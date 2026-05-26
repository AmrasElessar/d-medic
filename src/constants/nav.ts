import {
  LayoutDashboard,
  Stethoscope,
  ClipboardList,
  Play,
  BookOpen,
  History,
  Settings,
  Info,
  Trash2,
  HardDriveDownload,
} from 'lucide-vue-next';
import type { Component } from 'vue';
import type { ViewKey } from '@/stores/nav';

export interface NavEntry {
  key: ViewKey;
  labelKey: string;
  icon: Component;
  group: 'main' | 'tools' | 'system';
}

export const NAV_ENTRIES: NavEntry[] = [
  { key: 'dashboard', labelKey: 'nav.dashboard', icon: LayoutDashboard,    group: 'main' },
  { key: 'scan',      labelKey: 'nav.scan',      icon: Stethoscope,        group: 'main' },
  { key: 'plan',      labelKey: 'nav.plan',      icon: ClipboardList,      group: 'main' },
  { key: 'execute',   labelKey: 'nav.execute',   icon: Play,               group: 'main' },
  { key: 'guides',    labelKey: 'nav.guides',    icon: BookOpen,           group: 'main' },
  { key: 'history',   labelKey: 'nav.history',   icon: History,            group: 'main' },
  { key: 'uninstall', labelKey: 'nav.uninstall', icon: Trash2,             group: 'tools' },
  { key: 'defrag',    labelKey: 'nav.defrag',    icon: HardDriveDownload,  group: 'tools' },
  { key: 'settings',  labelKey: 'nav.settings',  icon: Settings,           group: 'system' },
  { key: 'about',     labelKey: 'nav.about',     icon: Info,               group: 'system' },
];
