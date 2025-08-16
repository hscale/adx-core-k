import React, { useState, useEffect } from 'react';
import { useTranslation, SUPPORTED_LANGUAGES, NAMESPACES } from '@adx-core/i18n';
import type { SupportedLanguage, Namespace } from '@adx-core/i18n';
import { TranslationEditor } from './TranslationEditor';
import { translationApi, TranslationStatus } from '../../api/translations';



export const LanguageManagement: React.FC = () => {
  const { t, currentLanguage, changeLanguage } = useTranslation('shell');
  const [selectedLanguage, setSelectedLanguage] = useState<SupportedLanguage>(currentLanguage);
  const [selectedNamespace, setSelectedNamespace] = useState<Namespace>('common');
  const [translationStatuses, setTranslationStatuses] = useState<TranslationStatus[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'editor' | 'status' | 'bulk'>('editor');

  // Load translation statuses
  useEffect(() => {
    const loadTranslationStatuses = async () => {
      setIsLoading(true);
      try {
        const data = await translationApi.getTranslationStatus();
        setTranslationStatuses(data);
      } catch (error) {
        console.error('Failed to load translation statuses:', error);
      } finally {
        setIsLoading(false);
      }
    };

    loadTranslationStatuses();
  }, []);

  const handleSaveTranslations = async (translations: Record<string, string>) => {
    await translationApi.saveTranslations(selectedLanguage, selectedNamespace, translations);

    // Refresh translation statuses
    const data = await translationApi.getTranslationStatus();
    setTranslationStatuses(data);
  };

  const handleExportTranslations = async (language?: SupportedLanguage) => {
    try {
      const blob = await translationApi.exportTranslations(language);
      const downloadUrl = window.URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = downloadUrl;
      link.download = language 
        ? `translations-${language}.json`
        : 'translations-all.json';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      window.URL.revokeObjectURL(downloadUrl);
    } catch (error) {
      console.error('Failed to export translations:', error);
    }
  };

  const handleImportTranslations = async (file: File) => {
    try {
      await translationApi.importTranslations(file);
      
      // Refresh translation statuses
      const data = await translationApi.getTranslationStatus();
      setTranslationStatuses(data);
    } catch (error) {
      console.error('Failed to import translations:', error);
    }
  };

  const getStatusColor = (percentage: number) => {
    if (percentage >= 90) return 'bg-green-500';
    if (percentage >= 70) return 'bg-yellow-500';
    return 'bg-red-500';
  };

  return (
    <div className="language-management">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">
          {t('languageManagement.title')}
        </h1>
        <p className="mt-2 text-gray-600">
          {t('languageManagement.description')}
        </p>
      </div>

      {/* Language and Namespace Selectors */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6 mb-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {t('languageManagement.currentLanguage')}
            </label>
            <select
              value={selectedLanguage}
              onChange={(e) => setSelectedLanguage(e.target.value as SupportedLanguage)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {Object.entries(SUPPORTED_LANGUAGES).map(([code, language]) => (
                <option key={code} value={code}>
                  {language.flag} {language.nativeName} ({language.name})
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              {t('languageManagement.translationEditor.namespace')}
            </label>
            <select
              value={selectedNamespace}
              onChange={(e) => setSelectedNamespace(e.target.value as Namespace)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {Object.values(NAMESPACES).map((namespace) => (
                <option key={namespace} value={namespace}>
                  {namespace}
                </option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="-mb-px flex space-x-8">
          <button
            onClick={() => setActiveTab('editor')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'editor'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            {t('languageManagement.translationEditor.title')}
          </button>
          <button
            onClick={() => setActiveTab('status')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'status'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            {t('languageManagement.translationStatus')}
          </button>
          <button
            onClick={() => setActiveTab('bulk')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${
              activeTab === 'bulk'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
            }`}
          >
            {t('languageManagement.bulkOperations.title')}
          </button>
        </nav>
      </div>

      {/* Tab Content */}
      {activeTab === 'editor' && (
        <TranslationEditor
          selectedLanguage={selectedLanguage}
          selectedNamespace={selectedNamespace}
          onSave={handleSaveTranslations}
        />
      )}

      {activeTab === 'status' && (
        <div className="space-y-6">
          {Object.entries(SUPPORTED_LANGUAGES).map(([langCode, language]) => (
            <div key={langCode} className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-medium text-gray-900">
                  {language.flag} {language.nativeName}
                </h3>
                <button
                  onClick={() => handleExportTranslations(langCode as SupportedLanguage)}
                  className="px-3 py-1 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
                >
                  {t('languageManagement.exportTranslations')}
                </button>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                {Object.values(NAMESPACES).map((namespace) => {
                  const status = translationStatuses.find(
                    s => s.language === langCode && s.namespace === namespace
                  );
                  const percentage = status?.completionPercentage || 0;

                  return (
                    <div key={namespace} className="border border-gray-200 rounded-lg p-4">
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm font-medium text-gray-700">
                          {namespace}
                        </span>
                        <span className="text-sm text-gray-500">
                          {percentage.toFixed(0)}%
                        </span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className={`h-2 rounded-full ${getStatusColor(percentage)}`}
                          style={{ width: `${percentage}%` }}
                        />
                      </div>
                      {status && (
                        <div className="mt-2 text-xs text-gray-500">
                          {status.translatedKeys} / {status.totalKeys} keys
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            </div>
          ))}
        </div>
      )}

      {activeTab === 'bulk' && (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          <h3 className="text-lg font-medium text-gray-900 mb-6">
            {t('languageManagement.bulkOperations.title')}
          </h3>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <h4 className="font-medium text-gray-900">Export Operations</h4>
              <button
                onClick={() => handleExportTranslations()}
                className="w-full px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
              >
                {t('languageManagement.bulkOperations.exportAll')}
              </button>
            </div>

            <div className="space-y-4">
              <h4 className="font-medium text-gray-900">Import Operations</h4>
              <div className="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center">
                <input
                  type="file"
                  accept=".json"
                  onChange={(e) => {
                    const file = e.target.files?.[0];
                    if (file) {
                      handleImportTranslations(file);
                    }
                  }}
                  className="hidden"
                  id="import-file"
                />
                <label
                  htmlFor="import-file"
                  className="cursor-pointer text-blue-600 hover:text-blue-700"
                >
                  {t('languageManagement.bulkOperations.importAll')}
                </label>
                <p className="mt-2 text-sm text-gray-500">
                  Select a JSON file to import translations
                </p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};