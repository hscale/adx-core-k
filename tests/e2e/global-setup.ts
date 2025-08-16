// Global setup for Playwright E2E tests
import { chromium, FullConfig } from '@playwright/test';

async function globalSetup(config: FullConfig) {
  console.log('🚀 Starting ADX CORE E2E Test Setup');
  
  // Launch browser for setup
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  
  try {
    // Wait for services to be ready
    console.log('⏳ Waiting for services to be ready...');
    
    // Check API Gateway health
    await page.goto('http://localhost:8080/health', { waitUntil: 'networkidle' });
    await page.waitForSelector('text=healthy', { timeout: 60000 });
    console.log('✅ API Gateway is healthy');
    
    // Check Shell Application
    await page.goto('http://localhost:3000', { waitUntil: 'networkidle' });
    await page.waitForSelector('[data-testid="shell-app"]', { timeout: 60000 });
    console.log('✅ Shell Application is ready');
    
    // Setup test data
    console.log('🔧 Setting up test data...');
    
    const setupResponse = await page.request.post('http://localhost:8080/api/test/setup', {
      data: {
        tenants: [
          {
            name: 'E2E Test Tenant 1',
            adminEmail: 'admin1@e2e.test',
            subscriptionTier: 'professional',
          },
          {
            name: 'E2E Test Tenant 2', 
            adminEmail: 'admin2@e2e.test',
            subscriptionTier: 'enterprise',
          },
        ],
        users: [
          {
            email: 'user@e2e.test',
            password: 'TestPassword123!',
            tenants: ['e2e-tenant-1', 'e2e-tenant-2'],
            roles: ['user'],
          },
          {
            email: 'admin@e2e.test',
            password: 'AdminPassword123!',
            tenants: ['e2e-tenant-1'],
            roles: ['admin'],
          },
        ],
        files: [
          {
            name: 'test-document.pdf',
            size: 1024000,
            contentType: 'application/pdf',
            ownerId: 'e2e-user-1',
            tenantId: 'e2e-tenant-1',
          },
        ],
      },
    });
    
    if (!setupResponse.ok()) {
      throw new Error(`Test setup failed: ${setupResponse.status()}`);
    }
    
    const setupResult = await setupResponse.json();
    console.log('✅ Test data setup complete:', setupResult);
    
    // Store test data for use in tests
    process.env.E2E_TEST_DATA = JSON.stringify(setupResult);
    
  } catch (error) {
    console.error('❌ Global setup failed:', error);
    throw error;
  } finally {
    await context.close();
    await browser.close();
  }
  
  console.log('✅ ADX CORE E2E Test Setup Complete');
}

export default globalSetup;