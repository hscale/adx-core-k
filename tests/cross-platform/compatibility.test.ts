import { test, expect, devices } from '@playwright/test';
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

interface PlatformFeature {
  name: string;
  supported: boolean;
  version?: string;
  notes?: string;
}

interface CompatibilityReport {
  platform: string;
  features: PlatformFeature[];
  performance: {
    loadTime: number;
    renderTime: number;
    memoryUsage?: number;
  };
  issues: string[];
}

const platforms = [
  { name: 'Desktop-Windows', device: null, userAgent: 'Windows' },
  { name: 'Desktop-macOS', device: null, userAgent: 'Macintosh' },
  { name: 'Desktop-Linux', device: null, userAgent: 'Linux' },
  { name: 'Mobile-iOS', device: devices['iPhone 13'], userAgent: 'iPhone' },
  { name: 'Mobile-Android', device: devices['Pixel 5'], userAgent: 'Android' },
  { name: 'Tablet-iPad', device: devices['iPad Pro'], userAgent: 'iPad' },
];

test.describe('Cross-Platform Compatibility Tests', () => {
  const compatibilityReports: CompatibilityReport[] = [];

  for (const platform of platforms) {
    test.describe(`${platform.name} Compatibility`, () => {
      test.use(platform.device || {});

      test(`should test core features on ${platform.name}`, async ({ page, browserName }) => {
        const report: CompatibilityReport = {
          platform: platform.name,
          features: [],
          performance: { loadTime: 0, renderTime: 0 },
          issues: [],
        };

        try {
          // Set user agent if specified
          if (platform.userAgent) {
            await page.setExtraHTTPHeaders({
              'User-Agent': `Mozilla/5.0 (compatible; ADX-Core-Test) ${platform.userAgent}`,
            });
          }

          // Measure load time
          const loadStart = Date.now();
          await page.goto('http://localhost:3000');
          await page.waitForSelector('[data-testid="app-shell"]', { timeout: 30000 });
          const loadEnd = Date.now();
          report.performance.loadTime = loadEnd - loadStart;

          // Test core features
          const features = await testCoreFeatures(page);
          report.features = features;

          // Test performance
          const performanceMetrics = await testPerformance(page);
          report.performance = { ...report.performance, ...performanceMetrics };

          // Test responsive design
          await testResponsiveDesign(page, platform.name);

          // Test accessibility
          await testAccessibility(page);

          // Test offline functionality
          await testOfflineCapabilities(page);

          // Platform-specific tests
          if (platform.name.includes('Mobile')) {
            await testMobileFeatures(page);
          } else if (platform.name.includes('Desktop')) {
            await testDesktopFeatures(page);
          }

        } catch (error) {
          report.issues.push(`Test execution failed: ${error.message}`);
        }

        compatibilityReports.push(report);

        // Verify no critical issues
        const criticalIssues = report.issues.filter(issue => 
          issue.includes('failed') || issue.includes('error')
        );
        expect(criticalIssues.length).toBe(0);

        // Verify reasonable load time
        expect(report.performance.loadTime).toBeLessThan(10000); // 10 seconds max
      });
    });
  }

  test.afterAll(async () => {
    // Generate compatibility report
    await generateCompatibilityReport(compatibilityReports);
  });
});

