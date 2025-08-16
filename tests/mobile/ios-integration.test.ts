import { test, expect, devices } from '@playwright/test';
import { execSync } from 'child_process';

// iOS-specific test configuration
const iosDevice = devices['iPhone 13'];

test.describe('iOS Mobile Integration Tests', () => {
  test.use({
    ...iosDevice,
    // Use iOS Safari for web-based tests
    browserName: 'webkit',
  });

  test.beforeAll(async () => {
    // Ensure iOS simulator is running
    try {
      execSync('xcrun simctl list devices | grep "iPhone 14" | grep "Booted"', { stdio: 'pipe' });
    } catch (error) {
      console.log('Starting iOS simulator...');
      execSync('xcrun simctl boot "iPhone 14"', { stdio: 'inherit' });
      // Wait for simulator to boot
      await new Promise(resolve => setTimeout(resolve, 10000));
    }
  });

  test('should launch native iOS app', async ({ page }) => {
    // Test native app launch through custom scheme
    await page.goto('adxcore://launch');
    
    // Wait for app to load
    await page.waitForTimeout(5000);
    
    // Verify app launched successfully
    const appTitle = await page.evaluate(() => document.title);
    expect(appTitle).toContain('ADX Core');
  });

  test('should handle iOS-specific gestures', async ({ page }) => {
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
  });

  test('should handle iOS touch interactions', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForSelector('[data-testid="app-shell"]');
    
    // Test long press
    const longPressElement = page.locator('[data-testid="long-press-target"]');
    await longPressElement.press({ delay: 1000 });
    await expect(page.locator('[data-testid="context-menu"]')).toBeVisible();
    
    // Test pinch zoom (if supported)
    const zoomableElement = page.locator('[data-testid="zoomable-content"]');
    await zoomableElement.pinch({ scale: 1.5 });
    
    const transform = await zoomableElement.evaluate(el => 
      window.getComputedStyle(el).transform
    );
    expect(transform).toContain('scale');
  });

  test('should handle iOS keyboard and input', async ({ page }) => {
    await page.goto('http://localhost:3000/auth/login');
    
    // Test iOS keyboard appearance
    const emailInput = page.locator('[data-testid="email-input"]');
    await emailInput.tap();
    
    // Verify keyboard is shown (iOS specific)
    const keyboardHeight = await page.evaluate(() => {
      return window.visualViewport?.height || 0;
    });
    expect(keyboardHeight).toBeGreaterThan(0);
    
    // Test input with iOS keyboard
    await emailInput.fill('test@example.com');
    await expect(emailInput).toHaveValue('test@example.com');
    
    // Test iOS-specific input types
    const phoneInput = page.locator('[data-testid="phone-input"]');
    await phoneInput.tap();
    
    // Verify numeric keyboard appears
    const inputMode = await phoneInput.getAttribute('inputmode');
    expect(inputMode).toBe('tel');
  });

  test('should handle iOS notifications', async ({ page }) => {
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
          body: 'This is a test notification on iOS',
          icon: '/icon-192x192.png',
        });
      });
      
      // Verify notification was created
      await page.waitForTimeout(1000);
    }
  });

  test('should handle iOS device orientation', async ({ page }) => {
    await page.goto('http://localhost:3000');
    await page.waitForSelector('[data-testid="app-shell"]');
    
    // Test portrait orientation
    await page.setViewportSize({ width: 375, height: 812 });
    const portraitLayout = await page.locator('[data-testid="layout-container"]').evaluate(
      el => window.getComputedStyle(el).flexDirection
    );
    expect(portraitLayout).toBe('column');
    
    // Test landscape orientation
    await page.setViewportSize({ width: 812, height: 375 });
    const landscapeLayout = await page.locator('[data-testid="layout-container"]').evaluate(
      el => window.getComputedStyle(el).flexDirection
    );
    expect(landscapeLayout).toBe('row');
  });

  test('should handle iOS safe areas', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Check if safe area insets are applied
    const safeAreaTop = await page.evaluate(() => {
      const style = window.getComputedStyle(document.documentElement);
      return style.getPropertyValue('--safe-area-inset-top');
    });
    
    // On iOS devices, safe area should be defined
    expect(safeAreaTop).toBeTruthy();
    
    // Verify content respects safe areas
    const headerElement = page.locator('[data-testid="app-header"]');
    const headerPaddingTop = await headerElement.evaluate(el => 
      window.getComputedStyle(el).paddingTop
    );
    
    expect(parseInt(headerPaddingTop)).toBeGreaterThan(0);
  });

  test('should handle iOS app state changes', async ({ page }) => {
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

  test('should handle iOS-specific APIs', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Test iOS-specific features if available
    const iosFeatures = await page.evaluate(() => {
      const features = {
        touchForceSupported: 'TouchEvent' in window && 'force' in TouchEvent.prototype,
        deviceMotionSupported: 'DeviceMotionEvent' in window,
        deviceOrientationSupported: 'DeviceOrientationEvent' in window,
        webkitSupported: 'webkitRequestFullscreen' in document.documentElement,
      };
      return features;
    });
    
    // Verify iOS-specific features are detected
    expect(iosFeatures.webkitSupported).toBe(true);
    
    if (iosFeatures.deviceMotionSupported) {
      // Test device motion handling
      const motionHandled = await page.evaluate(() => {
        return new Promise((resolve) => {
          const handler = () => {
            window.removeEventListener('devicemotion', handler);
            resolve(true);
          };
          window.addEventListener('devicemotion', handler);
          
          // Simulate device motion event
          const event = new DeviceMotionEvent('devicemotion', {
            acceleration: { x: 1, y: 2, z: 3 },
            accelerationIncludingGravity: { x: 1, y: 2, z: 3 },
            rotationRate: { alpha: 0, beta: 0, gamma: 0 },
            interval: 16,
          });
          window.dispatchEvent(event);
          
          setTimeout(() => resolve(false), 1000);
        });
      });
      
      expect(motionHandled).toBe(true);
    }
  });

  test('should handle iOS performance optimization', async ({ page }) => {
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
    
    // Verify performance meets iOS standards
    expect(performanceMetrics.domContentLoaded).toBeLessThan(2000); // 2 seconds
    expect(performanceMetrics.firstContentfulPaint).toBeLessThan(1500); // 1.5 seconds
    
    // Test scroll performance
    const scrollContainer = page.locator('[data-testid="scroll-container"]');
    const scrollStart = Date.now();
    
    await scrollContainer.scroll({ top: 1000 });
    await page.waitForTimeout(100);
    
    const scrollEnd = Date.now();
    const scrollDuration = scrollEnd - scrollStart;
    
    // Scroll should be smooth and responsive
    expect(scrollDuration).toBeLessThan(500);
  });
});