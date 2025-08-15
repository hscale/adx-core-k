import React from 'react';
import { X } from 'lucide-react';
import { ModuleSearchFilters, ModuleCategory, PricingModel, Platform } from '../types/module';
import { getCategoryDisplayName } from '../utils/moduleUtils';

interface ModuleFiltersProps {
  filters: ModuleSearchFilters;
  onFiltersChange: (filters: ModuleSearchFilters) => void;
  onClose: () => void;
}

export const ModuleFilters: React.FC<ModuleFiltersProps> = ({
  filters,
  onFiltersChange,
  onClose,
}) => {
  const handleFilterChange = (key: keyof ModuleSearchFilters, value: any) => {
    onFiltersChange({
      ...filters,
      [key]: value,
    });
  };

  const clearFilters = () => {
    onFiltersChange({});
  };

  return (
    <div className="border-t border-gray-200 pt-4">
      <div className="flex items-center justify-between mb-4">
        <h3 className="font-medium text-gray-900">Filters</h3>
        <div className="flex items-center gap-2">
          <button
            onClick={clearFilters}
            className="text-sm text-gray-500 hover:text-gray-700"
          >
            Clear all
          </button>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* Category Filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Category
          </label>
          <select
            value={filters.category || ''}
            onChange={(e) => handleFilterChange('category', e.target.value || undefined)}
            className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="">All Categories</option>
            {Object.values(ModuleCategory).map((category) => (
              <option key={category} value={category}>
                {getCategoryDisplayName(category)}
              </option>
            ))}
          </select>
        </div>

        {/* Pricing Model Filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Pricing
          </label>
          <select
            value={filters.pricingModel || ''}
            onChange={(e) => handleFilterChange('pricingModel', e.target.value || undefined)}
            className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="">All Pricing</option>
            <option value={PricingModel.Free}>Free</option>
            <option value={PricingModel.OneTime}>One-time Purchase</option>
            <option value={PricingModel.Subscription}>Subscription</option>
            <option value={PricingModel.Usage}>Usage-based</option>
            <option value={PricingModel.Enterprise}>Enterprise</option>
          </select>
        </div>

        {/* Platform Filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Platform
          </label>
          <select
            value={filters.platform || ''}
            onChange={(e) => handleFilterChange('platform', e.target.value || undefined)}
            className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="">All Platforms</option>
            <option value={Platform.Web}>Web</option>
            <option value={Platform.Desktop}>Desktop</option>
            <option value={Platform.Mobile}>Mobile</option>
          </select>
        </div>

        {/* Rating Filter */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Minimum Rating
          </label>
          <select
            value={filters.rating || ''}
            onChange={(e) => handleFilterChange('rating', e.target.value ? Number(e.target.value) : undefined)}
            className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            <option value="">Any Rating</option>
            <option value="4">4+ Stars</option>
            <option value="3">3+ Stars</option>
            <option value="2">2+ Stars</option>
            <option value="1">1+ Stars</option>
          </select>
        </div>
      </div>

      {/* Price Range Filter */}
      {filters.pricingModel !== PricingModel.Free && (
        <div className="mt-6">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Price Range
          </label>
          <div className="flex items-center gap-4">
            <div className="flex-1">
              <input
                type="number"
                placeholder="Min price"
                value={filters.priceRange?.[0] || ''}
                onChange={(e) => {
                  const min = e.target.value ? Number(e.target.value) : undefined;
                  const max = filters.priceRange?.[1];
                  handleFilterChange('priceRange', min !== undefined || max !== undefined ? [min || 0, max || 1000] : undefined);
                }}
                className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>
            <span className="text-gray-500">to</span>
            <div className="flex-1">
              <input
                type="number"
                placeholder="Max price"
                value={filters.priceRange?.[1] || ''}
                onChange={(e) => {
                  const max = e.target.value ? Number(e.target.value) : undefined;
                  const min = filters.priceRange?.[0];
                  handleFilterChange('priceRange', min !== undefined || max !== undefined ? [min || 0, max || 1000] : undefined);
                }}
                className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              />
            </div>
          </div>
        </div>
      )}

      {/* Tags Filter */}
      <div className="mt-6">
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Tags (comma-separated)
        </label>
        <input
          type="text"
          placeholder="e.g., analytics, reporting, dashboard"
          value={filters.tags?.join(', ') || ''}
          onChange={(e) => {
            const tags = e.target.value
              .split(',')
              .map(tag => tag.trim())
              .filter(tag => tag.length > 0);
            handleFilterChange('tags', tags.length > 0 ? tags : undefined);
          }}
          className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
        />
      </div>
    </div>
  );
};