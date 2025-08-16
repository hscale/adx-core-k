# ADX Core Cross-Platform Testing

This directory contains comprehensive cross-platform testing infrastructure for ADX Core, covering desktop applications, mobile applications, and web-based micro-frontends.

## Test Structure

```
tests/
├── desktop/                    # Desktop application tests (Tauri)
│   ├── tauri-integration.test.ts
│   ├── global-setup.ts
│   └── global-teardown.ts
├── mobile/                     # Mobile application tests
│   ├── ios-integration.test.ts
│   └── android-integration.test.ts
├── cross-platform/            # Cross-platform compatibility tests
│   └── compatibility.test.ts
├── performance/               # Performance tests across platforms
│   ├── global-setup.ts
│   └── global-teardown.ts
├── e2e/                       # End-to-end tests
│   └── specs/
│       └── cross-microfrontend.spec.ts
└── security/                  # Security tests
    └── security_tests.rs
```

## Test Types

### 1. Desktop Tests (`tests/desktop/`)

Tests for Tauri-based desktop applications across Windows, macOS, and Linux.

**Features tested:**
- Native window operations (minimize, maximize, close)
- File system access through Tauri APIs
- Native notifications
- Deep linking and custom protocols
- System theme integration
- Multi-window scenarios
- Platform-specific shortcuts

**Run desktop tests:**
```bash
npm run test:desktop:integration
npm run test:desktop:e2e
```

### 2. Mobile Tests (`tests/mobile/`)

Tests for mobile applications on iOS and Android platforms.

**Features tested:**
- Touch gestures and interactions
- Mobile-specific UI patterns
- Device orientation changes
- Native mobile APIs
- Performance on mobile devices
- App state management (background/foreground)

**Run mobile tests:**
```bash
npm run test:mobile:ios
npm run test:mobile:android
```

### 3. Cross-Platform Compatibility Tests (`tests/cross-platform/`)

Comprehensive compatibility testing across all supported platforms.

**Features tested:**
- JavaScript API compatibility
- CSS feature support
- Web API availability
- Performance metrics comparison
- Responsive design validation
- Accessibility compliance

**Run compatibility tests:**
```bash
npm run test:cross-platform:compatibility
```

### 4. Performance Tests (`tests/performance/`)

Performance testing across different platforms and devices.

**Metrics measured:**
- Page load times
- First Contentful Paint (FCP)
- Time to Interactive (TTI)
- Memory usage
- Bundle sizes
- Network performance

**Run performance tests:**
```bash
npm run test:performance:cross-platform
```

## Configuration Files

### Playwright Configurations

- `playwright.config.desktop.ts` - Desktop-specific test configuration
- `playwright.config.performance.ts` - Performance test configuration
- `playwright.config.ts` - Default configuration for web tests

### Test Scripts

- `scripts/test-cross-platform.sh` - Main cross-platform test runner
- `scripts/deployment-health-check.sh` - Post-deployment health checks
- `scripts/deployment-rollback.sh` - Automated rollback procedures

## Running Tests

### All Cross-Platform Tests
```bash
npm run test:cross-platform
```

### Specific Test Types
```bash
# Desktop tests only
npm run test:cross-platform desktop

# Mobile tests only
npm run test:cross-platform mobile

# Compatibility tests only
npm run test:cross-platform compatibility

# Performance tests only
npm run test:cross-platform performance
```

### Platform-Specific Tests
```bash
# Test specific platforms
./scripts/test-cross-platform.sh all windows,macos
./scripts/test-cross-platform.sh mobile ios
./scripts/test-cross-platform.sh desktop linux
```

## CI/CD Integration

### GitHub Actions Workflows

- `.github/workflows/tauri-desktop-tests.yml` - Desktop testing across platforms
- `.github/workflows/mobile-tests.yml` - Mobile application testing
- `.github/workflows/deploy-microservices.yml` - Backend service deployment
- `.github/workflows/deploy-microfrontends.yml` - Frontend deployment
- `.github/workflows/automated-rollback.yml` - Automated rollback procedures

