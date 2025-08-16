import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { I18nProvider, useTranslation, LanguageSelector } from '../index';

// Mock component to test useTranslation hook
const TestComponent: React.FC = () => {
  const { t, currentLanguage } = useTranslation('common');
  
  return (
    <div>
      <span data-testid="language">{currentLanguage}</span>
      <span data-testid="loading">{t('loading')}</span>
      <span data-testid="save">{t('save')}</span>
    </div>
  );
};

// Mock component to test LanguageSelector
const TestLanguageSelector: React.FC = () => {
  return (
    <I18nProvider defaultLanguage="en" namespaces={['common']}>
      <LanguageSelector data-testid="language-selector" />
    </I18nProvider>
  );
};

describe('I18n System', () => {
  it('should provide translation function', () => {
    render(
      <I18nProvider defaultLanguage="en" namespaces={['common']}>
        <TestComponent />
      </I18nProvider>
    );

    expect(screen.getByTestId('language')).toHaveTextContent('en');
    expect(screen.getByTestId('loading')).toBeInTheDocument();
    expect(screen.getByTestId('save')).toBeInTheDocument();
  });

  it('should render language selector', () => {
    render(<TestLanguageSelector />);
    
    const selector = screen.getByTestId('language-selector');
    expect(selector).toBeInTheDocument();
  });

  it('should support multiple namespaces', () => {
    const MultiNamespaceComponent: React.FC = () => {
      const { t: tCommon } = useTranslation('common');
      const { t: tAuth } = useTranslation('auth');
      
      return (
        <div>
          <span data-testid="common-loading">{tCommon('loading')}</span>
          <span data-testid="auth-title">{tAuth('title')}</span>
        </div>
      );
    };

    render(
      <I18nProvider defaultLanguage="en" namespaces={['common', 'auth']}>
        <MultiNamespaceComponent />
      </I18nProvider>
    );

    expect(screen.getByTestId('common-loading')).toBeInTheDocument();
    expect(screen.getByTestId('auth-title')).toBeInTheDocument();
  });
});