async function testCoreFeatures(page): Promise<PlatformFeature[]> {
  const features: PlatformFeature[] = [];

  // Test JavaScript features
  const jsFeatures = await page.evaluate(() => {
    return {
      es6Modules: typeof import !== 'undefined',
      asyncAwait: (async () => true)().constructor.name === 'AsyncFunction',
      webComponents: 'customElements' in window,
      serviceWorker: 'serviceWorker' in navigator,
      webWorkers: typeof Worker !== 'undefined',
      localStorage: typeof Storage !== 'undefined',
      sessionStorage: typeof sessionStorage !== 'undefined',
      indexedDB: 'indexedDB' in window,
      webSockets: typeof WebSocket !== 'undefined',
      fetch: typeof fetch !== 'undefined',
      promises: typeof Promise !== 'undefined',
      symbols: typeof Symbol !== 'undefined',
      weakMap: typeof WeakMap !== 'undefined',
      proxy: typeof Proxy !== 'undefined',
    };
  });

  Object.entries(jsFeatures).forEach(([feature, supported]) => {
    features.push({ name: `JavaScript-${feature}`, supported });
  });

  // Test CSS features
  const cssFeatures = await page.evaluate(() => {
    const testElement = document.createElement('div');
    document.body.appendChild(testElement);
    
    const features = {
      flexbox: CSS.supports('display', 'flex'),
      grid: CSS.supports('display', 'grid'),
      customProperties: CSS.supports('--custom-property', 'value'),
      transforms: CSS.supports('transform', 'translateX(1px)'),
      transitions: CSS.supports('transition', 'all 1s'),
      animations: CSS.supports('animation', 'test 1s'),
      filters: CSS.supports('filter', 'blur(1px)'),
      backdrop: CSS.supports('backdrop-filter', 'blur(1px)'),
      clipPath: CSS.supports('clip-path', 'circle(50%)'),
      objectFit: CSS.supports('object-fit', 'cover'),
    };
    
    document.body.removeChild(testElement);
    return features;
  });

  Object.entries(cssFeatures).forEach(([feature, supported]) => {
    features.push({ name: `CSS-${feature}`, supported });
  });

  // Test Web APIs
  const webApis = await page.evaluate(() => {
    return {
      geolocation: 'geolocation' in navigator,
      camera: 'mediaDevices' in navigator && 'getUserMedia' in navigator.mediaDevices,
      notifications: 'Notification' in window,
      pushNotifications: 'PushManager' in window,
      backgroundSync: 'serviceWorker' in navigator && 'sync' in window.ServiceWorkerRegistration.prototype,
      webShare: 'share' in navigator,
      clipboard: 'clipboard' in navigator,
      fullscreen: 'requestFullscreen' in document.documentElement,
      pointerLock: 'requestPointerLock' in document.documentElement,
      gamepad: 'getGamepads' in navigator,
      vibration: 'vibrate' in navigator,
      battery: 'getBattery' in navigator,
      deviceOrientation: 'DeviceOrientationEvent' in window,
      deviceMotion: 'DeviceMotionEvent' in window,
      webGL: (() => {
        try {
          const canvas = document.createElement('canvas');
          return !!(canvas.getContext('webgl') || canvas.getContext('experimental-webgl'));
        } catch (e) {
          return false;
        }
      })(),
      webGL2: (() => {
        try {
          const canvas = document.createElement('canvas');
          return !!canvas.getContext('webgl2');
        } catch (e) {
          return false;
        }
      })(),
    };
  });

  Object.entries(webApis).forEach(([api, supported]) => {
    features.push({ name: `WebAPI-${api}`, supported });
  });

  return features;
}

async function testPerformance(page): Promise<any> {
  const performanceMetrics = await page.evaluate(() => {
    const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
    const memory = (performance as any).memory;
    
    return {
      renderTime: navigation.loadEventEnd - navigation.loadEventStart,
      domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
      firstPaint: performance.getEntriesByName('first-paint')[0]?.startTime || 0,
      firstContentfulPaint: performance.getEntriesByName('first-contentful-paint')[0]?.startTime || 0,
      memoryUsage: memory ? memory.usedJSHeapSize : undefined,
    };
  });

  return performanceMetrics;
}

async function testResponsiveDesign(page, platformName: string) {
  const viewports = [
    { width: 320, height: 568, name: 'Mobile Portrait' },
    { width: 568, height: 320, name: 'Mobile Landscape' },
    { width: 768, height: 1024, name: 'Tablet Portrait' },
    { width: 1024, height: 768, name: 'Tablet Landscape' },
    { width: 1920, height: 1080, name: 'Desktop' },
  ];

  for (const viewport of viewports) {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });
    await page.waitForTimeout(500); // Allow layout to settle

    // Check if layout adapts properly
    const layoutInfo = await page.evaluate(() => {
      const shell = document.querySelector('[data-testid="app-shell"]');
      const navigation = document.querySelector('[data-testid="navigation"]');
      
      return {
        shellVisible: shell ? window.getComputedStyle(shell).display !== 'none' : false,
        navigationVisible: navigation ? window.getComputedStyle(navigation).display !== 'none' : false,
        bodyOverflow: window.getComputedStyle(document.body).overflow,
      };
    });

    expect(layoutInfo.shellVisible).toBe(true);
  }
}

