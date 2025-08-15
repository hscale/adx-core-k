import React, { useState } from 'react';
import { Save, FileText, Folder, FolderOpen } from 'lucide-react';
import { ModuleDevelopmentProject, SourceFile } from '../types/module';

interface ModuleEditorProps {
  project: ModuleDevelopmentProject;
}

export const ModuleEditor: React.FC<ModuleEditorProps> = ({ project }) => {
  const [selectedFile, setSelectedFile] = useState<SourceFile | null>(
    project.sourceFiles[0] || null
  );
  const [fileContent, setFileContent] = useState(selectedFile?.content || '');
  const [hasChanges, setHasChanges] = useState(false);

  React.useEffect(() => {
    if (selectedFile) {
      setFileContent(selectedFile.content);
      setHasChanges(false);
    }
  }, [selectedFile]);

  const handleContentChange = (content: string) => {
    setFileContent(content);
    setHasChanges(content !== selectedFile?.content);
  };

  const handleSave = () => {
    if (selectedFile && hasChanges) {
      // In a real implementation, this would save to the backend
      console.log('Saving file:', selectedFile.path, fileContent);
      setHasChanges(false);
    }
  };

  const getFileIcon = (path: string) => {
    if (path.endsWith('.ts') || path.endsWith('.tsx')) {
      return '📘';
    } else if (path.endsWith('.js') || path.endsWith('.jsx')) {
      return '📙';
    } else if (path.endsWith('.json')) {
      return '📋';
    } else if (path.endsWith('.md')) {
      return '📝';
    }
    return '📄';
  };

  const getLanguageFromPath = (path: string): string => {
    if (path.endsWith('.ts') || path.endsWith('.tsx')) return 'typescript';
    if (path.endsWith('.js') || path.endsWith('.jsx')) return 'javascript';
    if (path.endsWith('.json')) return 'json';
    if (path.endsWith('.md')) return 'markdown';
    if (path.endsWith('.css')) return 'css';
    if (path.endsWith('.html')) return 'html';
    return 'text';
  };

  return (
    <div className="flex h-96 border border-gray-200 rounded-lg overflow-hidden">
      {/* File Explorer */}
      <div className="w-64 bg-gray-50 border-r border-gray-200">
        <div className="p-3 border-b border-gray-200">
          <h3 className="font-medium text-gray-900">Files</h3>
        </div>
        <div className="p-2">
          {project.sourceFiles.map((file) => (
            <button
              key={file.path}
              onClick={() => setSelectedFile(file)}
              className={`w-full flex items-center gap-2 px-2 py-1 text-left text-sm rounded hover:bg-gray-100 ${
                selectedFile?.path === file.path ? 'bg-blue-100 text-blue-700' : 'text-gray-700'
              }`}
            >
              <span>{getFileIcon(file.path)}</span>
              <span className="truncate">{file.path}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Editor */}
      <div className="flex-1 flex flex-col">
        {selectedFile ? (
          <>
            {/* Editor Header */}
            <div className="flex items-center justify-between px-4 py-2 bg-gray-50 border-b border-gray-200">
              <div className="flex items-center gap-2">
                <span>{getFileIcon(selectedFile.path)}</span>
                <span className="font-medium text-gray-900">{selectedFile.path}</span>
                {hasChanges && <span className="text-orange-500">•</span>}
              </div>
              <button
                onClick={handleSave}
                disabled={!hasChanges}
                className="flex items-center gap-1 px-3 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Save className="w-3 h-3" />
                Save
              </button>
            </div>

            {/* Editor Content */}
            <div className="flex-1 p-4">
              <textarea
                value={fileContent}
                onChange={(e) => handleContentChange(e.target.value)}
                className="w-full h-full font-mono text-sm border border-gray-300 rounded p-3 focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
                placeholder="Start coding..."
                spellCheck={false}
              />
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-500">
            <div className="text-center">
              <FileText className="w-12 h-12 mx-auto mb-2 text-gray-400" />
              <p>Select a file to start editing</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};