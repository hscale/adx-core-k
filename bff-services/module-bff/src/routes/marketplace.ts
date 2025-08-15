import { Router } from 'express';
import { AuthenticatedRequest } from '../middleware/auth';
import { TenantRequest } from '../middleware/tenant';
import { createError } from '../middleware/errorHandler';

const router = Router();

// Mock module data
const mockModules = [
  {
    id: 'client-management',
    name: 'Client Management',
    version: '1.2.0',
    description: 'Comprehensive client and customer management system',
    longDescription: 'Full-featured client management with customer portals, project tracking, and communication tools',
    author: {
      name: 'ADX Core Team',
      email: 'modules@adxcore.com',
      website: 'https://adxcore.com',
    },
    category: 'business-management',
    price: 0,
    pricingModel: 'free',
    rating: 4.8,
    reviewCount: 156,
    downloads: 2847,
    activeInstallations: 1203,
    screenshots: [
      { url: '/screenshots/client-mgmt-1.png', caption: 'Client Dashboard' },
    ],
    documentationUrl: 'https://docs.adxcore.com/modules/client-management',
    supportUrl: 'https://support.adxcore.com',
    tags: ['crm', 'clients', 'management'],
    supportedPlatforms: ['web', 'desktop', 'mobile'],
    compatibility: {
      minAdxVersion: '2.0.0',
      supportedPlatforms: ['web', 'desktop', 'mobile'],
      dependencies: [],
    },
    securityScanResults: {
      passed: true,
      score: 95,
      issues: [],
      lastScanned: '2024-01-15T10:00:00Z',
    },
    performanceMetrics: {
      bundleSize: 245760,
      loadTime: 1200,
      memoryUsage: 45,
      cpuUsage: 12,
    },
    lastUpdated: '2024-01-10T15:30:00Z',
    changelog: [
      {
        version: '1.2.0',
        date: '2024-01-10',
        changes: ['Added project tracking', 'Improved performance'],
      },
    ],
    status: 'published',
  },
  {
    id: 'analytics-dashboard',
    name: 'Analytics Dashboard',
    version: '2.1.0',
    description: 'Advanced analytics and reporting dashboard',
    longDescription: 'Comprehensive analytics with customizable dashboards, real-time metrics, and advanced reporting',
    author: {
      name: 'Analytics Pro',
      email: 'support@analyticspro.com',
      website: 'https://analyticspro.com',
    },
    category: 'analytics',
    price: 29.99,
    pricingModel: 'subscription',
    rating: 4.6,
    reviewCount: 89,
    downloads: 1456,
    activeInstallations: 678,
    screenshots: [
      { url: '/screenshots/analytics-1.png', caption: 'Main Dashboard' },
    ],
    documentationUrl: 'https://docs.analyticspro.com',
    supportUrl: 'https://support.analyticspro.com',
    tags: ['analytics', 'dashboard', 'reporting'],
    supportedPlatforms: ['web', 'desktop'],
    compatibility: {
      minAdxVersion: '2.0.0',
      supportedPlatforms: ['web', 'desktop'],
      dependencies: [],
    },
    securityScanResults: {
      passed: true,
      score: 88,
      issues: [],
      lastScanned: '2024-01-12T14:20:00Z',
    },
    performanceMetrics: {
      bundleSize: 512000,
      loadTime: 1800,
      memoryUsage: 78,
      cpuUsage: 25,
    },
    lastUpdated: '2024-01-08T09:15:00Z',
    changelog: [
      {
        version: '2.1.0',
        date: '2024-01-08',
        changes: ['New chart types', 'Performance improvements'],
      },
    ],
    status: 'published',
  },
];

// Search modules
router.get('/search', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { q, category, pricingModel, platform, rating, page = '1', pageSize = '20' } = req.query;
  
  let filteredModules = [...mockModules];
  
  // Apply filters
  if (q) {
    const query = (q as string).toLowerCase();
    filteredModules = filteredModules.filter(module =>
      module.name.toLowerCase().includes(query) ||
      module.description.toLowerCase().includes(query) ||
      module.tags.some(tag => tag.toLowerCase().includes(query))
    );
  }
  
  if (category) {
    filteredModules = filteredModules.filter(module => module.category === category);
  }
  
  if (pricingModel) {
    filteredModules = filteredModules.filter(module => module.pricingModel === pricingModel);
  }
  
  if (platform) {
    filteredModules = filteredModules.filter(module =>
      module.supportedPlatforms.includes(platform as string)
    );
  }
  
  if (rating) {
    const minRating = parseFloat(rating as string);
    filteredModules = filteredModules.filter(module => module.rating >= minRating);
  }
  
  // Pagination
  const pageNum = parseInt(page as string);
  const pageSizeNum = parseInt(pageSize as string);
  const startIndex = (pageNum - 1) * pageSizeNum;
  const endIndex = startIndex + pageSizeNum;
  
  const paginatedModules = filteredModules.slice(startIndex, endIndex);
  
  res.json({
    modules: paginatedModules,
    total: filteredModules.length,
    page: pageNum,
    pageSize: pageSizeNum,
    filters: { q, category, pricingModel, platform, rating },
  });
});

// Get specific module
router.get('/modules/:moduleId', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { moduleId } = req.params;
  const module = mockModules.find(m => m.id === moduleId);
  
  if (!module) {
    throw createError('Module not found', 404, 'MODULE_NOT_FOUND');
  }
  
  res.json(module);
});

// Get featured modules
router.get('/featured', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { limit = '10' } = req.query;
  const limitNum = parseInt(limit as string);
  
  // Return top-rated modules as featured
  const featured = mockModules
    .sort((a, b) => b.rating - a.rating)
    .slice(0, limitNum);
  
  res.json(featured);
});

// Get trending modules
router.get('/trending', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { limit = '10' } = req.query;
  const limitNum = parseInt(limit as string);
  
  // Return most downloaded modules as trending
  const trending = mockModules
    .sort((a, b) => b.downloads - a.downloads)
    .slice(0, limitNum);
  
  res.json(trending);
});

// Get recommended modules
router.get('/recommended', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { limit = '10' } = req.query;
  const limitNum = parseInt(limit as string);
  
  // Return a mix of highly rated and popular modules
  const recommended = mockModules
    .sort((a, b) => (b.rating * b.downloads) - (a.rating * a.downloads))
    .slice(0, limitNum);
  
  res.json(recommended);
});

export { router as marketplaceRoutes };