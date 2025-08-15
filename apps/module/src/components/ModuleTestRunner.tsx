import React, { useState } from 'react';
import { Play, CheckCircle, XCircle, Clock, AlertTriangle } from 'lucide-react';
import { ModuleDevelopmentProject, TestCase } from '../types/module';

interface ModuleTestRunnerProps {
  project: ModuleDevelopmentProject;
}

export const ModuleTestRunner: React.FC<ModuleTestRunnerProps> = ({ project }) => {
  const [isRunning, setIsRunning] = useState(false);
  const [testResults, setTestResults] = useState(project.testResults);

  const handleRunTests = async () => {
    setIsRunning(true);
    
    // Simulate test execution
    setTimeout(() => {
      const mockResults = {
        passed: Math.floor(Math.random() * 10) + 5,
        failed: Math.floor(Math.random() * 3),
        total: 0,
        coverage: Math.floor(Math.random() * 30) + 70,
        details: [] as TestCase[],
      };
      
      mockResults.total = mockResults.passed + mockResults.failed;
      
      // Generate mock test cases
      for (let i = 0; i < mockResults.total; i++) {
        mockResults.details.push({
          name: `Test case ${i + 1}`,
          status: i < mockResults.passed ? 'passed' : 'failed',
          duration: Math.floor(Math.random() * 1000) + 100,
          error: i >= mockResults.passed ? 'Assertion failed: expected true but got false' : undefined,
        });
      }
      
      setTestResults(mockResults);
      setIsRunning(false);
    }, 3000);
  };

  const getStatusIcon = (status: TestCase['status']) => {
    switch (status) {
      case 'passed':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'failed':
        return <XCircle className="w-4 h-4 text-red-500" />;
      case 'skipped':
        return <Clock className="w-4 h-4 text-yellow-500" />;
      default:
        return <AlertTriangle className="w-4 h-4 text-gray-500" />;
    }
  };

  const getStatusColor = (status: TestCase['status']) => {
    switch (status) {
      case 'passed':
        return 'text-green-700 bg-green-50';
      case 'failed':
        return 'text-red-700 bg-red-50';
      case 'skipped':
        return 'text-yellow-700 bg-yellow-50';
      default:
        return 'text-gray-700 bg-gray-50';
    }
  };

  return (
    <div className="space-y-6">
      {/* Test Controls */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium text-gray-900">Test Runner</h3>
          <p className="text-gray-600">Run tests to validate your module functionality</p>
        </div>
        <button
          onClick={handleRunTests}
          disabled={isRunning}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
        >
          {isRunning ? (
            <>
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
              Running Tests...
            </>
          ) : (
            <>
              <Play className="w-4 h-4" />
              Run Tests
            </>
          )}
        </button>
      </div>

      {/* Test Results Summary */}
      {testResults && (
        <div className="bg-gray-50 rounded-lg p-4">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-gray-900">{testResults.total}</div>
              <div className="text-sm text-gray-600">Total Tests</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-green-600">{testResults.passed}</div>
              <div className="text-sm text-gray-600">Passed</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-red-600">{testResults.failed}</div>
              <div className="text-sm text-gray-600">Failed</div>
            </div>
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">{testResults.coverage}%</div>
              <div className="text-sm text-gray-600">Coverage</div>
            </div>
          </div>

          {/* Progress Bar */}
          <div className="w-full bg-gray-200 rounded-full h-2 mb-2">
            <div
              className={`h-2 rounded-full ${
                testResults.failed === 0 ? 'bg-green-600' : 'bg-red-600'
              }`}
              style={{ width: `${(testResults.passed / testResults.total) * 100}%` }}
            />
          </div>
          
          <div className="text-sm text-gray-600 text-center">
            {testResults.failed === 0 ? 'All tests passed!' : `${testResults.failed} test(s) failed`}
          </div>
        </div>
      )}

      {/* Test Cases */}
      {testResults?.details && testResults.details.length > 0 && (
        <div className="bg-white border border-gray-200 rounded-lg">
          <div className="px-4 py-3 border-b border-gray-200">
            <h4 className="font-medium text-gray-900">Test Cases</h4>
          </div>
          <div className="divide-y divide-gray-200">
            {testResults.details.map((testCase, index) => (
              <div key={index} className="p-4">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-3">
                    {getStatusIcon(testCase.status)}
                    <span className="font-medium text-gray-900">{testCase.name}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(testCase.status)}`}>
                      {testCase.status}
                    </span>
                    <span className="text-sm text-gray-500">{testCase.duration}ms</span>
                  </div>
                </div>
                
                {testCase.error && (
                  <div className="mt-2 p-3 bg-red-50 border border-red-200 rounded text-sm text-red-700">
                    <strong>Error:</strong> {testCase.error}
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* No Results */}
      {!testResults && !isRunning && (
        <div className="text-center py-12">
          <Play className="w-12 h-12 text-gray-400 mx-auto mb-4" />
          <p className="text-gray-500 mb-4">No test results yet.</p>
          <p className="text-sm text-gray-400">Click "Run Tests" to execute your module tests.</p>
        </div>
      )}

      {/* Running State */}
      {isRunning && (
        <div className="text-center py-12">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Running tests...</p>
        </div>
      )}
    </div>
  );
};