import type { SupportedLanguage, Namespace } from '@adx-core/i18n';

export interface TranslationStatus {
  language: SupportedLanguage;
  namespace: Namespace;
  totalKeys: number;
  translatedKeys: number;
  completionPercentage: number;
}

export interface TranslationKey {
  key: string;
  value: string;
  namespace: Namespace;
  language: SupportedLanguage;
}

// Mock API functions - in real implementation these would call backend services
export const translationApi = {
  // Get translations for a specific language and namespace
  async getTranslations(language: SupportedLanguage, namespace: Namespace): Promise<Record<string, string>> {
    try {
      const response = await fetch(`/locales/${language}/${namespace}.json`);
      if (response.ok) {
        return await response.json();
      }
      return {};
    } catch (error) {
      console.error('Failed to load translations:', error);
      return {};
    }
  },

  // Save translations for a specific language and namespace
  async saveTranslations(
    language: SupportedLanguage, 
    namespace: Namespace, 
    translations: Record<string, string>
  ): Promise<void> {
    // In a real implementation, this would save to backend
    // For now, we'll simulate the API call
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Store in localStorage for demo purposes
    const key = `translations_${language}_${namespace}`;
    localStorage.setItem(key, JSON.stringify(translations));
  },

  // Get translation status for all languages and namespaces
  async getTranslationStatus(): Promise<TranslationStatus[]> {
    const languages: SupportedLanguage[] = ['en', 'es', 'fr', 'de', 'ja', 'zh'];
    const namespaces: Namespace[] = ['common', 'auth', 'tenant', 'file', 'user', 'workflow', 'module', 'shell'];
    const statuses: TranslationStatus[] = [];

    for (const language of languages) {
      for (const namespace of namespaces) {
        try {
          const translations = await this.getTranslations(language, namespace);
          const totalKeys = Object.keys(translations).length;
          const translatedKeys = Object.values(translations).filter(value => value && value.trim() !== '').length;
          const completionPercentage = totalKeys > 0 ? (translatedKeys / totalKeys) * 100 : 0;

          statuses.push({
            language,
            namespace,
            totalKeys,
            translatedKeys,
            completionPercentage,
          });
        } catch (error) {
          // If translation file doesn't exist, mark as 0% complete
          statuses.push({
            language,
            namespace,
            totalKeys: 0,
            translatedKeys: 0,
            completionPercentage: 0,
          });
        }
      }
    }

    return statuses;
  },

  // Export translations for a specific language or all languages
  async exportTranslations(language?: SupportedLanguage): Promise<Blob> {
    const languages: SupportedLanguage[] = language ? [language] : ['en', 'es', 'fr', 'de', 'ja', 'zh'];
    const namespaces: Namespace[] = ['common', 'auth', 'tenant', 'file', 'user', 'workflow', 'module', 'shell'];
    const exportData: Record<string, Record<string, Record<string, string>>> = {};

    for (const lang of languages) {
      exportData[lang] = {};
      for (const namespace of namespaces) {
        const translations = await this.getTranslations(lang, namespace);
        if (Object.keys(translations).length > 0) {
          exportData[lang][namespace] = translations;
        }
      }
    }

    const jsonString = JSON.stringify(exportData, null, 2);
    return new Blob([jsonString], { type: 'application/json' });
  },

  // Import translations from a JSON file
  async importTranslations(file: File): Promise<void> {
    const text = await file.text();
    const importData = JSON.parse(text);

    // Validate and save imported translations
    for (const [language, namespaces] of Object.entries(importData)) {
      if (typeof namespaces === 'object' && namespaces !== null) {
        for (const [namespace, translations] of Object.entries(namespaces)) {
          if (typeof translations === 'object' && translations !== null) {
            await this.saveTranslations(
              language as SupportedLanguage,
              namespace as Namespace,
              translations as Record<string, string>
            );
          }
        }
      }
    }
  },

  // Validate translations for missing keys
  async validateTranslations(): Promise<{
    language: SupportedLanguage;
    namespace: Namespace;
    missingKeys: string[];
  }[]> {
    const languages: SupportedLanguage[] = ['en', 'es', 'fr', 'de', 'ja', 'zh'];
    const namespaces: Namespace[] = ['common', 'auth', 'tenant', 'file', 'user', 'workflow', 'module', 'shell'];
    const validationResults = [];

    // Use English as the reference language
    const referenceTranslations: Record<Namespace, Record<string, string>> = {};
    for (const namespace of namespaces) {
      referenceTranslations[namespace] = await this.getTranslations('en', namespace);
    }

    for (const language of languages) {
      if (language === 'en') continue; // Skip reference language

      for (const namespace of namespaces) {
        const translations = await this.getTranslations(language, namespace);
        const referenceKeys = Object.keys(referenceTranslations[namespace]);
        const translationKeys = Object.keys(translations);
        const missingKeys = referenceKeys.filter(key => !translationKeys.includes(key));

        if (missingKeys.length > 0) {
          validationResults.push({
            language,
            namespace,
            missingKeys,
          });
        }
      }
    }

    return validationResults;
  },

  // Auto-translate missing keys (mock implementation)
  async autoTranslateMissingKeys(
    targetLanguage: SupportedLanguage,
    namespace: Namespace,
    missingKeys: string[]
  ): Promise<Record<string, string>> {
    // In a real implementation, this would use a translation service like Google Translate
    // For now, we'll return placeholder translations
    const autoTranslations: Record<string, string> = {};
    
    for (const key of missingKeys) {
      autoTranslations[key] = `[AUTO] ${key} (${targetLanguage})`;
    }

    return autoTranslations;
  },
};