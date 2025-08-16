import { format, formatDistanceToNow, parseISO } from 'date-fns';
import { formatInTimeZone } from 'date-fns-tz';
import { enUS, es, fr, de, ja, zhCN, ar, he } from 'date-fns/locale';
import { SupportedLanguage } from './config';

// Date-fns locale mapping
const DATE_FNS_LOCALES = {
  en: enUS,
  es: es,
  fr: fr,
  de: de,
  ja: ja,
  zh: zhCN,
  ar: ar,
  he: he,
} as const;

// Number formatting locales
const NUMBER_LOCALES = {
  en: 'en-US',
  es: 'es-ES',
  fr: 'fr-FR',
  de: 'de-DE',
  ja: 'ja-JP',
  zh: 'zh-CN',
  ar: 'ar-SA',
  he: 'he-IL',
} as const;

// Currency codes by locale
const CURRENCY_CODES = {
  en: 'USD',
  es: 'EUR',
  fr: 'EUR',
  de: 'EUR',
  ja: 'JPY',
  zh: 'CNY',
  ar: 'SAR',
  he: 'ILS',
} as const;

export const formatters = {
  // Date formatting
  formatDate: (language: SupportedLanguage) => (
    date: Date | string | number,
    formatString: string = 'PP'
  ): string => {
    const dateObj = typeof date === 'string' ? parseISO(date) : new Date(date);
    const locale = DATE_FNS_LOCALES[language] || DATE_FNS_LOCALES.en;
    return format(dateObj, formatString, { locale });
  },

  // Time formatting
  formatTime: (language: SupportedLanguage) => (
    date: Date | string | number,
    formatString: string = 'p'
  ): string => {
    const dateObj = typeof date === 'string' ? parseISO(date) : new Date(date);
    const locale = DATE_FNS_LOCALES[language] || DATE_FNS_LOCALES.en;
    return format(dateObj, formatString, { locale });
  },

  // DateTime formatting
  formatDateTime: (language: SupportedLanguage) => (
    date: Date | string | number,
    formatString: string = 'PPp'
  ): string => {
    const dateObj = typeof date === 'string' ? parseISO(date) : new Date(date);
    const locale = DATE_FNS_LOCALES[language] || DATE_FNS_LOCALES.en;
    return format(dateObj, formatString, { locale });
  },

  // DateTime with timezone
  formatDateTimeWithTZ: (language: SupportedLanguage) => (
    date: Date | string | number,
    timeZone: string,
    formatString: string = 'PPp'
  ): string => {
    const dateObj = typeof date === 'string' ? parseISO(date) : new Date(date);
    const locale = DATE_FNS_LOCALES[language] || DATE_FNS_LOCALES.en;
    return formatInTimeZone(dateObj, timeZone, formatString, { locale });
  },

  // Relative time formatting
  formatRelativeTime: (language: SupportedLanguage) => (
    date: Date | string | number
  ): string => {
    const dateObj = typeof date === 'string' ? parseISO(date) : new Date(date);
    const locale = DATE_FNS_LOCALES[language] || DATE_FNS_LOCALES.en;
    return formatDistanceToNow(dateObj, { addSuffix: true, locale });
  },

  // Number formatting
  formatNumber: (language: SupportedLanguage) => (
    number: number,
    options?: Intl.NumberFormatOptions
  ): string => {
    const locale = NUMBER_LOCALES[language] || NUMBER_LOCALES.en;
    return new Intl.NumberFormat(locale, options).format(number);
  },

  // Currency formatting
  formatCurrency: (language: SupportedLanguage) => (
    amount: number,
    currency?: string,
    options?: Intl.NumberFormatOptions
  ): string => {
    const locale = NUMBER_LOCALES[language] || NUMBER_LOCALES.en;
    const currencyCode = currency || CURRENCY_CODES[language] || CURRENCY_CODES.en;
    
    return new Intl.NumberFormat(locale, {
      style: 'currency',
      currency: currencyCode,
      ...options,
    }).format(amount);
  },

  // Percentage formatting
  formatPercent: (language: SupportedLanguage) => (
    number: number,
    options?: Intl.NumberFormatOptions
  ): string => {
    const locale = NUMBER_LOCALES[language] || NUMBER_LOCALES.en;
    return new Intl.NumberFormat(locale, {
      style: 'percent',
      ...options,
    }).format(number);
  },

  // File size formatting
  formatFileSize: (language: SupportedLanguage) => (
    bytes: number,
    decimals: number = 2
  ): string => {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];

    const i = Math.floor(Math.log(bytes) / Math.log(k));
    const size = parseFloat((bytes / Math.pow(k, i)).toFixed(dm));
    
    return formatters.formatNumber(language)(size) + ' ' + sizes[i];
  },

  // Duration formatting (in seconds)
  formatDuration: (language: SupportedLanguage) => (
    seconds: number
  ): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;

    const parts: string[] = [];
    
    if (hours > 0) {
      parts.push(`${hours}h`);
    }
    if (minutes > 0) {
      parts.push(`${minutes}m`);
    }
    if (remainingSeconds > 0 || parts.length === 0) {
      parts.push(`${remainingSeconds}s`);
    }

    return parts.join(' ');
  },
};

// Utility function to get user's timezone
export const getUserTimezone = (): string => {
  return Intl.DateTimeFormat().resolvedOptions().timeZone;
};

// Utility function to get locale-specific date format patterns
export const getDateFormatPatterns = (language: SupportedLanguage) => {
  const locale = NUMBER_LOCALES[language] || NUMBER_LOCALES.en;
  const formatter = new Intl.DateTimeFormat(locale);
  const parts = formatter.formatToParts(new Date());
  
  return {
    datePattern: parts.map(part => 
      part.type === 'literal' ? part.value : `{${part.type}}`
    ).join(''),
    locale,
  };
};