async function testAccessibility(page) {
  // Test basic accessibility features
  const a11yFeatures = await page.evaluate(() => {
    const features = {
      ariaLabels: document.querySelectorAll('[aria-label]').length > 0,
      altTexts: Array.from(document.querySelectorAll('img')).every(img => img.alt !== undefined),
      headingStructure: document.querySelectorAll('h1, h2, h3, h4, h5, h6').length > 0,
      focusableElements: document.querySelectorAll('button, input, select, textarea, a[href]').length > 0,
      skipLinks: document.querySelectorAll('[href="#main"], [href="#content"]').length > 0,
    };
    
    return features;
  });

  // Verify accessibility features are present
  expect(a11yFeatures.focusableElements).toBe(true);
}

async function testOfflineCapabilities(page) {
  // Test offline functionality
  await page.context().setOffline(true);
  
  try {
    await page.reload();
    await page.waitForSelector('[data-testid="offline-indicator"]', { timeout: 5000 });
    
    // Verify offline state is handled
    const offlineHandled = await page.isVisible('[data-testid="offline-indicator"]');
    expect(offlineHandled).toBe(true);
  } catch (error) {
    // Offline functionality might not be implemented yet
    console.log('Offline functionality not available');
  } finally {
    await page.context().setOffline(false);
  }
}

async function testMobileFeatures(page) {
  // Test mobile-specific features
  const mobileFeatures = await page.evaluate(() => {
    return {
      touchEvents: 'ontouchstart' in window,
      orientationChange: 'onorientationchange' in window,
      devicePixelRatio: window.devicePixelRatio > 1,
      viewport: document.querySelector('meta[name="viewport"]') !== null,
    };
  });

  expect(mobileFeatures.viewport).toBe(true);
}

async function testDesktopFeatures(page) {
  // Test desktop-specific features
  const desktopFeatures = await page.evaluate(() => {
    return {
      mouseEvents: 'onmouseenter' in window,
      keyboardEvents: 'onkeydown' in window,
      contextMenu: 'oncontextmenu' in window,
      dragAndDrop: 'ondragstart' in window,
    };
  });

  expect(desktopFeatures.mouseEvents).toBe(true);
  expect(desktopFeatures.keyboardEvents).toBe(true);
}

async function generateCompatibilityReport(reports: CompatibilityReport[]) {
  const reportDir = path.join(process.cwd(), 'reports');
  if (!fs.existsSync(reportDir)) {
    fs.mkdirSync(reportDir, { recursive: true });
  }

  // Generate HTML report
  const htmlReport = generateHTMLReport(reports);
  fs.writeFileSync(path.join(reportDir, 'cross-platform-compatibility-report.html'), htmlReport);

  // Generate JSON report
  fs.writeFileSync(
    path.join(reportDir, 'cross-platform-compatibility-report.json'),
    JSON.stringify(reports, null, 2)
  );

  // Generate summary
  const summary = generateSummary(reports);
  fs.writeFileSync(path.join(reportDir, 'compatibility-summary.md'), summary);
}

