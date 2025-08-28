import React, { useState, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { Search, Filter, Star, Download, Eye, Package, Zap, Shield, Trending } from 'lucide-react';
import { useBFFClient } from '@adx-core/bff-client';
import { useEventBus } from '@adx-core/event-bus';
import LoadingSpinner from './LoadingSpinner';

interface Module {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  category: string;
  tags: string[];
  rating: number;
  downloads: number;
  price: number;
  currency: string;
  screenshots: string[];
  features: string[];
  compatibility: string[];
  lastUpdated: string;
  verified: boolean;
  trending: boolean;
}

interface MarketplaceFilters {
  category: string;
  priceRange: 'free' | 'paid' | 'all';
  rating: number;
  verified: boolean;
  trending: boolean;
}

const ModuleMarketplace: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [filters, setFilters] = useState<MarketplaceFilters>({
    category: 'all',
    priceRange: 'all',
    rating: 0,
    verified: false,
    trending: false,
  });
  const [selectedModule, setSelectedModule] = useState<Module | null>(null);
  const [showFilters, setShowFilters] = useState(false);

  const bffClient = useBFFClient();
  const { emit } = useEventBus();

  const { data: modules, isLoading, error } = useQuery({
    queryKey: ['modules', 'marketplace', searchQuery, filters],
    queryFn: async () => {
      const params = new URLSearchParams({
        search: searchQuery,
        category: filters.category,
        priceRange: filters.priceRange,
        minRating: filters.rating.toString(),
        verified: filters.verified.toString(),
        trending: filters.trending.toString(),
      });
      
      return bffClient.get(`/modules/marketplace?${params}`);
    },
  });

  const { data: categories } = useQuery({
    queryKey: ['modules', 'categories'],
    queryFn: () => bffClient.get('/modules/categories'),
  });

  const handleInstallModule = async (moduleId: string) => {
    try {
      emit('module:install-start', { moduleId });
      
      const result = await bffClient.post('/modules/install', { moduleId });
      
      if (result.workflowId) {
        emit('module:install-workflow', { 
          moduleId, 
          workflowId: result.workflowId 
        });
      }
      
      emit('module:install-success', { moduleId });
    } catch (error) {
      emit('module:install-error', { moduleId, error });
    }
  };

  const handleViewModule = (module: Module) => {
    setSelectedModule(module);
    emit('module:view', { moduleId: module.id });
  };

  if (isLoading) {
    return <LoadingSpinner text="Loading marketplace..." />;
  }

  if (error) {
    return (
      <div className="p-6 text-center">
        <p className="text-red-600 dark:text-red-400">
          Failed to load marketplace. Please try again.
        </p>
      </div>
    );
  }

  return (
    <div className="module-marketplace p-6">
      {/* Header */}
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
          Module Marketplace
        </h1>
        <p className="text-gray-600 dark:text-gray-300">
          Discover and install modules to extend your ADX Core experience
        </p>
      </div>

      {/* Search and Filters */}
      <div className="mb-6 space-y-4">
        <div className="flex gap-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search modules..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          <button
            onClick={() => setShowFilters(!showFilters)}
            className="flex items-center px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
          >
            <Filter className="w-4 h-4 mr-2" />
            Filters
          </button>
        </div>

        {/* Filter Panel */}
        {showFilters && (
          <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Category
                </label>
                <select
                  value={filters.category}
                  onChange={(e) => setFilters({ ...filters, category: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                >
                  <option value="all">All Categories</option>
                  {categories?.map((category: string) => (
                    <option key={category} value={category}>
                      {category}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Price
                </label>
                <select
                  value={filters.priceRange}
                  onChange={(e) => setFilters({ ...filters, priceRange: e.target.value as any })}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                >
                  <option value="all">All Prices</option>
                  <option value="free">Free</option>
                  <option value="paid">Paid</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Min Rating
                </label>
                <select
                  value={filters.rating}
                  onChange={(e) => setFilters({ ...filters, rating: Number(e.target.value) })}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-gray-900 dark:text-white"
                >
                  <option value={0}>Any Rating</option>
                  <option value={3}>3+ Stars</option>
                  <option value={4}>4+ Stars</option>
                  <option value={4.5}>4.5+ Stars</option>
                </select>
              </div>

              <div className="space-y-2">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={filters.verified}
                    onChange={(e) => setFilters({ ...filters, verified: e.target.checked })}
                    className="mr-2"
                  />
                  <span className="text-sm text-gray-700 dark:text-gray-300">Verified Only</span>
                </label>
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={filters.trending}
                    onChange={(e) => setFilters({ ...filters, trending: e.target.checked })}
                    className="mr-2"
                  />
                  <span className="text-sm text-gray-700 dark:text-gray-300">Trending</span>
                </label>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Module Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {modules?.map((module: Module) => (
          <div
            key={module.id}
            className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-6 hover:shadow-lg transition-shadow cursor-pointer"
            onClick={() => handleViewModule(module)}
          >
            <div className="flex items-start justify-between mb-3">
              <div className="flex items-center">
                <Package className="w-8 h-8 text-blue-600 dark:text-blue-400 mr-3" />
                <div>
                  <h3 className="font-semibold text-gray-900 dark:text-white">
                    {module.name}
                  </h3>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    by {module.author}
                  </p>
                </div>
              </div>
              <div className="flex items-center space-x-1">
                {module.verified && (
                  <Shield className="w-4 h-4 text-green-500" title="Verified" />
                )}
                {module.trending && (
                  <Trending className="w-4 h-4 text-orange-500" title="Trending" />
                )}
              </div>
            </div>

            <p className="text-gray-600 dark:text-gray-300 text-sm mb-4 line-clamp-2">
              {module.description}
            </p>

            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center">
                <Star className="w-4 h-4 text-yellow-400 mr-1" />
                <span className="text-sm text-gray-600 dark:text-gray-300">
                  {module.rating.toFixed(1)}
                </span>
              </div>
              <div className="flex items-center text-sm text-gray-500 dark:text-gray-400">
                <Download className="w-4 h-4 mr-1" />
                {module.downloads.toLocaleString()}
              </div>
            </div>

            <div className="flex items-center justify-between">
              <div className="text-lg font-semibold text-gray-900 dark:text-white">
                {module.price === 0 ? 'Free' : `$${module.price}`}
              </div>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleInstallModule(module.id);
                }}
                className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md text-sm transition-colors"
              >
                Install
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Module Detail Modal */}
      {selectedModule && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
          <div className="bg-white dark:bg-gray-800 rounded-lg max-w-4xl w-full max-h-[90vh] overflow-y-auto">
            <div className="p-6">
              <div className="flex items-start justify-between mb-4">
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
                    {selectedModule.name}
                  </h2>
                  <p className="text-gray-600 dark:text-gray-300">
                    by {selectedModule.author} • v{selectedModule.version}
                  </p>
                </div>
                <button
                  onClick={() => setSelectedModule(null)}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
                >
                  ×
                </button>
              </div>

              <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div className="lg:col-span-2">
                  <p className="text-gray-700 dark:text-gray-300 mb-4">
                    {selectedModule.description}
                  </p>

                  <div className="mb-4">
                    <h3 className="font-semibold text-gray-900 dark:text-white mb-2">
                      Features
                    </h3>
                    <ul className="list-disc list-inside text-gray-600 dark:text-gray-300 space-y-1">
                      {selectedModule.features.map((feature, index) => (
                        <li key={index}>{feature}</li>
                      ))}
                    </ul>
                  </div>
                </div>

                <div>
                  <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-4">
                    <div className="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                      {selectedModule.price === 0 ? 'Free' : `$${selectedModule.price}`}
                    </div>
                    <button
                      onClick={() => handleInstallModule(selectedModule.id)}
                      className="w-full px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md transition-colors"
                    >
                      Install Module
                    </button>
                  </div>

                  <div className="space-y-3 text-sm">
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-300">Rating:</span>
                      <div className="flex items-center">
                        <Star className="w-4 h-4 text-yellow-400 mr-1" />
                        {selectedModule.rating.toFixed(1)}
                      </div>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-300">Downloads:</span>
                      <span className="text-gray-900 dark:text-white">
                        {selectedModule.downloads.toLocaleString()}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-300">Category:</span>
                      <span className="text-gray-900 dark:text-white">
                        {selectedModule.category}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-gray-600 dark:text-gray-300">Last Updated:</span>
                      <span className="text-gray-900 dark:text-white">
                        {new Date(selectedModule.lastUpdated).toLocaleDateString()}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default ModuleMarketplace;