import {
  Zap,
  ShieldCheck,
  Lock,
  Cpu,
  HardDrive,
  Network,
  BatteryCharging,
  DatabaseBackup,
} from 'lucide-vue-next';
import type { Component } from 'vue';
import type { Category } from '@/types';

export const CATEGORY_META: Record<Category, { icon: Component; labelKey: string }> = {
  performance:   { icon: Zap,              labelKey: 'category.performance' },
  stability:     { icon: ShieldCheck,      labelKey: 'category.stability' },
  security:      { icon: Lock,             labelKey: 'category.security' },
  compatibility: { icon: Cpu,              labelKey: 'category.compatibility' },
  storage:       { icon: HardDrive,        labelKey: 'category.storage' },
  network:       { icon: Network,          labelKey: 'category.network' },
  power:         { icon: BatteryCharging,  labelKey: 'category.power' },
  data:          { icon: DatabaseBackup,   labelKey: 'category.data' },
};
