// Core configuration and initialization
export { initializeI18n, SUPPORTED_LANGUAGES, NAMESPACES } from './config';
export type { SupportedLanguage, Namespace } from './config';

// Hooks
export {
  useTranslation,
  useLanguageManager,
  useLocaleFormatting,
  useRTL,
} from './hooks';

// Components
export {
  I18nProvider,
  LanguageSelector,
  Translation,
  useLanguageContext,
} from './components';

// Formatters
export { formatters, getUserTimezone, getDateFormatPatterns } from './formatters';

// Default export
export { default as i18n } from './config';