import React, { useState, useMemo } from 'react';
import { Search, Filter, Star, Download, Tag, ExternalLink } from 'lucide-react';
import { useModules, useModuleSearch } from '../hooks/useModules';
import { ModuleSearchFilters, ModuleCategory, PricingModel, Platform } from '../types/module';
import { 
  formatPrice, 
  getCategoryDisplayName, 
  getInstallationStatusColor,
  getInstallationStatusDisplayName,
  getPlatformIcon,
  getSecurityScoreColor,
  getSecurityScoreLabel
} from '../utils/moduleUtils';
import { ModuleCard } from './ModuleCard';
import { ModuleFilters } from './ModuleFilters';

export const ModuleMarketplace: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [filters, setFilters] = useState<ModuleSearchFilters>({});
  const [showFilters, setShowFilters] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedTab, setSelectedTab] = useState<'all' | 'featured' | 'trending' | 'recommended'>('all');

  const pageSize = 12;

  const { 
    featuredModules, 
    trendingModules, 
    recommendedModules,
    installModule,
    installedModules 
  } = useModules();

  const searchResults = useModuleSearch(
    searchQuery || undefined,
    filters,
    currentPage,
    pageSize
  );

  const currentModules = useMemo(() => {
    switch (selectedTab) {
      case 'featured':
        return { data: { modules: featuredModules.data || [], total: featuredModules.data?.length || 0 } };
      case 'trending':
        return { data: { modules: trendingModules.data || [], total: trendingModules.data?.length || 0 } };
      case 'recommended':
        return { data: { modules: recommendedModules.data || [], total: recommendedModules.data?.length || 0 } };
      default:
        return searchResults;
    }
  }, [selectedTab, searchResults, featuredModules, trendingModules, recommendedModules]);

  const installedModuleIds = useMemo(() => 
    new Set(installedModules.data?.map(m => m.id) || []),
    [installedModules.data]
  );

  const handleInstallModule = async (moduleId: string, version?: string) => {
    try {
      await installModule.mutateAsync({
        moduleId,
        version,
        tenantId: '', // Will be set by the BFF client
      });
    } catch (error) {
      console.error('Failed to install module:', error);
    }
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    setCurrentPage(1);
    setSelectedTab('all');
  };

  const handleFilterChange = (newFilters: ModuleSearchFilters) => {
    setFilters(newFilters);
    setCurrentPage(1);
    setSelectedTab('all');
  };

  const totalPages = Math.ceil((currentModules.data?.total || 0) / pageSize);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Module Marketplace</h1>
          <p className="text-gray-600">Discover and install modules to extend ADX Core functionality</p>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <form onSubmit={handleSearch} className="flex gap-4 mb-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 w-5 h-5" />
            <input
              type="text"
              placeholder="Search modules..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          <button
            type="button"
            onClick={() => setShowFilters(!showFilters)}
            className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
          >
            <Filter className="w-4 h-4" />
            Filters
          </button>
          <button
            type="submit"
            className="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
          >
            Search
          </button>
        </form>

        {showFilters && (
          <ModuleFilters
            filters={filters}
            onFiltersChange={handleFilterChange}
            onClose={() => setShowFilters(false)}
          />
        )}
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {[
            { key: 'all', label: 'All Modules' },
            { key: 'featured', label: 'Featured' },
            { key: 'trending', label: 'Trending' },
            { key: 'recommended', label: 'Recommended' },
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => {
                setSelectedTab(tab.key as any);
                setCurrentPage(1);
              }}
              className={`py-2 px-1 border-b-2 font-medium text-sm ${
                selectedTab === tab.key
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Loading State */}
      {currentModules.isLoading && (
        <div className="flex items-center justify-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        </div>
      )}

      {/* Error State */}
      {currentModules.error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <p className="text-red-800">
            Failed to load modules: {(currentModules.error as Error).message}
          </p>
        </div>
      )}

      {/* Results */}
      {currentModules.data && (
        <>
          <div className="flex items-center justify-between">
            <p className="text-gray-600">
              {currentModules.data.total} modules found
            </p>
            {selectedTab === 'all' && totalPages > 1 && (
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
                  disabled={currentPage === 1}
                  className="px-3 py-1 border border-gray-300 rounded disabled:opacity-50"
                >
                  Previous
                </button>
                <span className="text-sm text-gray-600">
                  Page {currentPage} of {totalPages}
                </span>
                <button
                  onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
                  disabled={currentPage === totalPages}
                  className="px-3 py-1 border border-gray-300 rounded disabled:opacity-50"
                >
                  Next
                </button>
              </div>
            )}
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
            {currentModules.data.modules.map((module) => (
              <ModuleCard
                key={module.id}
                module={module}
                isInstalled={installedModuleIds.has(module.id)}
                onInstall={() => handleInstallModule(module.id, module.version)}
                isInstalling={installModule.isPending}
              />
            ))}
          </div>

          {currentModules.data.modules.length === 0 && (
            <div className="text-center py-12">
              <p className="text-gray-500">No modules found matching your criteria.</p>
            </div>
          )}
        </>
      )}
    </div>
  );
};