### Test Execution in CI

Tests are automatically executed on:
- Push to main/develop branches
- Pull requests
- Manual workflow dispatch
- Scheduled runs (nightly)

## Test Reports

### Generated Reports

- **Compatibility Report**: `reports/cross-platform-compatibility-report.html`
- **Performance Report**: `test-results/performance/performance-report.html`
- **Mobile Compatibility**: `reports/mobile-compatibility-report.html`
- **Desktop Test Results**: `test-results/desktop/`

### Report Contents

1. **Feature Compatibility Matrix**: Shows which features are supported on each platform
2. **Performance Metrics**: Load times, rendering performance, memory usage
3. **Error Summary**: Failed tests and issues found
4. **Recommendations**: Actionable suggestions for improvements

## Prerequisites

### Desktop Testing
- Rust and Cargo
- Tauri CLI: `cargo install tauri-cli`
- Platform-specific dependencies:
  - **Windows**: Visual Studio Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: GTK development libraries

### Mobile Testing
- **iOS**: Xcode, iOS Simulator
- **Android**: Android SDK, Android Emulator
- Node.js 18+
- Playwright browsers

### General Requirements
- Node.js 18+
- npm or yarn
- Playwright: `npx playwright install`

## Environment Variables

```bash
# Test configuration
HEADLESS=true                    # Run tests in headless mode
PLATFORM=all                    # Target platforms
TEST_TIMEOUT=60000              # Test timeout in milliseconds

# CI/CD configuration
CI=true                         # Enable CI-specific settings
ARTIFACTS_PATH=test-results/    # Test artifacts location
SLACK_WEBHOOK_URL=...          # Slack notifications
NOTIFICATION_EMAIL=...         # Email notifications

# Deployment configuration
AWS_ACCESS_KEY_ID=...          # AWS credentials
AWS_SECRET_ACCESS_KEY=...      # AWS credentials
KUBE_CONFIG=...                # Kubernetes configuration
CLOUDFRONT_DISTRIBUTION_ID=... # CloudFront distribution
```

## Troubleshooting

### Common Issues

1. **Desktop app not found**
   ```bash
   # Build the desktop app first
   cargo tauri build --debug
   ```

2. **Mobile emulator not running**
   ```bash
   # Start iOS simulator
   xcrun simctl boot "iPhone 14"
   
   # Start Android emulator
   emulator -avd Pixel_5_API_33
   ```

3. **Playwright browsers not installed**
   ```bash
   npx playwright install
   ```

4. **Permission denied on scripts**
   ```bash
   chmod +x scripts/*.sh
   ```

### Debug Mode

Run tests with debug output:
```bash
DEBUG=pw:api npm run test:cross-platform
HEADLESS=false npm run test:desktop:e2e
```

### Test Data Cleanup

Clean up test artifacts:
```bash
rm -rf test-results/
rm -rf reports/
rm -rf screenshots/
```

## Contributing

### Adding New Tests

1. Create test files in the appropriate directory
2. Follow the existing naming convention
3. Add test scripts to `package.json`
4. Update CI/CD workflows if needed
5. Document any new dependencies

### Test Guidelines

- Use descriptive test names
- Include proper error handling
- Add appropriate timeouts
- Clean up resources after tests
- Generate meaningful reports
- Follow accessibility best practices

### Platform-Specific Considerations

- **Desktop**: Test native integrations and window management
- **Mobile**: Focus on touch interactions and mobile-specific APIs
- **Web**: Ensure cross-browser compatibility
- **Performance**: Test on various device capabilities

## Monitoring and Alerting

### Deployment Monitoring

- Health checks after deployment
- Performance regression detection
- Error rate monitoring
- Rollback triggers

### Alert Channels

- Slack notifications for test failures
- Email alerts for critical issues
- GitHub issues for rollback failures
- Dashboard updates for metrics

This comprehensive testing infrastructure ensures ADX Core works reliably across all supported platforms and provides early detection of issues through automated testing and monitoring.