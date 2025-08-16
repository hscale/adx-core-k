import React, { createContext, useContext, useEffect, useState } from 'react';
import { I18nextProvider } from 'react-i18next';
import { initializeI18n, SupportedLanguage, SUPPORTED_LANGUAGES, Namespace } from './config';
import { useTranslation, useRTL } from './hooks';

// Language context for micro-frontend communication
interface LanguageContextType {
  currentLanguage: SupportedLanguage;
  changeLanguage: (language: SupportedLanguage) => Promise<void>;
  isRTL: boolean;
  supportedLanguages: typeof SUPPORTED_LANGUAGES;
}

const LanguageContext = createContext<LanguageContextType | null>(null);

// I18n Provider component
interface I18nProviderProps {
  children: React.ReactNode;
  defaultLanguage?: SupportedLanguage;
  namespaces?: Namespace[];
}

export const I18nProvider: React.FC<I18nProviderProps> = ({
  children,
  defaultLanguage = 'en',
  namespaces = ['common'],
}) => {
  const [i18nInstance] = useState(() => {
    return initializeI18n({
      fallbackLng: defaultLanguage,
      ns: namespaces,
      defaultNS: namespaces[0],
    });
  });

  return (
    <I18nextProvider i18n={i18nInstance}>
      <LanguageContextProvider>
        <RTLStyleProvider>
          {children}
        </RTLStyleProvider>
      </LanguageContextProvider>
    </I18nextProvider>
  );
};

// Language context provider
const LanguageContextProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { changeLanguage, currentLanguage, isRTL, supportedLanguages } = useTranslation();

  const contextValue: LanguageContextType = {
    currentLanguage,
    changeLanguage,
    isRTL,
    supportedLanguages,
  };

  return (
    <LanguageContext.Provider value={contextValue}>
      {children}
    </LanguageContext.Provider>
  );
};

// RTL style provider
const RTLStyleProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { isRTL, direction } = useRTL();

  useEffect(() => {
    // Add RTL class to body for global styling
    document.body.classList.toggle('rtl', isRTL);
    document.body.classList.toggle('ltr', !isRTL);
    
    // Set CSS custom properties
    document.documentElement.style.setProperty('--text-direction', direction);
    document.documentElement.style.setProperty('--start', isRTL ? 'right' : 'left');
    document.documentElement.style.setProperty('--end', isRTL ? 'left' : 'right');
  }, [isRTL, direction]);

  return <>{children}</>;
};

// Language selector component
interface LanguageSelectorProps {
  className?: string;
  showFlags?: boolean;
  showNativeNames?: boolean;
}

export const LanguageSelector: React.FC<LanguageSelectorProps> = ({
  className = '',
  showFlags = true,
  showNativeNames = true,
}) => {
  const context = useContext(LanguageContext);
  
  if (!context) {
    throw new Error('LanguageSelector must be used within I18nProvider');
  }

  const { currentLanguage, changeLanguage, supportedLanguages } = context;
  const { t } = useTranslation('common');

  const handleLanguageChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
    const newLanguage = event.target.value as SupportedLanguage;
    changeLanguage(newLanguage);
  };

  return (
    <div className={`language-selector ${className}`}>
      <label htmlFor="language-select" className="sr-only">
        {t('selectLanguage')}
      </label>
      <select
        id="language-select"
        value={currentLanguage}
        onChange={handleLanguageChange}
        className="language-select"
      >
        {Object.entries(supportedLanguages).map(([code, language]) => (
          <option key={code} value={code}>
            {showFlags && language.flag} {' '}
            {showNativeNames ? language.nativeName : language.name}
          </option>
        ))}
      </select>
    </div>
  );
};

// Translation component for complex interpolations
interface TranslationProps {
  i18nKey: string;
  namespace?: string;
  values?: Record<string, any>;
  components?: Record<string, React.ReactElement>;
  className?: string;
}

export const Translation: React.FC<TranslationProps> = ({
  i18nKey,
  namespace,
  values,
  components,
  className,
}) => {
  const { t } = useTranslation(namespace as any);

  return (
    <span className={className}>
      {t(i18nKey, { ...values, ...components }) as React.ReactNode}
    </span>
  );
};

// Hook to use language context
export const useLanguageContext = () => {
  const context = useContext(LanguageContext);
  
  if (!context) {
    throw new Error('useLanguageContext must be used within I18nProvider');
  }
  
  return context;
};