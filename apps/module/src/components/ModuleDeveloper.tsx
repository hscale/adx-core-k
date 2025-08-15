import React, { useState } from 'react';
import { Plus, Code, Play, Upload, FileText, Settings, TestTube, CheckCircle, AlertTriangle } from 'lucide-react';
import { useModuleDevelopment, useModuleDevelopmentProject } from '../hooks/useModuleDevelopment';
import { ModuleDevelopmentProject, ModuleManifest } from '../types/module';
import { validateModuleName, validateVersion, generateModuleId } from '../utils/moduleUtils';
import { ModuleEditor } from './ModuleEditor';
import { ModuleTestRunner } from './ModuleTestRunner';

export const ModuleDeveloper: React.FC = () => {
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);
  const [showCreateProject, setShowCreateProject] = useState(false);
  const [activeTab, setActiveTab] = useState<'overview' | 'editor' | 'test' | 'publish'>('overview');

  const { projects, createProject, testModule, publishModule } = useModuleDevelopment();

  const handleCreateProject = async (projectData: Omit<ModuleDevelopmentProject, 'id' | 'created' | 'lastModified'>) => {
    try {
      const newProject = await createProject.mutateAsync(projectData);
      setSelectedProjectId(newProject.id);
      setShowCreateProject(false);
    } catch (error) {
      console.error('Failed to create project:', error);
    }
  };

  const handleTestModule = async (projectId: string) => {
    try {
      await testModule.mutateAsync(projectId);
    } catch (error) {
      console.error('Failed to test module:', error);
    }
  };

  const handlePublishModule = async (projectId: string) => {
    try {
      await publishModule.mutateAsync(projectId);
    } catch (error) {
      console.error('Failed to publish module:', error);
    }
  };

  if (projects.isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (projects.error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="flex items-center gap-2">
          <AlertTriangle className="w-5 h-5 text-red-600" />
          <p className="text-red-800">
            Failed to load development projects: {(projects.error as Error).message}
          </p>
        </div>
      </div>
    );
  }

  const projectList = projects.data || [];

  if (selectedProjectId) {
    return (
      <ModuleDeveloperProject
        projectId={selectedProjectId}
        activeTab={activeTab}
        onTabChange={setActiveTab}
        onBack={() => setSelectedProjectId(null)}
        onTest={() => handleTestModule(selectedProjectId)}
        onPublish={() => handlePublishModule(selectedProjectId)}
      />
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Module Development</h1>
          <p className="text-gray-600">Create and manage your custom modules</p>
        </div>
        <button
          onClick={() => setShowCreateProject(true)}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          <Plus className="w-4 h-4" />
          New Project
        </button>
      </div>

      {/* Projects List */}
      {projectList.length === 0 ? (
        <div className="text-center py-12">
          <Code className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-500 mb-4">No development projects yet.</p>
          <button
            onClick={() => setShowCreateProject(true)}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
          >
            Create Your First Project
          </button>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {projectList.map((project) => (
            <div
              key={project.id}
              className="bg-white rounded-lg shadow-sm border border-gray-200 hover:shadow-md transition-shadow cursor-pointer"
              onClick={() => setSelectedProjectId(project.id)}
            >
              <div className="p-6">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="font-semibold text-gray-900">{project.name}</h3>
                  <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${
                    project.status === 'published'
                      ? 'bg-green-100 text-green-800'
                      : project.status === 'ready'
                      ? 'bg-blue-100 text-blue-800'
                      : project.status === 'testing'
                      ? 'bg-yellow-100 text-yellow-800'
                      : 'bg-gray-100 text-gray-800'
                  }`}>
                    {project.status}
                  </span>
                </div>
                
                <p className="text-sm text-gray-600 mb-4 line-clamp-2">
                  {project.description}
                </p>
                
                <div className="flex items-center justify-between text-xs text-gray-500">
                  <span>v{project.version}</span>
                  <span>{new Date(project.lastModified).toLocaleDateString()}</span>
                </div>
                
                {project.testResults && (
                  <div className="mt-3 flex items-center gap-2 text-xs">
                    <div className={`flex items-center gap-1 ${
                      project.testResults.failed === 0 ? 'text-green-600' : 'text-red-600'
                    }`}>
                      {project.testResults.failed === 0 ? (
                        <CheckCircle className="w-3 h-3" />
                      ) : (
                        <AlertTriangle className="w-3 h-3" />
                      )}
                      {project.testResults.passed}/{project.testResults.total} tests
                    </div>
                    <span className="text-gray-400">•</span>
                    <span className="text-gray-600">
                      {project.testResults.coverage}% coverage
                    </span>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Create Project Modal */}
      {showCreateProject && (
        <CreateProjectModal
          onClose={() => setShowCreateProject(false)}
          onCreate={handleCreateProject}
          isCreating={createProject.isPending}
        />
      )}
    </div>
  );
};

interface CreateProjectModalProps {
  onClose: () => void;
  onCreate: (project: Omit<ModuleDevelopmentProject, 'id' | 'created' | 'lastModified'>) => void;
  isCreating: boolean;
}

const CreateProjectModal: React.FC<CreateProjectModalProps> = ({ onClose, onCreate, isCreating }) => {
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    version: '1.0.0',
    author: '',
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    const newErrors: Record<string, string> = {};
    
    const nameError = validateModuleName(formData.name);
    if (nameError) newErrors.name = nameError;
    
    const versionError = validateVersion(formData.version);
    if (versionError) newErrors.version = versionError;
    
    if (!formData.description.trim()) {
      newErrors.description = 'Description is required';
    }
    
    if (!formData.author.trim()) {
      newErrors.author = 'Author is required';
    }
    
    setErrors(newErrors);
    
    if (Object.keys(newErrors).length === 0) {
      const moduleId = generateModuleId(formData.name);
      
      const manifest: ModuleManifest = {
        name: formData.name,
        version: formData.version,
        description: formData.description,
        author: {
          name: formData.author,
          email: '',
        },
        license: 'MIT',
        adxCore: {
          minVersion: '2.0.0',
        },
        dependencies: {},
        permissions: [],
        extensionPoints: {},
        resources: {
          memory: '256MB',
          cpu: '0.5',
          storage: '100MB',
          networkAccess: false,
        },
      };
      
      onCreate({
        name: formData.name,
        description: formData.description,
        version: formData.version,
        author: formData.author,
        status: 'draft',
        manifest,
        sourceFiles: [
          {
            path: 'src/index.ts',
            content: '// Module entry point\nexport default {};\n',
            language: 'typescript',
            lastModified: new Date().toISOString(),
          },
          {
            path: 'manifest.json',
            content: JSON.stringify(manifest, null, 2),
            language: 'json',
            lastModified: new Date().toISOString(),
          },
        ],
      });
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full mx-4">
        <div className="p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Create New Module Project</h2>
          
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Module Name
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                className={`w-full border rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
                  errors.name ? 'border-red-300' : 'border-gray-300'
                }`}
                placeholder="My Awesome Module"
              />
              {errors.name && <p className="text-red-600 text-xs mt-1">{errors.name}</p>}
            </div>
            
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Description
              </label>
              <textarea
                value={formData.description}
                onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                className={`w-full border rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
                  errors.description ? 'border-red-300' : 'border-gray-300'
                }`}
                rows={3}
                placeholder="A brief description of what your module does..."
              />
              {errors.description && <p className="text-red-600 text-xs mt-1">{errors.description}</p>}
            </div>
            
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Version
                </label>
                <input
                  type="text"
                  value={formData.version}
                  onChange={(e) => setFormData(prev => ({ ...prev, version: e.target.value }))}
                  className={`w-full border rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
                    errors.version ? 'border-red-300' : 'border-gray-300'
                  }`}
                  placeholder="1.0.0"
                />
                {errors.version && <p className="text-red-600 text-xs mt-1">{errors.version}</p>}
              </div>
              
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  Author
                </label>
                <input
                  type="text"
                  value={formData.author}
                  onChange={(e) => setFormData(prev => ({ ...prev, author: e.target.value }))}
                  className={`w-full border rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent ${
                    errors.author ? 'border-red-300' : 'border-gray-300'
                  }`}
                  placeholder="Your Name"
                />
                {errors.author && <p className="text-red-600 text-xs mt-1">{errors.author}</p>}
              </div>
            </div>
            
            <div className="flex justify-end gap-3 pt-4">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={isCreating}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
              >
                {isCreating ? 'Creating...' : 'Create Project'}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
};

interface ModuleDeveloperProjectProps {
  projectId: string;
  activeTab: 'overview' | 'editor' | 'test' | 'publish';
  onTabChange: (tab: 'overview' | 'editor' | 'test' | 'publish') => void;
  onBack: () => void;
  onTest: () => void;
  onPublish: () => void;
}

const ModuleDeveloperProject: React.FC<ModuleDeveloperProjectProps> = ({
  projectId,
  activeTab,
  onTabChange,
  onBack,
  onTest,
  onPublish,
}) => {
  const { project } = useModuleDevelopmentProject(projectId);

  if (project.isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (project.error || !project.data) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="flex items-center gap-2">
          <AlertTriangle className="w-5 h-5 text-red-600" />
          <p className="text-red-800">
            Failed to load project: {project.error ? (project.error as Error).message : 'Project not found'}
          </p>
        </div>
      </div>
    );
  }

  const projectData = project.data;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <button
            onClick={onBack}
            className="text-gray-600 hover:text-gray-900"
          >
            ← Back
          </button>
          <div>
            <h1 className="text-2xl font-bold text-gray-900">{projectData.name}</h1>
            <p className="text-gray-600">{projectData.description}</p>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <button
            onClick={onTest}
            className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
          >
            <TestTube className="w-4 h-4" />
            Test
          </button>
          <button
            onClick={onPublish}
            disabled={projectData.status !== 'ready'}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
          >
            <Upload className="w-4 h-4" />
            Publish
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {[
            { key: 'overview', label: 'Overview', icon: FileText },
            { key: 'editor', label: 'Code Editor', icon: Code },
            { key: 'test', label: 'Testing', icon: TestTube },
            { key: 'publish', label: 'Publish', icon: Upload },
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => onTabChange(tab.key as any)}
              className={`flex items-center gap-2 py-2 px-1 border-b-2 font-medium text-sm ${
                activeTab === tab.key
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <tab.icon className="w-4 h-4" />
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        {activeTab === 'overview' && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <h3 className="font-medium text-gray-900 mb-2">Project Information</h3>
                <dl className="space-y-2 text-sm">
                  <div>
                    <dt className="text-gray-500">Version:</dt>
                    <dd className="text-gray-900">{projectData.version}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-500">Author:</dt>
                    <dd className="text-gray-900">{projectData.author}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-500">Status:</dt>
                    <dd className="text-gray-900">{projectData.status}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-500">Last Modified:</dt>
                    <dd className="text-gray-900">
                      {new Date(projectData.lastModified).toLocaleString()}
                    </dd>
                  </div>
                </dl>
              </div>
              
              <div>
                <h3 className="font-medium text-gray-900 mb-2">Files</h3>
                <ul className="space-y-1 text-sm">
                  {projectData.sourceFiles.map((file) => (
                    <li key={file.path} className="text-gray-600">
                      {file.path}
                    </li>
                  ))}
                </ul>
              </div>
            </div>
            
            {projectData.testResults && (
              <div>
                <h3 className="font-medium text-gray-900 mb-2">Test Results</h3>
                <div className="bg-gray-50 rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm text-gray-600">
                      {projectData.testResults.passed}/{projectData.testResults.total} tests passed
                    </span>
                    <span className="text-sm text-gray-600">
                      {projectData.testResults.coverage}% coverage
                    </span>
                  </div>
                  <div className="w-full bg-gray-200 rounded-full h-2">
                    <div
                      className="bg-green-600 h-2 rounded-full"
                      style={{ width: `${(projectData.testResults.passed / projectData.testResults.total) * 100}%` }}
                    />
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
        
        {activeTab === 'editor' && (
          <ModuleEditor project={projectData} />
        )}
        
        {activeTab === 'test' && (
          <ModuleTestRunner project={projectData} />
        )}
        
        {activeTab === 'publish' && (
          <div className="text-center py-12">
            <Upload className="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <p className="text-gray-500 mb-4">Publishing functionality coming soon...</p>
          </div>
        )}
      </div>
    </div>
  );
};