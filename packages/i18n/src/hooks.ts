import { useTranslation as useI18nTranslation } from 'react-i18next';
import { useCallback, useEffect, useState } from 'react';
import { SUPPORTED_LANGUAGES, SupportedLanguage, Namespace } from './config';
import { formatters } from './formatters';

// Enhanced useTranslation hook with namespace support
export const useTranslation = (namespace?: Namespace) => {
  const { t, i18n, ready } = useI18nTranslation(namespace);
  
  const changeLanguage = useCallback(async (language: SupportedLanguage) => {
    await i18n.changeLanguage(language);
    
    // Update document direction for RTL languages
    const isRTL = SUPPORTED_LANGUAGES[language]?.rtl || false;
    document.documentElement.dir = isRTL ? 'rtl' : 'ltr';
    document.documentElement.lang = language;
    
    // Emit custom event for micro-frontends to react to language changes
    window.dispatchEvent(new CustomEvent('adx-language-changed', {
      detail: { language, isRTL }
    }));
  }, [i18n]);
  
  return {
    t,
    i18n,
    ready,
    changeLanguage,
    currentLanguage: i18n.language as SupportedLanguage,
    isRTL: SUPPORTED_LANGUAGES[i18n.language as SupportedLanguage]?.rtl || false,
    supportedLanguages: SUPPORTED_LANGUAGES,
  };
};

// Hook for language management
export const useLanguageManager = () => {
  const { i18n } = useI18nTranslation();
  const [isLoading, setIsLoading] = useState(false);
  
  const loadNamespace = useCallback(async (namespace: Namespace) => {
    setIsLoading(true);
    try {
      await i18n.loadNamespaces(namespace);
    } finally {
      setIsLoading(false);
    }
  }, [i18n]);
  
  const reloadResources = useCallback(async (language?: SupportedLanguage) => {
    setIsLoading(true);
    try {
      await i18n.reloadResources(language);
    } finally {
      setIsLoading(false);
    }
  }, [i18n]);
  
  const addResourceBundle = useCallback((
    language: SupportedLanguage,
    namespace: Namespace,
    resources: Record<string, any>
  ) => {
    i18n.addResourceBundle(language, namespace, resources, true, true);
  }, [i18n]);
  
  return {
    loadNamespace,
    reloadResources,
    addResourceBundle,
    isLoading,
    currentLanguage: i18n.language as SupportedLanguage,
    availableLanguages: Object.keys(SUPPORTED_LANGUAGES) as SupportedLanguage[],
  };
};

// Hook for locale-specific formatting
export const useLocaleFormatting = () => {
  const { i18n } = useI18nTranslation();
  const currentLanguage = i18n.language as SupportedLanguage;
  
  return {
    formatDate: formatters.formatDate(currentLanguage),
    formatTime: formatters.formatTime(currentLanguage),
    formatDateTime: formatters.formatDateTime(currentLanguage),
    formatNumber: formatters.formatNumber(currentLanguage),
    formatCurrency: formatters.formatCurrency(currentLanguage),
    formatPercent: formatters.formatPercent(currentLanguage),
    formatRelativeTime: formatters.formatRelativeTime(currentLanguage),
    currentLanguage,
  };
};

// Hook for RTL support
export const useRTL = () => {
  const { i18n } = useI18nTranslation();
  const [isRTL, setIsRTL] = useState(false);
  
  useEffect(() => {
    const updateRTL = () => {
      const currentLang = i18n.language as SupportedLanguage;
      const rtl = SUPPORTED_LANGUAGES[currentLang]?.rtl || false;
      setIsRTL(rtl);
      
      // Update CSS custom property for RTL
      document.documentElement.style.setProperty('--text-direction', rtl ? 'rtl' : 'ltr');
      document.documentElement.style.setProperty('--start', rtl ? 'right' : 'left');
      document.documentElement.style.setProperty('--end', rtl ? 'left' : 'right');
    };
    
    updateRTL();
    i18n.on('languageChanged', updateRTL);
    
    return () => {
      i18n.off('languageChanged', updateRTL);
    };
  }, [i18n]);
  
  return {
    isRTL,
    direction: isRTL ? 'rtl' : 'ltr',
    textAlign: isRTL ? 'right' : 'left',
    marginStart: isRTL ? 'marginRight' : 'marginLeft',
    marginEnd: isRTL ? 'marginLeft' : 'marginRight',
    paddingStart: isRTL ? 'paddingRight' : 'paddingLeft',
    paddingEnd: isRTL ? 'paddingLeft' : 'paddingRight',
  };
};