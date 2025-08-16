// Global teardown for Playwright E2E tests
import { chromium, FullConfig } from '@playwright/test';

async function globalTeardown(config: FullConfig) {
  console.log('🧹 Starting ADX CORE E2E Test Teardown');
  
  // Launch browser for teardown
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  
  try {
    // Clean up test data
    console.log('🗑️ Cleaning up test data...');
    
    const cleanupResponse = await page.request.post('http://localhost:8080/api/test/cleanup', {
      data: {
        testDataIds: process.env.E2E_TEST_DATA ? JSON.parse(process.env.E2E_TEST_DATA) : {},
      },
    });
    
    if (cleanupResponse.ok()) {
      console.log('✅ Test data cleanup complete');
    } else {
      console.warn('⚠️ Test data cleanup failed, but continuing...');
    }
    
  } catch (error) {
    console.warn('⚠️ Teardown error (non-fatal):', error);
  } finally {
    await context.close();
    await browser.close();
  }
  
  console.log('✅ ADX CORE E2E Test Teardown Complete');
}

export default globalTeardown;