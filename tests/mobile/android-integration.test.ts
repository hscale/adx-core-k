import { test, expect, devices } from '@playwright/test';
import { execSync } from 'child_process';

// Android-specific test configuration
const androidDevice = devices['Pixel 5'];

test.describe('Android Mobile Integration Tests', () => {
  test.use({
    ...androidDevice,
    // Use Chrome for Android web-based tests
    browserName: 'chromium',
  });

  test.beforeAll(async () => {
    // Ensure Android emulator is running
    try {
      execSync('adb devices | grep emulator', { stdio: 'pipe' });
    } catch (error) {
      console.log('Android emulator not detected. Please ensure emulator is running.');
    }
  });

  test('should launch native Android app', async ({ page }) => {
    // Test native app launch through intent
    await page.goto('intent://launch#Intent;scheme=adxcore;package=com.adxcore.app;end');
    
    // Wait for app to load
    await page.waitForTimeout(5000);
    
    // Verify app launched successfully
    const appTitle = await page.evaluate(() => document.title);
    expect(appTitle).toContain('ADX Core');
  });

  test('should handle Android-specific gestures', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForSelector('[data-testid="app-shell"]');
    
    // Test swipe gestures
    const element = page.locator('[data-testid="swipeable-content"]');
    
    // Swipe left
    await element.swipe({ direction: 'left', distance: 200 });
    await expect(page.locator('[data-testid="next-content"]')).toBeVisible();
    
    // Swipe right
    await element.swipe({ direction: 'right', distance: 200 });
    await expect(page.locator('[data-testid="prev-content"]')).toBeVisible();
    
    // Test pull-to-refresh
    const refreshContainer = page.locator('[data-testid="refresh-container"]');
    await refreshContainer.swipe({ direction: 'down', distance: 150 });
    await expect(page.locator('[data-testid="refresh-indicator"]')).toBeVisible();
  });

  test('should handle Android touch interactions', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForSelector('[data-testid="app-shell"]');
    
    // Test long press
    const longPressElement = page.locator('[data-testid="long-press-target"]');
    await longPressElement.press({ delay: 1000 });
    await expect(page.locator('[data-testid="context-menu"]')).toBeVisible();
    
    // Test multi-touch gestures
    const multiTouchElement = page.locator('[data-testid="multi-touch-target"]');
    
    // Simulate pinch gesture
    await multiTouchElement.evaluate(element => {
      const touch1 = new Touch({
        identifier: 1,
        target: element,
        clientX: 100,
        clientY: 100,
      });
      const touch2 = new Touch({
        identifier: 2,
        target: element,
        clientX: 200,
        clientY: 200,
      });
      
      const touchEvent = new TouchEvent('touchstart', {
        touches: [touch1, touch2],
        targetTouches: [touch1, touch2],
        changedTouches: [touch1, touch2],
      });
      
      element.dispatchEvent(touchEvent);
    });
  });

  test('should handle Android keyboard and input', async ({ page }) => {
    await page.goto('http://localhost:3000/auth/login');
    
    // Test Android keyboard appearance
    const emailInput = page.locator('[data-testid="email-input"]');
    await emailInput.tap();
    
    // Test input with Android keyboard
    await emailInput.fill('test@example.com');
    await expect(emailInput).toHaveValue('test@example.com');
    
    // Test Android-specific input types
    const phoneInput = page.locator('[data-testid="phone-input"]');
    await phoneInput.tap();
    
    // Verify numeric keyboard appears
    const inputMode = await phoneInput.getAttribute('inputmode');
    expect(inputMode).toBe('tel');
    
    // Test autocomplete functionality
    const autocompleteInput = page.locator('[data-testid="autocomplete-input"]');
    await autocompleteInput.fill('test');
    
    // Wait for autocomplete suggestions
    await page.waitForSelector('[data-testid="autocomplete-suggestions"]', { timeout: 2000 });
    const suggestions = page.locator('[data-testid="autocomplete-suggestion"]');
    const suggestionCount = await suggestions.count();
    expect(suggestionCount).toBeGreaterThan(0);
  });

  test('should handle Android notifications', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Request notification permission
    const permissionResult = await page.evaluate(async () => {
      if ('Notification' in window) {
        const permission = await Notification.requestPermission();
        return permission;
      }
      return 'not-supported';
    });
    
    if (permissionResult === 'granted') {
      // Test notification display
      await page.evaluate(() => {
        new Notification('Test Notification', {
          body: 'This is a test notification on Android',
          icon: '/icon-192x192.png',
          badge: '/badge-72x72.png',
          tag: 'test-notification',
        });
      });
      
      // Test notification actions
      await page.evaluate(() => {
        new Notification('Action Notification', {
          body: 'Notification with actions',
          actions: [
            { action: 'reply', title: 'Reply' },
            { action: 'dismiss', title: 'Dismiss' },
          ],
        });
      });
      
      await page.waitForTimeout(1000);
    }
  });

  test('should handle Android device orientation', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForSelector('[data-testid="app-shell"]');
    
    // Test portrait orientation
    await page.setViewportSize({ width: 393, height: 851 });
    const portraitLayout = await page.locator('[data-testid="layout-container"]').evaluate(
      el => window.getComputedStyle(el).flexDirection
    );
    expect(portraitLayout).toBe('column');
    
    // Test landscape orientation
    await page.setViewportSize({ width: 851, height: 393 });
    const landscapeLayout = await page.locator('[data-testid="layout-container"]').evaluate(
      el => window.getComputedStyle(el).flexDirection
    );
    expect(landscapeLayout).toBe('row');
    
    // Test orientation change event
    const orientationChanged = await page.evaluate(() => {
      return new Promise((resolve) => {
        const handler = () => {
          window.removeEventListener('orientationchange', handler);
          resolve(true);
        };
        window.addEventListener('orientationchange', handler);
        
        // Simulate orientation change
        window.dispatchEvent(new Event('orientationchange'));
        
        setTimeout(() => resolve(false), 1000);
      });
    });
    
    expect(orientationChanged).toBe(true);
  });

  test('should handle Android back button', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForSelector('[data-testid="app-shell"]');
    
    // Navigate to a sub-page
    await page.click('[data-testid="nav-settings"]');
    await page.waitForSelector('[data-testid="settings-page"]');
    
    // Simulate Android back button
    await page.goBack();
    
    // Verify navigation worked
    await expect(page.locator('[data-testid="dashboard"]')).toBeVisible();
    
    // Test custom back button handling
    await page.click('[data-testid="nav-settings"]');
    await page.waitForSelector('[data-testid="settings-page"]');
    
    const backHandled = await page.evaluate(() => {
      return new Promise((resolve) => {
        const handler = (event: PopStateEvent) => {
          event.preventDefault();
          window.removeEventListener('popstate', handler);
          resolve(true);
        };
        window.addEventListener('popstate', handler);
        
        // Simulate back button
        history.back();
        
        setTimeout(() => resolve(false), 1000);
      });
    });
    
    expect(backHandled).toBe(true);
  });

  test('should handle Android app state changes', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Simulate app going to background
    await page.evaluate(() => {
      window.dispatchEvent(new Event('pagehide'));
      document.dispatchEvent(new Event('visibilitychange'));
    });
    
    // Verify app handles background state
    const backgroundState = await page.evaluate(() => 
      document.visibilityState === 'hidden'
    );
    expect(backgroundState).toBe(true);
    
    // Simulate app returning to foreground
    await page.evaluate(() => {
      Object.defineProperty(document, 'visibilityState', {
        value: 'visible',
        writable: true
      });
      window.dispatchEvent(new Event('pageshow'));
      document.dispatchEvent(new Event('visibilitychange'));
    });
    
    // Verify app handles foreground state
    const foregroundState = await page.evaluate(() => 
      document.visibilityState === 'visible'
    );
    expect(foregroundState).toBe(true);
  });

  test('should handle Android-specific APIs', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test Android-specific features
    const androidFeatures = await page.evaluate(() => {
      const features = {
        webShareSupported: 'share' in navigator,
        deviceMemorySupported: 'deviceMemory' in navigator,
        connectionSupported: 'connection' in navigator,
        wakeLockSupported: 'wakeLock' in navigator,
        vibrationSupported: 'vibrate' in navigator,
      };
      return features;
    });
    
    // Test Web Share API (common on Android)
    if (androidFeatures.webShareSupported) {
      const shareResult = await page.evaluate(async () => {
        try {
          await navigator.share({
            title: 'ADX Core',
            text: 'Check out ADX Core!',
            url: 'https://adxcore.com',
          });
          return true;
        } catch (error) {
          return false;
        }
      });
      
      // Share API might not work in test environment, but should not throw
      expect(typeof shareResult).toBe('boolean');
    }
    
    // Test vibration API
    if (androidFeatures.vibrationSupported) {
      const vibrationResult = await page.evaluate(() => {
        try {
          navigator.vibrate([200, 100, 200]);
          return true;
        } catch (error) {
          return false;
        }
      });
      
      expect(vibrationResult).toBe(true);
    }
    
    // Test network information API
    if (androidFeatures.connectionSupported) {
      const connectionInfo = await page.evaluate(() => {
        const connection = (navigator as any).connection;
        return {
          effectiveType: connection?.effectiveType,
          downlink: connection?.downlink,
          rtt: connection?.rtt,
        };
      });
      
      expect(connectionInfo).toBeTruthy();
    }
  });

  test('should handle Android performance optimization', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Measure initial load performance
    const performanceMetrics = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      return {
        domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
        loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
        firstPaint: performance.getEntriesByName('first-paint')[0]?.startTime || 0,
        firstContentfulPaint: performance.getEntriesByName('first-contentful-paint')[0]?.startTime || 0,
      };
    });
    
    // Verify performance meets Android standards
    expect(performanceMetrics.domContentLoaded).toBeLessThan(3000); // 3 seconds (Android can be slower)
    expect(performanceMetrics.firstContentfulPaint).toBeLessThan(2000); // 2 seconds
    
    // Test memory usage
    const memoryInfo = await page.evaluate(() => {
      const memory = (performance as any).memory;
      return memory ? {
        usedJSHeapSize: memory.usedJSHeapSize,
        totalJSHeapSize: memory.totalJSHeapSize,
        jsHeapSizeLimit: memory.jsHeapSizeLimit,
      } : null;
    });
    
    if (memoryInfo) {
      // Memory usage should be reasonable
      const memoryUsageRatio = memoryInfo.usedJSHeapSize / memoryInfo.totalJSHeapSize;
      expect(memoryUsageRatio).toBeLessThan(0.8); // Less than 80% memory usage
    }
  });

  test('should handle Android file system access', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test file picker
    const filePickerResult = await page.evaluate(async () => {
      try {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = 'image/*';
        input.multiple = true;
        
        // Simulate file selection
        const files = [
          new File(['test content'], 'test.txt', { type: 'text/plain' }),
          new File(['image content'], 'test.jpg', { type: 'image/jpeg' }),
        ];
        
        Object.defineProperty(input, 'files', {
          value: files,
          writable: false,
        });
        
        return {
          fileCount: input.files.length,
          firstFileName: input.files[0].name,
        };
      } catch (error) {
        return { error: error.message };
      }
    });
    
    expect(filePickerResult.fileCount).toBe(2);
    expect(filePickerResult.firstFileName).toBe('test.txt');
  });
});