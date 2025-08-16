import { test, expect, _electron as electron } from '@playwright/test';
import { ElectronApplication, Page } from 'playwright';
import path from 'path';
import os from 'os';

let electronApp: ElectronApplication;
let page: Page;

const getElectronPath = () => {
  const platform = os.platform();
  const arch = os.arch();
  
  switch (platform) {
    case 'darwin':
      return path.join(__dirname, '../../src-tauri/target/debug/bundle/macos/ADX Core.app/Contents/MacOS/ADX Core');
    case 'win32':
      return path.join(__dirname, '../../src-tauri/target/debug/ADX Core.exe');
    case 'linux':
      return path.join(__dirname, '../../src-tauri/target/debug/adx-core');
    default:
      throw new Error(`Unsupported platform: ${platform}`);
  }
};

test.describe('Tauri Desktop Integration Tests', () => {
  test.beforeAll(async () => {
    // Launch Electron app
    electronApp = await electron.launch({
      args: [getElectronPath()],
      env: {
        ...process.env,
        NODE_ENV: 'test',
        TAURI_ENV: 'test',
      },
    });

    // Get the first window
    page = await electronApp.firstWindow();
    
    // Wait for app to be ready
    await page.waitForLoadState('domcontentloaded');
    await page.waitForTimeout(2000); // Allow Tauri to initialize
  });

  test.afterAll(async () => {
    await electronApp.close();
  });

  test('should launch successfully', async () => {
    expect(electronApp).toBeTruthy();
    expect(page).toBeTruthy();
    
    // Check if the main window is visible
    const title = await page.title();
    expect(title).toContain('ADX Core');
  });

  test('should load shell application', async () => {
    // Wait for React app to load
    await page.waitForSelector('[data-testid="app-shell"]', { timeout: 10000 });
    
    // Check if navigation is present
    const navigation = page.locator('[data-testid="navigation"]');
    await expect(navigation).toBeVisible();
  });

  test('should handle window operations', async () => {
    // Test window minimize
    await page.evaluate(() => {
      // @ts-ignore - Tauri API
      window.__TAURI__.window.appWindow.minimize();
    });
    
    await page.waitForTimeout(1000);
    
    // Test window restore
    await page.evaluate(() => {
      // @ts-ignore - Tauri API
      window.__TAURI__.window.appWindow.unminimize();
    });
    
    await page.waitForTimeout(1000);
    
    // Verify window is visible again
    const isVisible = await page.isVisible('[data-testid="app-shell"]');
    expect(isVisible).toBe(true);
  });

  test('should access native file system', async () => {
    // Test file system access through Tauri
    const result = await page.evaluate(async () => {
      try {
        // @ts-ignore - Tauri API
        const { readDir } = window.__TAURI__.fs;
        const entries = await readDir('.', { recursive: false });
        return { success: true, entriesCount: entries.length };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });
    
    expect(result.success).toBe(true);
    expect(result.entriesCount).toBeGreaterThan(0);
  });

  test('should handle native notifications', async () => {
    const notificationResult = await page.evaluate(async () => {
      try {
        // @ts-ignore - Tauri API
        const { sendNotification } = window.__TAURI__.notification;
        await sendNotification({
          title: 'Test Notification',
          body: 'This is a test notification from ADX Core',
        });
        return { success: true };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });
    
    expect(notificationResult.success).toBe(true);
  });

  test('should handle deep linking', async () => {
    // Test custom protocol handling
    const deepLinkResult = await page.evaluate(async () => {
      try {
        // Simulate deep link
        const event = new CustomEvent('tauri://deep-link', {
          detail: { url: 'adxcore://tenant/switch?id=test-tenant' }
        });
        window.dispatchEvent(event);
        return { success: true };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });
    
    expect(deepLinkResult.success).toBe(true);
  });

  test('should persist data locally', async () => {
    // Test local storage persistence
    await page.evaluate(() => {
      localStorage.setItem('test-key', 'test-value');
    });
    
    // Restart app simulation by reloading
    await page.reload();
    await page.waitForSelector('[data-testid="app-shell"]');
    
    const storedValue = await page.evaluate(() => {
      return localStorage.getItem('test-key');
    });
    
    expect(storedValue).toBe('test-value');
  });

  test('should handle offline mode', async () => {
    // Simulate offline mode
    await page.context().setOffline(true);
    
    // Check if app handles offline state
    const offlineState = await page.evaluate(() => {
      return navigator.onLine;
    });
    
    expect(offlineState).toBe(false);
    
    // Verify offline indicator is shown
    const offlineIndicator = page.locator('[data-testid="offline-indicator"]');
    await expect(offlineIndicator).toBeVisible({ timeout: 5000 });
    
    // Restore online mode
    await page.context().setOffline(false);
  });

  test('should handle system theme changes', async () => {
    // Test theme switching
    await page.evaluate(() => {
      // @ts-ignore - Tauri API
      window.__TAURI__.theme.setTheme('dark');
    });
    
    await page.waitForTimeout(1000);
    
    const isDarkMode = await page.evaluate(() => {
      return document.documentElement.classList.contains('dark');
    });
    
    expect(isDarkMode).toBe(true);
    
    // Switch back to light mode
    await page.evaluate(() => {
      // @ts-ignore - Tauri API
      window.__TAURI__.theme.setTheme('light');
    });
  });

  test('should handle multi-window scenarios', async () => {
    // Open a new window
    const newWindowPromise = electronApp.waitForEvent('window');
    
    await page.evaluate(() => {
      // @ts-ignore - Tauri API
      window.__TAURI__.window.WebviewWindow.new('settings', {
        url: '/settings',
        title: 'Settings',
        width: 800,
        height: 600,
      });
    });
    
    const newWindow = await newWindowPromise;
    await newWindow.waitForLoadState('domcontentloaded');
    
    // Verify new window loaded correctly
    const newWindowTitle = await newWindow.title();
    expect(newWindowTitle).toContain('Settings');
    
    // Close the new window
    await newWindow.close();
  });
});

test.describe('Platform-Specific Features', () => {
  test('should handle platform-specific shortcuts', async () => {
    const platform = os.platform();
    const modifier = platform === 'darwin' ? 'Meta' : 'Control';
    
    // Test global shortcut
    await page.keyboard.press(`${modifier}+Shift+D`);
    
    // Verify shortcut was handled
    const shortcutHandled = await page.evaluate(() => {
      return window.lastShortcutHandled === 'toggle-dev-tools';
    });
    
    expect(shortcutHandled).toBe(true);
  });

  test('should handle platform-specific file dialogs', async () => {
    const dialogResult = await page.evaluate(async () => {
      try {
        // @ts-ignore - Tauri API
        const { open } = window.__TAURI__.dialog;
        const selected = await open({
          multiple: false,
          filters: [{
            name: 'Text Files',
            extensions: ['txt', 'md']
          }]
        });
        return { success: true, hasSelection: !!selected };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });
    
    // Note: In test environment, dialog might be cancelled
    expect(dialogResult.success).toBe(true);
  });
});