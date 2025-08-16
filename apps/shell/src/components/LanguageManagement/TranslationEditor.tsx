import React, { useState, useEffect } from 'react';
import { useTranslation, useLanguageManager } from '@adx-core/i18n';
import type { SupportedLanguage, Namespace } from '@adx-core/i18n';
import { translationApi } from '../../api/translations';

interface TranslationKey {
  key: string;
  value: string;
  namespace: Namespace;
  language: SupportedLanguage;
}

interface TranslationEditorProps {
  selectedLanguage: SupportedLanguage;
  selectedNamespace: Namespace;
  onSave: (translations: Record<string, string>) => Promise<void>;
}

export const TranslationEditor: React.FC<TranslationEditorProps> = ({
  selectedLanguage,
  selectedNamespace,
  onSave,
}) => {
  const { t } = useTranslation('shell');
  const { addResourceBundle } = useLanguageManager();
  const [translations, setTranslations] = useState<Record<string, string>>({});
  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [hasChanges, setHasChanges] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');

  // Load translations for selected language and namespace
  useEffect(() => {
    const loadTranslations = async () => {
      setIsLoading(true);
      try {
        const data = await translationApi.getTranslations(selectedLanguage, selectedNamespace);
        setTranslations(data);
      } catch (error) {
        console.error('Failed to load translations:', error);
      } finally {
        setIsLoading(false);
      }
    };

    loadTranslations();
  }, [selectedLanguage, selectedNamespace]);

  const handleTranslationChange = (key: string, value: string) => {
    setTranslations(prev => ({
      ...prev,
      [key]: value,
    }));
    setHasChanges(true);
  };

  const handleAddTranslation = () => {
    if (newKey && newValue) {
      setTranslations(prev => ({
        ...prev,
        [newKey]: newValue,
      }));
      setNewKey('');
      setNewValue('');
      setHasChanges(true);
    }
  };

  const handleDeleteTranslation = (key: string) => {
    setTranslations(prev => {
      const updated = { ...prev };
      delete updated[key];
      return updated;
    });
    setHasChanges(true);
  };

  const handleSave = async () => {
    setIsLoading(true);
    try {
      await translationApi.saveTranslations(selectedLanguage, selectedNamespace, translations);
      
      // Update i18n instance with new translations
      addResourceBundle(selectedLanguage, selectedNamespace, translations);
      
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to save translations:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleDiscard = () => {
    // Reload translations from server
    window.location.reload();
  };

  const filteredTranslations = Object.entries(translations).filter(([key, value]) =>
    key.toLowerCase().includes(searchTerm.toLowerCase()) ||
    value.toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (isLoading && Object.keys(translations).length === 0) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
          <p className="mt-2 text-gray-600">{t('loading')}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="translation-editor">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-2xl font-bold text-gray-900">
            {t('languageManagement.translationEditor.title')}
          </h2>
          <p className="text-gray-600">
            {selectedLanguage.toUpperCase()} - {selectedNamespace}
          </p>
        </div>
        
        <div className="flex space-x-3">
          {hasChanges && (
            <button
              onClick={handleDiscard}
              className="px-4 py-2 text-gray-700 bg-gray-200 rounded-md hover:bg-gray-300 transition-colors"
            >
              {t('languageManagement.translationEditor.discardChanges')}
            </button>
          )}
          <button
            onClick={handleSave}
            disabled={!hasChanges || isLoading}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {isLoading ? t('loading') : t('languageManagement.translationEditor.saveChanges')}
          </button>
        </div>
      </div>

      {/* Search */}
      <div className="mb-4">
        <input
          type="text"
          placeholder={t('search')}
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
      </div>

      {/* Add New Translation */}
      <div className="bg-gray-50 p-4 rounded-lg mb-6">
        <h3 className="text-lg font-medium mb-3">
          {t('languageManagement.translationEditor.addKey')}
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
          <input
            type="text"
            placeholder={t('languageManagement.translationEditor.key')}
            value={newKey}
            onChange={(e) => setNewKey(e.target.value)}
            className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <input
            type="text"
            placeholder={t('languageManagement.translationEditor.value')}
            value={newValue}
            onChange={(e) => setNewValue(e.target.value)}
            className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <button
            onClick={handleAddTranslation}
            disabled={!newKey || !newValue}
            className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {t('create')}
          </button>
        </div>
      </div>

      {/* Translation List */}
      <div className="space-y-2">
        {filteredTranslations.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            {searchTerm ? t('search.noResults') : 'No translations found'}
          </div>
        ) : (
          filteredTranslations.map(([key, value]) => (
            <div key={key} className="bg-white border border-gray-200 rounded-lg p-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    {t('languageManagement.translationEditor.key')}
                  </label>
                  <input
                    type="text"
                    value={key}
                    readOnly
                    className="w-full px-3 py-2 bg-gray-50 border border-gray-300 rounded-md text-gray-600"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    {t('languageManagement.translationEditor.value')}
                  </label>
                  <div className="flex space-x-2">
                    <textarea
                      value={value}
                      onChange={(e) => handleTranslationChange(key, e.target.value)}
                      rows={2}
                      className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                    />
                    <button
                      onClick={() => handleDeleteTranslation(key)}
                      className="px-3 py-2 text-red-600 hover:bg-red-50 rounded-md transition-colors"
                      title={t('delete')}
                    >
                      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>
                </div>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Status Bar */}
      {hasChanges && (
        <div className="fixed bottom-4 right-4 bg-yellow-100 border border-yellow-400 text-yellow-800 px-4 py-2 rounded-md shadow-lg">
          <div className="flex items-center space-x-2">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
            </svg>
            <span>Unsaved changes</span>
          </div>
        </div>
      )}
    </div>
  );
};