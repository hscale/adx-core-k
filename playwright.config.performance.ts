import { defineConfig, devices } from '@playwright/test';

/**
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './tests/performance',
  /* Run tests in files in parallel */
  fullyParallel: false, // Performance tests should run sequentially
  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,
  /* Retry on CI only */
  retries: process.env.CI ? 1 : 0,
  /* Opt out of parallel tests on CI. */
  workers: 1, // Performance tests need dedicated resources
  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ['html', { outputFolder: 'test-results/performance-html-report' }],
    ['json', { outputFile: 'test-results/performance-results.json' }],
    ['junit', { outputFile: 'test-results/performance-results.xml' }],
  ],
  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL: 'http://localhost:3000',

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: 'on-first-retry',
    
    /* Take screenshot on failure */
    screenshot: 'only-on-failure',
    
    /* Record video on failure */
    video: 'retain-on-failure',
    
    /* Performance testing specific settings */
    actionTimeout: 30 * 1000,
    navigationTimeout: 30 * 1000,
  },

  /* Configure projects for performance testing across platforms */
  projects: [
    {
      name: 'Performance Desktop Chrome',
      use: { 
        ...devices['Desktop Chrome'],
        viewport: { width: 1920, height: 1080 },
        // Throttle network for realistic testing
        launchOptions: {
          args: [
            '--disable-web-security',
            '--disable-features=TranslateUI',
            '--disable-ipc-flooding-protection',
          ],
        },
      },
    },

    {
      name: 'Performance Mobile Chrome',
      use: { 
        ...devices['Pixel 5'],
        // Simulate slower mobile network
        contextOptions: {
          offline: false,
        },
      },
    },

    {
      name: 'Performance Tablet iPad',
      use: { 
        ...devices['iPad Pro'],
      },
    },

    {
      name: 'Performance Low-End Mobile',
      use: { 
        ...devices['Galaxy S5'],
        // Simulate very slow network
        contextOptions: {
          offline: false,
        },
      },
    },
  ],

  /* Run your local dev server before starting the tests */
  webServer: {
    command: 'npm run dev:all',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },

  /* Global setup and teardown */
  globalSetup: require.resolve('./tests/performance/global-setup.ts'),
  globalTeardown: require.resolve('./tests/performance/global-teardown.ts'),

  /* Extended timeout for performance tests */
  timeout: 120 * 1000,
  expect: {
    timeout: 30 * 1000,
  },

  /* Output directory */
  outputDir: 'test-results/performance/',
});