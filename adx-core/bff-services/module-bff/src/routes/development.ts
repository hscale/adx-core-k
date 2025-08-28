import { Router } from 'express';
import { AuthenticatedRequest, requirePermission } from '../middleware/auth';
import { TenantRequest } from '../middleware/tenant';
import { createError } from '../middleware/errorHandler';

const router = Router();

// Apply development permission requirement to all routes
router.use(requirePermission('module:develop'));

// Mock development projects
const mockProjects = [
  {
    id: 'project-1',
    name: 'My Custom Module',
    description: 'A custom module for my specific needs',
    version: '1.0.0',
    author: 'John Developer',
    created: '2024-01-10T10:00:00Z',
    lastModified: '2024-01-15T14:30:00Z',
    status: 'draft',
    manifest: {
      name: 'My Custom Module',
      version: '1.0.0',
      description: 'A custom module for my specific needs',
      author: {
        name: 'John Developer',
        email: 'john@example.com',
      },
      license: 'MIT',
      adxCore: {
        minVersion: '2.0.0',
      },
      dependencies: {},
      permissions: ['database:read'],
      extensionPoints: {
        frontend: {
          components: ['./src/components/MyComponent.tsx'],
        },
      },
      resources: {
        memory: '256MB',
        cpu: '0.5',
        storage: '100MB',
        networkAccess: false,
      },
    },
    sourceFiles: [
      {
        path: 'src/index.ts',
        content: '// Module entry point\nexport default {};\n',
        language: 'typescript',
        lastModified: '2024-01-15T14:30:00Z',
      },
      {
        path: 'manifest.json',
        content: '{\n  "name": "My Custom Module",\n  "version": "1.0.0"\n}',
        language: 'json',
        lastModified: '2024-01-10T10:00:00Z',
      },
    ],
    testResults: {
      passed: 8,
      failed: 2,
      total: 10,
      coverage: 75,
      details: [
        {
          name: 'Component renders correctly',
          status: 'passed',
          duration: 150,
        },
        {
          name: 'API integration works',
          status: 'failed',
          duration: 300,
          error: 'Network timeout',
        },
      ],
    },
  },
];

// Get development projects
router.get('/projects', (req: AuthenticatedRequest & TenantRequest, res) => {
  // In a real implementation, this would filter by user and tenant
  res.json(mockProjects);
});

// Create new development project
router.post('/projects', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { name, description, version, author, manifest, sourceFiles } = req.body;
  
  if (!name || !description || !version || !author) {
    throw createError('Missing required fields', 400, 'MISSING_FIELDS');
  }
  
  const newProject = {
    id: `project-${Date.now()}`,
    name,
    description,
    version,
    author,
    created: new Date().toISOString(),
    lastModified: new Date().toISOString(),
    status: 'draft',
    manifest: manifest || {
      name,
      version,
      description,
      author: { name: author, email: '' },
      license: 'MIT',
      adxCore: { minVersion: '2.0.0' },
      dependencies: {},
      permissions: [],
      extensionPoints: {},
      resources: {
        memory: '256MB',
        cpu: '0.5',
        storage: '100MB',
        networkAccess: false,
      },
    },
    sourceFiles: sourceFiles || [
      {
        path: 'src/index.ts',
        content: '// Module entry point\nexport default {};\n',
        language: 'typescript',
        lastModified: new Date().toISOString(),
      },
    ],
    testResults: {
      passed: 0,
      failed: 0,
      total: 0,
      coverage: 0,
      details: [],
    },
  };
  
  mockProjects.push(newProject);
  res.status(201).json(newProject);
});

// Get specific development project
router.get('/projects/:projectId', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { projectId } = req.params;
  
  const project = mockProjects.find(p => p.id === projectId);
  if (!project) {
    throw createError('Project not found', 404, 'PROJECT_NOT_FOUND');
  }
  
  res.json(project);
});

// Update development project
router.put('/projects/:projectId', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { projectId } = req.params;
  const updates = req.body;
  
  const projectIndex = mockProjects.findIndex(p => p.id === projectId);
  if (projectIndex === -1) {
    throw createError('Project not found', 404, 'PROJECT_NOT_FOUND');
  }
  
  mockProjects[projectIndex] = {
    ...mockProjects[projectIndex],
    ...updates,
    lastModified: new Date().toISOString(),
  };
  
  res.json(mockProjects[projectIndex]);
});

// Delete development project
router.delete('/projects/:projectId', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { projectId } = req.params;
  
  const projectIndex = mockProjects.findIndex(p => p.id === projectId);
  if (projectIndex === -1) {
    throw createError('Project not found', 404, 'PROJECT_NOT_FOUND');
  }
  
  mockProjects.splice(projectIndex, 1);
  res.status(204).send();
});

// Get project source files
router.get('/projects/:projectId/files', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { projectId } = req.params;
  
  const project = mockProjects.find(p => p.id === projectId);
  if (!project) {
    throw createError('Project not found', 404, 'PROJECT_NOT_FOUND');
  }
  
  res.json(project.sourceFiles);
});

// Update project source file
router.put('/projects/:projectId/files/:filePath', (req: AuthenticatedRequest & TenantRequest, res) => {
  const { projectId, filePath } = req.params;
  const { content } = req.body;
  
  if (!filePath) {
    throw createError('File path is required', 400, 'MISSING_FILE_PATH');
  }
  
  const project = mockProjects.find(p => p.id === projectId);
  if (!project) {
    throw createError('Project not found', 404, 'PROJECT_NOT_FOUND');
  }
  
  const decodedPath = decodeURIComponent(filePath);
  const fileIndex = project.sourceFiles.findIndex(f => f.path === decodedPath);
  
  if (fileIndex === -1) {
    throw createError('File not found', 404, 'FILE_NOT_FOUND');
  }
  
  const existingFile = project.sourceFiles[fileIndex];
  if (!existingFile) {
    throw createError('File not found in project', 404, 'FILE_NOT_FOUND');
  }
  
  project.sourceFiles[fileIndex] = {
    path: existingFile.path,
    content,
    language: existingFile.language,
    lastModified: new Date().toISOString(),
  };
  
  project.lastModified = new Date().toISOString();
  
  res.json(project.sourceFiles[fileIndex]);
});

export { router as developmentRoutes };