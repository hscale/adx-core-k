// Cross micro-frontend integration E2E tests
import { test, expect, Page } from '@playwright/test';

// Test data from global setup
const getTestData = () => {
  const testData = process.env.E2E_TEST_DATA;
  return testData ? JSON.parse(testData) : {};
};

// Helper functions
async function loginUser(page: Page, email: string, password: string) {
  await page.goto('/auth/login');
  await page.fill('[data-testid="email-input"]', email);
  await page.fill('[data-testid="password-input"]', password);
  await page.click('[data-testid="login-button"]');
  
  // Wait for successful login
  await page.waitForSelector('[data-testid="dashboard"]', { timeout: 30000 });
}

async function waitForMicroFrontendLoad(page: Page, microfrontendName: string) {
  await page.waitForSelector(`[data-testid="${microfrontendName}-app"]`, { timeout: 15000 });
  await page.waitForLoadState('networkidle');
}

test.describe('Cross Micro-Frontend Integration', () => {
  test.beforeEach(async ({ page }) => {
    // Set up page with error handling
    page.on('pageerror', (error) => {
      console.error('Page error:', error);
    });
    
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        console.error('Console error:', msg.text());
      }
    });
  });

  test('should handle complete user workflow across all micro-frontends', async ({ page }) => {
    const testData = getTestData();
    
    // Step 1: Login through Auth micro-frontend
    await loginUser(page, 'user@e2e.test', 'TestPassword123!');
    
    // Verify we're on the dashboard
    await expect(page.locator('[data-testid="user-name"]')).toHaveText('E2E Test User');
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('E2E Test Tenant 1');
    
    // Step 2: Navigate to Tenant micro-frontend and switch tenant
    await page.click('[data-testid="nav-tenant"]');
    await waitForMicroFrontendLoad(page, 'tenant');
    
    // Switch to second tenant
    await page.selectOption('[data-testid="tenant-switcher"]', 'e2e-tenant-2');
    await page.waitForSelector('[data-testid="tenant-switch-complete"]', { timeout: 30000 });
    
    // Verify tenant switch across all micro-frontends
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('E2E Test Tenant 2');
    
    // Step 3: Navigate to File micro-frontend
    await page.click('[data-testid="nav-files"]');
    await waitForMicroFrontendLoad(page, 'file');
    
    // Verify tenant context is correct in file micro-frontend
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('E2E Test Tenant 2');
    
    // Upload a file
    const fileInput = page.locator('[data-testid="file-upload-input"]');
    await fileInput.setInputFiles({
      name: 'test-upload.txt',
      mimeType: 'text/plain',
      buffer: Buffer.from('This is a test file for E2E testing'),
    });
    
    // Wait for upload workflow to complete
    await page.waitForSelector('[data-testid="upload-complete"]', { timeout: 60000 });
    
    // Verify file appears in list
    await expect(page.locator('[data-testid="file-list"]')).toContainText('test-upload.txt');
    
    // Step 4: Navigate to User micro-frontend
    await page.click('[data-testid="nav-users"]');
    await waitForMicroFrontendLoad(page, 'user');
    
    // Verify tenant context is still correct
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('E2E Test Tenant 2');
    
    // Update user profile
    await page.click('[data-testid="edit-profile-button"]');
    await page.fill('[data-testid="first-name-input"]', 'Updated');
    await page.fill('[data-testid="last-name-input"]', 'Name');
    await page.click('[data-testid="save-profile-button"]');
    
    // Wait for profile update workflow
    await page.waitForSelector('[data-testid="profile-updated"]', { timeout: 30000 });
    
    // Step 5: Navigate to Workflow micro-frontend
    await page.click('[data-testid="nav-workflows"]');
    await waitForMicroFrontendLoad(page, 'workflow');
    
    // Verify we can see all the workflows that were executed
    await expect(page.locator('[data-testid="workflow-list"]')).toContainText('Tenant Switch');
    await expect(page.locator('[data-testid="workflow-list"]')).toContainText('File Upload');
    await expect(page.locator('[data-testid="workflow-list"]')).toContainText('Profile Update');
    
    // Verify all workflows completed successfully
    const workflowItems = page.locator('[data-testid="workflow-item"]');
    const count = await workflowItems.count();
    
    for (let i = 0; i < count; i++) {
      const workflow = workflowItems.nth(i);
      await expect(workflow.locator('[data-testid="workflow-status"]')).toHaveText('Completed');
    }
    
    // Step 6: Navigate back to Shell dashboard
    await page.click('[data-testid="nav-dashboard"]');
    await page.waitForSelector('[data-testid="dashboard"]');
    
    // Verify all changes are reflected in dashboard
    await expect(page.locator('[data-testid="user-name"]')).toHaveText('Updated Name');
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('E2E Test Tenant 2');
    await expect(page.locator('[data-testid="recent-files"]')).toContainText('test-upload.txt');
  });

  test('should handle micro-frontend error boundaries gracefully', async ({ page }) => {
    await loginUser(page, 'user@e2e.test', 'TestPassword123!');
    
    // Simulate a micro-frontend error by navigating to a non-existent route
    await page.goto('/files/non-existent-route');
    
    // Should show error boundary instead of crashing
    await expect(page.locator('[data-testid="error-boundary"]')).toBeVisible();
    await expect(page.locator('[data-testid="error-message"]')).toContainText('Something went wrong');
    
    // Should have a way to recover
    await page.click('[data-testid="retry-button"]');
    
    // Should navigate back to working state
    await waitForMicroFrontendLoad(page, 'file');
    await expect(page.locator('[data-testid="file-list"]')).toBeVisible();
  });

  test('should maintain state consistency across micro-frontend navigation', async ({ page }) => {
    await loginUser(page, 'user@e2e.test', 'TestPassword123!');
    
    // Start a long-running workflow in one micro-frontend
    await page.click('[data-testid="nav-files"]');
    await waitForMicroFrontendLoad(page, 'file');
    
    // Start a large file upload
    const fileInput = page.locator('[data-testid="file-upload-input"]');
    await fileInput.setInputFiles({
      name: 'large-file.zip',
      mimeType: 'application/zip',
      buffer: Buffer.alloc(5 * 1024 * 1024), // 5MB file
    });
    
    // Navigate to workflow micro-frontend while upload is in progress
    await page.click('[data-testid="nav-workflows"]');
    await waitForMicroFrontendLoad(page, 'workflow');
    
    // Should see the upload workflow in progress
    await expect(page.locator('[data-testid="workflow-list"]')).toContainText('File Upload');
    const uploadWorkflow = page.locator('[data-testid="workflow-item"]').filter({ hasText: 'File Upload' });
    await expect(uploadWorkflow.locator('[data-testid="workflow-status"]')).toHaveText('Running');
    
    // Navigate back to files
    await page.click('[data-testid="nav-files"]');
    await waitForMicroFrontendLoad(page, 'file');
    
    // Should still show upload progress
    await expect(page.locator('[data-testid="upload-progress"]')).toBeVisible();
    
    // Wait for upload to complete
    await page.waitForSelector('[data-testid="upload-complete"]', { timeout: 120000 });
    
    // Navigate back to workflows
    await page.click('[data-testid="nav-workflows"]');
    await waitForMicroFrontendLoad(page, 'workflow');
    
    // Should now show completed status
    const completedWorkflow = page.locator('[data-testid="workflow-item"]').filter({ hasText: 'File Upload' });
    await expect(completedWorkflow.locator('[data-testid="workflow-status"]')).toHaveText('Completed');
  });

  test('should handle concurrent operations across micro-frontends', async ({ page }) => {
    await loginUser(page, 'admin@e2e.test', 'AdminPassword123!');
    
    // Open multiple tabs for concurrent operations
    const context = page.context();
    const page2 = await context.newPage();
    const page3 = await context.newPage();
    
    // Navigate each tab to different micro-frontends
    await page.goto('/files');
    await waitForMicroFrontendLoad(page, 'file');
    
    await page2.goto('/users');
    await waitForMicroFrontendLoad(page2, 'user');
    
    await page3.goto('/tenant');
    await waitForMicroFrontendLoad(page3, 'tenant');
    
    // Start concurrent operations
    const operations = [
      // File upload in tab 1
      (async () => {
        const fileInput = page.locator('[data-testid="file-upload-input"]');
        await fileInput.setInputFiles({
          name: 'concurrent-file-1.txt',
          mimeType: 'text/plain',
          buffer: Buffer.from('Concurrent operation 1'),
        });
        await page.waitForSelector('[data-testid="upload-complete"]', { timeout: 60000 });
      })(),
      
      // User creation in tab 2
      (async () => {
        await page2.click('[data-testid="create-user-button"]');
        await page2.fill('[data-testid="user-email-input"]', 'concurrent@e2e.test');
        await page2.fill('[data-testid="user-password-input"]', 'ConcurrentPass123!');
        await page2.click('[data-testid="save-user-button"]');
        await page2.waitForSelector('[data-testid="user-created"]', { timeout: 60000 });
      })(),
      
      // Tenant settings update in tab 3
      (async () => {
        await page3.click('[data-testid="tenant-settings-button"]');
        await page3.fill('[data-testid="tenant-name-input"]', 'Updated Tenant Name');
        await page3.click('[data-testid="save-settings-button"]');
        await page3.waitForSelector('[data-testid="settings-updated"]', { timeout: 60000 });
      })(),
    ];
    
    // Wait for all operations to complete
    await Promise.all(operations);
    
    // Verify all operations completed successfully
    await expect(page.locator('[data-testid="file-list"]')).toContainText('concurrent-file-1.txt');
    await expect(page2.locator('[data-testid="user-list"]')).toContainText('concurrent@e2e.test');
    await expect(page3.locator('[data-testid="tenant-name"]')).toHaveText('Updated Tenant Name');
    
    // Clean up
    await page2.close();
    await page3.close();
  });

  test('should handle offline/online state across micro-frontends', async ({ page, context }) => {
    await loginUser(page, 'user@e2e.test', 'TestPassword123!');
    
    // Navigate to files micro-frontend
    await page.click('[data-testid="nav-files"]');
    await waitForMicroFrontendLoad(page, 'file');
    
    // Go offline
    await context.setOffline(true);
    
    // Try to upload a file while offline
    const fileInput = page.locator('[data-testid="file-upload-input"]');
    await fileInput.setInputFiles({
      name: 'offline-file.txt',
      mimeType: 'text/plain',
      buffer: Buffer.from('Offline test'),
    });
    
    // Should show offline indicator
    await expect(page.locator('[data-testid="offline-indicator"]')).toBeVisible();
    await expect(page.locator('[data-testid="upload-queued"]')).toBeVisible();
    
    // Navigate to other micro-frontends while offline
    await page.click('[data-testid="nav-users"]');
    await waitForMicroFrontendLoad(page, 'user');
    
    // Should still show offline indicator
    await expect(page.locator('[data-testid="offline-indicator"]')).toBeVisible();
    
    // Go back online
    await context.setOffline(false);
    
    // Should automatically sync queued operations
    await page.waitForSelector('[data-testid="online-indicator"]', { timeout: 10000 });
    
    // Navigate back to files
    await page.click('[data-testid="nav-files"]');
    await waitForMicroFrontendLoad(page, 'file');
    
    // File should now be uploaded
    await expect(page.locator('[data-testid="file-list"]')).toContainText('offline-file.txt');
  });

  test('should handle module federation loading failures gracefully', async ({ page }) => {
    await loginUser(page, 'user@e2e.test', 'TestPassword123!');
    
    // Simulate module federation failure by blocking requests to a micro-frontend
    await page.route('**/assets/remoteEntry.js', route => route.abort());
    
    // Try to navigate to a micro-frontend
    await page.click('[data-testid="nav-files"]');
    
    // Should show loading fallback
    await expect(page.locator('[data-testid="microfrontend-loading"]')).toBeVisible();
    
    // Should eventually show error fallback
    await expect(page.locator('[data-testid="microfrontend-error"]')).toBeVisible({ timeout: 30000 });
    
    // Should have retry option
    await expect(page.locator('[data-testid="retry-load-button"]')).toBeVisible();
    
    // Clear the route block and retry
    await page.unroute('**/assets/remoteEntry.js');
    await page.click('[data-testid="retry-load-button"]');
    
    // Should successfully load the micro-frontend
    await waitForMicroFrontendLoad(page, 'file');
    await expect(page.locator('[data-testid="file-list"]')).toBeVisible();
  });

  test('should maintain accessibility across all micro-frontends', async ({ page }) => {
    await loginUser(page, 'user@e2e.test', 'TestPassword123!');
    
    const microFrontends = ['tenant', 'files', 'users', 'workflows'];
    
    for (const mf of microFrontends) {
      await page.click(`[data-testid="nav-${mf}"]`);
      await waitForMicroFrontendLoad(page, mf);
      
      // Check for basic accessibility requirements
      await expect(page.locator('h1')).toBeVisible(); // Page should have a main heading
      await expect(page.locator('[role="main"]')).toBeVisible(); // Should have main landmark
      
      // Check keyboard navigation
      await page.keyboard.press('Tab');
      const focusedElement = await page.locator(':focus').first();
      await expect(focusedElement).toBeVisible();
      
      // Check for skip links
      await page.keyboard.press('Tab');
      const skipLink = page.locator('[data-testid="skip-to-content"]');
      if (await skipLink.isVisible()) {
        await expect(skipLink).toHaveAttribute('href', '#main-content');
      }
    }
  });
});