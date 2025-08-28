import express from 'express';
import multer from 'multer';
import axios from 'axios';
import { z } from 'zod';
import { TenantRequest } from '../middleware/tenant.js';
import { uploadRateLimit } from '../middleware/rateLimit.js';
import { cacheService } from '../services/redis.js';
import { createLogger } from '../utils/logger.js';
import { ValidationError, NotFoundError } from '../middleware/errorHandler.js';

const router = express.Router();
const logger = createLogger('files-routes');

// Configure multer for file uploads
const upload = multer({
  storage: multer.memoryStorage(),
  limits: {
    fileSize: parseInt(process.env.MAX_FILE_SIZE || '1073741824'), // 1GB default
    files: 20, // Max 20 files per upload
  },
  fileFilter: (req, file, cb) => {
    // Add file type validation here if needed
    cb(null, true);
  },
});

// Validation schemas
const fileSearchSchema = z.object({
  path: z.string().optional().default('/'),
  query: z.string().optional(),
  type: z.enum(['file', 'folder', 'all']).optional(),
  mimeTypes: z.array(z.string()).optional(),
  extensions: z.array(z.string()).optional(),
  sizeMin: z.number().optional(),
  sizeMax: z.number().optional(),
  dateFrom: z.string().optional(),
  dateTo: z.string().optional(),
  createdBy: z.array(z.string()).optional(),
  tags: z.array(z.string()).optional(),
  sharedOnly: z.boolean().optional(),
  page: z.number().min(1).optional().default(1),
  pageSize: z.number().min(1).max(100).optional().default(20),
});

const createFolderSchema = z.object({
  name: z.string().min(1).max(255),
  parentPath: z.string().optional().default('/'),
});

const renameFileSchema = z.object({
  name: z.string().min(1).max(255),
});

const moveFilesSchema = z.object({
  fileIds: z.array(z.string()).min(1),
  targetPath: z.string(),
});

const shareFileSchema = z.object({
  shareType: z.enum(['public', 'email', 'team']),
  permissions: z.enum(['read', 'write', 'admin']),
  sharedWith: z.string().optional(),
  expiresAt: z.string().optional(),
  password: z.string().optional(),
  downloadAllowed: z.boolean().optional().default(true),
  message: z.string().optional(),
});

// Helper function to make API Gateway requests
const makeApiRequest = async (req: TenantRequest, endpoint: string, options: any = {}) => {
  const apiGatewayUrl = process.env.API_GATEWAY_URL || 'http://localhost:8080';
  const timeout = parseInt(process.env.API_GATEWAY_TIMEOUT || '30000');

  const config = {
    ...options,
    url: `${apiGatewayUrl}${endpoint}`,
    timeout,
    headers: {
      'Authorization': req.headers.authorization,
      'X-Tenant-ID': req.tenantId,
      'X-Request-ID': req.requestId,
      'Content-Type': 'application/json',
      ...options.headers,
    },
  };

  try {
    const response = await axios(config);
    return response.data;
  } catch (error: any) {
    logger.error('API Gateway request failed', {
      endpoint,
      error: error.message,
      status: error.response?.status,
      requestId: req.requestId,
    });

    if (error.response?.status === 404) {
      throw new NotFoundError(error.response.data?.message || 'Resource not found');
    }

    throw error;
  }
};

