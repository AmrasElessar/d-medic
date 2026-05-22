import { createI18n } from 'vue-i18n';
import tr from './locales/tr.json';
import en from './locales/en.json';

export type SupportedLocale = 'tr' | 'en';

export const i18n = createI18n({
  legacy: false,
  locale: 'tr' as SupportedLocale,
  fallbackLocale: 'en' as SupportedLocale,
  messages: { tr, en },
});