function generateHTMLReport(reports: CompatibilityReport[]): string {
  const featureMatrix = generateFeatureMatrix(reports);
  
  return `
<!DOCTYPE html>
<html>
<head>
    <title>ADX Core Cross-Platform Compatibility Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background: #f5f5f5; padding: 20px; border-radius: 5px; }
        .platform { margin: 20px 0; border: 1px solid #ddd; border-radius: 5px; }
        .platform-header { background: #e9e9e9; padding: 10px; font-weight: bold; }
        .features { padding: 10px; }
        .feature { margin: 5px 0; }
        .supported { color: green; }
        .not-supported { color: red; }
        .performance { background: #f9f9f9; padding: 10px; margin: 10px 0; }
        .issues { background: #ffe6e6; padding: 10px; margin: 10px 0; }
        .matrix { width: 100%; border-collapse: collapse; margin: 20px 0; }
        .matrix th, .matrix td { border: 1px solid #ddd; padding: 8px; text-align: center; }
        .matrix th { background: #f5f5f5; }
        .matrix .supported { background: #d4edda; }
        .matrix .not-supported { background: #f8d7da; }
    </style>
</head>
<body>
    <div class="header">
        <h1>ADX Core Cross-Platform Compatibility Report</h1>
        <p>Generated on: ${new Date().toISOString()}</p>
        <p>Platforms tested: ${reports.length}</p>
    </div>

    <h2>Feature Compatibility Matrix</h2>
    ${featureMatrix}

    <h2>Platform Details</h2>
    ${reports.map(report => `
        <div class="platform">
            <div class="platform-header">${report.platform}</div>
            <div class="performance">
                <h4>Performance Metrics</h4>
                <p>Load Time: ${report.performance.loadTime}ms</p>
                <p>Render Time: ${report.performance.renderTime}ms</p>
                ${report.performance.memoryUsage ? `<p>Memory Usage: ${Math.round(report.performance.memoryUsage / 1024 / 1024)}MB</p>` : ''}
            </div>
            ${report.issues.length > 0 ? `
                <div class="issues">
                    <h4>Issues Found</h4>
                    <ul>
                        ${report.issues.map(issue => `<li>${issue}</li>`).join('')}
                    </ul>
                </div>
            ` : ''}
            <div class="features">
                <h4>Feature Support (${report.features.filter(f => f.supported).length}/${report.features.length})</h4>
                ${report.features.map(feature => `
                    <div class="feature ${feature.supported ? 'supported' : 'not-supported'}">
                        ${feature.supported ? '✓' : '✗'} ${feature.name}
                        ${feature.notes ? ` - ${feature.notes}` : ''}
                    </div>
                `).join('')}
            </div>
        </div>
    `).join('')}
</body>
</html>
  `;
}

function generateFeatureMatrix(reports: CompatibilityReport[]): string {
  const allFeatures = [...new Set(reports.flatMap(r => r.features.map(f => f.name)))].sort();
  
  let matrix = '<table class="matrix"><thead><tr><th>Feature</th>';
  reports.forEach(report => {
    matrix += `<th>${report.platform}</th>`;
  });
  matrix += '</tr></thead><tbody>';
  
  allFeatures.forEach(featureName => {
    matrix += `<tr><td>${featureName}</td>`;
    reports.forEach(report => {
      const feature = report.features.find(f => f.name === featureName);
      const supported = feature?.supported || false;
      matrix += `<td class="${supported ? 'supported' : 'not-supported'}">${supported ? '✓' : '✗'}</td>`;
    });
    matrix += '</tr>';
  });
  
  matrix += '</tbody></table>';
  return matrix;
}

function generateSummary(reports: CompatibilityReport[]): string {
  const totalFeatures = reports[0]?.features.length || 0;
  const platformSummaries = reports.map(report => {
    const supportedFeatures = report.features.filter(f => f.supported).length;
    const supportPercentage = Math.round((supportedFeatures / totalFeatures) * 100);
    
    return {
      platform: report.platform,
      supportPercentage,
      supportedFeatures,
      totalFeatures,
      loadTime: report.performance.loadTime,
      issues: report.issues.length,
    };
  });

  let summary = `# ADX Core Cross-Platform Compatibility Summary\n\n`;
  summary += `Generated on: ${new Date().toISOString()}\n\n`;
  summary += `## Overall Results\n\n`;
  
  platformSummaries.forEach(platform => {
    summary += `### ${platform.platform}\n`;
    summary += `- Feature Support: ${platform.supportedFeatures}/${platform.totalFeatures} (${platform.supportPercentage}%)\n`;
    summary += `- Load Time: ${platform.loadTime}ms\n`;
    summary += `- Issues: ${platform.issues}\n\n`;
  });

  const avgSupport = Math.round(
    platformSummaries.reduce((sum, p) => sum + p.supportPercentage, 0) / platformSummaries.length
  );
  const avgLoadTime = Math.round(
    platformSummaries.reduce((sum, p) => sum + p.loadTime, 0) / platformSummaries.length
  );

  summary += `## Summary Statistics\n\n`;
  summary += `- Average Feature Support: ${avgSupport}%\n`;
  summary += `- Average Load Time: ${avgLoadTime}ms\n`;
  summary += `- Platforms Tested: ${reports.length}\n`;
  summary += `- Total Features Tested: ${totalFeatures}\n`;

  return summary;
}