import { FullConfig } from '@playwright/test';
import fs from 'fs';
import path from 'path';

async function globalTeardown(config: FullConfig) {
  console.log('Tearing down performance testing environment...');

  // Generate performance report
  const testResultsDir = path.join(process.cwd(), 'test-results', 'performance');
  
  if (fs.existsSync(testResultsDir)) {
    const files = fs.readdirSync(testResultsDir);
    
    // Collect all performance data
    const performanceData: any[] = [];
    
    for (const file of files) {
      if (file.endsWith('.json') && file.includes('performance')) {
        try {
          const data = JSON.parse(fs.readFileSync(path.join(testResultsDir, file), 'utf8'));
          performanceData.push({ file, data });
        } catch (error) {
          console.warn(`Failed to parse ${file}:`, error);
        }
      }
    }

    // Generate performance summary
    const summary = {
      timestamp: new Date().toISOString(),
      totalTests: performanceData.length,
      baseline: performanceData.find(d => d.file.includes('baseline'))?.data,
      results: performanceData.filter(d => !d.file.includes('baseline')),
      recommendations: generatePerformanceRecommendations(performanceData),
    };

    fs.writeFileSync(
      path.join(testResultsDir, 'performance-summary.json'),
      JSON.stringify(summary, null, 2)
    );

    // Generate HTML report
    generateHTMLPerformanceReport(summary, testResultsDir);

    console.log(`Performance test summary generated: ${summary.totalTests} tests`);
  }

  console.log('Performance testing environment teardown complete');
}

function generatePerformanceRecommendations(performanceData: any[]): string[] {
  const recommendations: string[] = [];
  
  // Analyze load times
  const loadTimes = performanceData
    .filter(d => d.data.loadTime)
    .map(d => d.data.loadTime);
  
  if (loadTimes.length > 0) {
    const avgLoadTime = loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length;
    const maxLoadTime = Math.max(...loadTimes);
    
    if (avgLoadTime > 3000) {
      recommendations.push('Average load time exceeds 3 seconds. Consider optimizing bundle size and lazy loading.');
    }
    
    if (maxLoadTime > 5000) {
      recommendations.push('Maximum load time exceeds 5 seconds. Investigate slow-loading components.');
    }
  }
  
  // Analyze FCP times
  const fcpTimes = performanceData
    .filter(d => d.data.metrics?.firstContentfulPaint)
    .map(d => d.data.metrics.firstContentfulPaint);
  
  if (fcpTimes.length > 0) {
    const avgFCP = fcpTimes.reduce((a, b) => a + b, 0) / fcpTimes.length;
    
    if (avgFCP > 2000) {
      recommendations.push('First Contentful Paint exceeds 2 seconds. Consider server-side rendering or critical CSS inlining.');
    }
  }
  
  return recommendations;
}

function generateHTMLPerformanceReport(summary: any, outputDir: string) {
  const html = `
<!DOCTYPE html>
<html>
<head>
    <title>ADX Core Performance Test Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f5f5f5; padding: 20px; border-radius: 5px; }
        .metric { margin: 10px 0; padding: 10px; border: 1px solid #ddd; border-radius: 3px; }
        .good { background: #d4edda; }
        .warning { background: #fff3cd; }
        .danger { background: #f8d7da; }
        .chart { margin: 20px 0; }
        table { width: 100%; border-collapse: collapse; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background: #f5f5f5; }
    </style>
</head>
<body>
    <div class="header">
        <h1>ADX Core Performance Test Report</h1>
        <p>Generated on: ${summary.timestamp}</p>
        <p>Total tests: ${summary.totalTests}</p>
    </div>

    <h2>Performance Metrics</h2>
    <table>
        <thead>
            <tr>
                <th>Test</th>
                <th>Load Time (ms)</th>
                <th>First Contentful Paint (ms)</th>
                <th>DOM Content Loaded (ms)</th>
            </tr>
        </thead>
        <tbody>
            ${summary.results.map((result: any) => `
                <tr>
                    <td>${result.file}</td>
                    <td>${result.data.loadTime || 'N/A'}</td>
                    <td>${result.data.metrics?.firstContentfulPaint || 'N/A'}</td>
                    <td>${result.data.metrics?.domContentLoaded || 'N/A'}</td>
                </tr>
            `).join('')}
        </tbody>
    </table>

    <h2>Recommendations</h2>
    <ul>
        ${summary.recommendations.map((rec: string) => `<li>${rec}</li>`).join('')}
    </ul>
</body>
</html>
  `;

  fs.writeFileSync(path.join(outputDir, 'performance-report.html'), html);
}

export default globalTeardown;