// GET /api/files - List files with caching
router.get('/', async (req: TenantRequest, res) => {
  try {
    const filters = fileSearchSchema.parse(req.query);
    const cacheKey = cacheService.generateFileKey(req.tenantId!, filters.path, filters);

    // Try to get from cache first
    const cachedFiles = await cacheService.getCachedFiles(req.tenantId!, filters.path, filters);
    if (cachedFiles) {
      logger.debug('Returning cached files', {
        tenantId: req.tenantId,
        path: filters.path,
        requestId: req.requestId,
      });
      return res.json(cachedFiles);
    }

    // Fetch from API Gateway
    const params = new URLSearchParams();
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        if (Array.isArray(value)) {
          value.forEach(v => params.append(`${key}[]`, v.toString()));
        } else {
          params.append(key, value.toString());
        }
      }
    });

    const files = await makeApiRequest(req, `/api/v1/files?${params.toString()}`);

    // Cache the result
    await cacheService.cacheFiles(req.tenantId!, filters.path, files, filters);

    logger.info('Files retrieved', {
      tenantId: req.tenantId,
      path: filters.path,
      count: files.items?.length || 0,
      requestId: req.requestId,
    });

    res.json(files);
  } catch (error) {
    logger.error('Get files error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/files/search - Search files
router.post('/search', async (req: TenantRequest, res) => {
  try {
    const filters = fileSearchSchema.parse(req.body);

    const files = await makeApiRequest(req, '/api/v1/files/search', {
      method: 'POST',
      data: filters,
    });

    logger.info('File search completed', {
      tenantId: req.tenantId,
      query: filters.query,
      count: files.items?.length || 0,
      requestId: req.requestId,
    });

    res.json(files);
  } catch (error) {
    logger.error('Search files error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/files/:id - Get single file
router.get('/:id', async (req: TenantRequest, res) => {
  try {
    const { id } = req.params;

    const file = await makeApiRequest(req, `/api/v1/files/${id}`);

    logger.info('File retrieved', {
      tenantId: req.tenantId,
      fileId: id,
      requestId: req.requestId,
    });

    res.json(file);
  } catch (error) {
    logger.error('Get file error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/files/upload - Upload files with progress tracking
router.post('/upload', uploadRateLimit, upload.array('files', 20), async (req: TenantRequest, res) => {
  try {
    const files = req.files as Express.Multer.File[];
    const { path = '/' } = req.body;

    if (!files || files.length === 0) {
      throw new ValidationError('No files provided');
    }

    logger.info('File upload started', {
      tenantId: req.tenantId,
      fileCount: files.length,
      path,
      requestId: req.requestId,
    });

    // Create form data for API Gateway
    const FormData = require('form-data');
    const formData = new FormData();
    
    files.forEach((file, index) => {
      formData.append('files', file.buffer, {
        filename: file.originalname,
        contentType: file.mimetype,
      });
    });
    
    formData.append('path', path);

    const uploadResult = await makeApiRequest(req, '/api/v1/workflows/file-upload', {
      method: 'POST',
      data: formData,
      headers: {
        ...formData.getHeaders(),
      },
      maxContentLength: Infinity,
      maxBodyLength: Infinity,
    });

    // Invalidate file cache for the path
    await cacheService.invalidateFileCache(req.tenantId!, path);

    logger.info('File upload completed', {
      tenantId: req.tenantId,
      fileCount: files.length,
      operationId: uploadResult.operationId,
      requestId: req.requestId,
    });

    res.status(202).json(uploadResult);
  } catch (error) {
    logger.error('Upload files error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/files/:id/download - Download file
router.get('/:id/download', async (req: TenantRequest, res) => {
  try {
    const { id } = req.params;

    const response = await axios({
      method: 'GET',
      url: `${process.env.API_GATEWAY_URL}/api/v1/files/${id}/download`,
      headers: {
        'Authorization': req.headers.authorization,
        'X-Tenant-ID': req.tenantId,
        'X-Request-ID': req.requestId,
      },
      responseType: 'stream',
    });

    // Forward headers
    res.set({
      'Content-Type': response.headers['content-type'],
      'Content-Length': response.headers['content-length'],
      'Content-Disposition': response.headers['content-disposition'],
    });

    response.data.pipe(res);

    logger.info('File download initiated', {
      tenantId: req.tenantId,
      fileId: id,
      requestId: req.requestId,
    });
  } catch (error) {
    logger.error('Download file error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/files/download - Download multiple files
router.post('/download', async (req: TenantRequest, res) => {
  try {
    const { fileIds } = req.body;

    if (!Array.isArray(fileIds) || fileIds.length === 0) {
      throw new ValidationError('File IDs array is required');
    }

    const response = await axios({
      method: 'POST',
      url: `${process.env.API_GATEWAY_URL}/api/v1/files/download`,
      headers: {
        'Authorization': req.headers.authorization,
        'X-Tenant-ID': req.tenantId,
        'X-Request-ID': req.requestId,
        'Content-Type': 'application/json',
      },
      data: { fileIds },
      responseType: 'stream',
    });

    // Forward headers
    res.set({
      'Content-Type': response.headers['content-type'],
      'Content-Length': response.headers['content-length'],
      'Content-Disposition': response.headers['content-disposition'],
    });

    response.data.pipe(res);

    logger.info('Multiple files download initiated', {
      tenantId: req.tenantId,
      fileCount: fileIds.length,
      requestId: req.requestId,
    });
  } catch (error) {
    logger.error('Download multiple files error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/files/folders - Create folder
router.post('/folders', async (req: TenantRequest, res) => {
  try {
    const { name, parentPath } = createFolderSchema.parse(req.body);

    const folder = await makeApiRequest(req, '/api/v1/files/folders', {
      method: 'POST',
      data: { name, parentPath },
    });

    // Invalidate cache for parent path
    await cacheService.invalidateFileCache(req.tenantId!, parentPath);

    logger.info('Folder created', {
      tenantId: req.tenantId,
      folderName: name,
      parentPath,
      requestId: req.requestId,
    });

    res.status(201).json(folder);
  } catch (error) {
    logger.error('Create folder error', { error, requestId: req.requestId });
    throw error;
  }
});

// PUT /api/files/:id/rename - Rename file
router.put('/:id/rename', async (req: TenantRequest, res) => {
  try {
    const { id } = req.params;
    const { name } = renameFileSchema.parse(req.body);

    const file = await makeApiRequest(req, `/api/v1/files/${id}/rename`, {
      method: 'PUT',
      data: { name },
    });

    // Invalidate cache
    await cacheService.invalidateFileCache(req.tenantId!);

    logger.info('File renamed', {
      tenantId: req.tenantId,
      fileId: id,
      newName: name,
      requestId: req.requestId,
    });

    res.json(file);
  } catch (error) {
    logger.error('Rename file error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/files/move - Move files
router.post('/move', async (req: TenantRequest, res) => {
  try {
    const { fileIds, targetPath } = moveFilesSchema.parse(req.body);

    const operation = await makeApiRequest(req, '/api/v1/workflows/move-files', {
      method: 'POST',
      data: { fileIds, targetPath },
    });

    // Invalidate cache
    await cacheService.invalidateFileCache(req.tenantId!);

    logger.info('Files move initiated', {
      tenantId: req.tenantId,
      fileCount: fileIds.length,
      targetPath,
      operationId: operation.operationId,
      requestId: req.requestId,
    });

    res.status(202).json(operation);
  } catch (error) {
    logger.error('Move files error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/files/copy - Copy files
router.post('/copy', async (req: TenantRequest, res) => {
  try {
    const { fileIds, targetPath } = moveFilesSchema.parse(req.body);

    const operation = await makeApiRequest(req, '/api/v1/workflows/copy-files', {
      method: 'POST',
      data: { fileIds, targetPath },
    });

    // Invalidate cache
    await cacheService.invalidateFileCache(req.tenantId!);

    logger.info('Files copy initiated', {
      tenantId: req.tenantId,
      fileCount: fileIds.length,
      targetPath,
      operationId: operation.operationId,
      requestId: req.requestId,
    });

    res.status(202).json(operation);
  } catch (error) {
    logger.error('Copy files error', { error, requestId: req.requestId });
    throw error;
  }
});

// DELETE /api/files/delete - Delete files
router.delete('/delete', async (req: TenantRequest, res) => {
  try {
    const { fileIds } = req.body;

    if (!Array.isArray(fileIds) || fileIds.length === 0) {
      throw new ValidationError('File IDs array is required');
    }

    const operation = await makeApiRequest(req, '/api/v1/workflows/delete-files', {
      method: 'DELETE',
      data: { fileIds },
    });

    // Invalidate cache
    await cacheService.invalidateFileCache(req.tenantId!);

    logger.info('Files deletion initiated', {
      tenantId: req.tenantId,
      fileCount: fileIds.length,
      operationId: operation.operationId,
      requestId: req.requestId,
    });

    res.status(202).json(operation);
  } catch (error) {
    logger.error('Delete files error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/files/:id/share - Share file
router.post('/:id/share', async (req: TenantRequest, res) => {
  try {
    const { id } = req.params;
    const shareSettings = shareFileSchema.parse(req.body);

    const share = await makeApiRequest(req, `/api/v1/files/${id}/share`, {
      method: 'POST',
      data: shareSettings,
    });

    logger.info('File shared', {
      tenantId: req.tenantId,
      fileId: id,
      shareType: shareSettings.shareType,
      requestId: req.requestId,
    });

    res.status(201).json(share);
  } catch (error) {
    logger.error('Share file error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/files/:id/shares - Get file shares
router.get('/:id/shares', async (req: TenantRequest, res) => {
  try {
    const { id } = req.params;

    const shares = await makeApiRequest(req, `/api/v1/files/${id}/shares`);

    logger.info('File shares retrieved', {
      tenantId: req.tenantId,
      fileId: id,
      shareCount: shares.length,
      requestId: req.requestId,
    });

    res.json(shares);
  } catch (error) {
    logger.error('Get file shares error', { error, requestId: req.requestId });
    throw error;
  }
});

// PUT /api/files/:id/permissions - Update file permissions
router.put('/:id/permissions', async (req: TenantRequest, res) => {
  try {
    const { id } = req.params;
    const permissions = req.body;

    const updatedPermissions = await makeApiRequest(req, `/api/v1/files/${id}/permissions`, {
      method: 'PUT',
      data: permissions,
    });

    logger.info('File permissions updated', {
      tenantId: req.tenantId,
      fileId: id,
      requestId: req.requestId,
    });

    res.json(updatedPermissions);
  } catch (error) {
    logger.error('Update file permissions error', { error, requestId: req.requestId });
    throw error;
  }
});

export { router as fileRoutes };