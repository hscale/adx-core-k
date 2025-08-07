# ADX CORE Frontend

Universal cross-platform frontend for the ADX CORE platform, built with React, TypeScript, and Tauri 2.0.

## Features

- **Universal Cross-Platform**: Web, Desktop (Windows, macOS, Linux), and Mobile (iOS, Android)
- **Modern Tech Stack**: React 18, TypeScript, TailwindCSS, Vite
- **Multi-tenant Architecture**: Full tenant isolation and branding support
- **Internationalization**: Support for 6 languages (EN, ES, FR, DE, JA, ZH)
- **Dark/Light Theme**: Automatic system detection with manual override
- **Real-time Updates**: WebSocket integration for live data
- **Offline Support**: Progressive Web App capabilities
- **Enterprise Security**: JWT authentication, role-based access control

## Quick Start

### Web Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

### Desktop Development

```bash
# Install Tauri CLI
cargo install @tauri-apps/cli@next

# Start desktop development
npm run tauri:dev

# Build desktop app
npm run tauri:build
```

### Mobile Development

```bash
# Add mobile targets
npm run tauri add android
npm run tauri add ios

# Start mobile development
npm run tauri android dev
npm run tauri ios dev

# Build mobile apps
npm run tauri android build
npm run tauri ios build
```

## Project Structure

```
frontend/
├── src/
│   ├── components/          # Reusable UI components
│   │   └── ui/             # Base UI components
│   ├── contexts/           # React contexts
│   ├── hooks/              # Custom React hooks
│   ├── i18n/               # Internationalization
│   │   └── locales/        # Translation files
│   ├── layouts/            # Page layouts
│   ├── pages/              # Application pages
│   ├── services/           # API services
│   ├── stores/             # State management
│   ├── types/              # TypeScript types
│   └── utils/              # Utility functions
├── src-tauri/              # Tauri desktop/mobile app
├── public/                 # Static assets
└── dist/                   # Build output
```

## Available Scripts

### Development
- `npm run dev` - Start web development server
- `npm run dev:web` - Start web-specific development
- `npm run dev:desktop` - Start desktop-specific development
- `npm run dev:mobile` - Start mobile-specific development

### Building
- `npm run build` - Build for web production
- `npm run build:web` - Build web version
- `npm run build:desktop` - Build desktop version
- `npm run build:mobile` - Build mobile version

### Tauri (Desktop/Mobile)
- `npm run tauri:dev` - Start Tauri development
- `npm run tauri:build` - Build Tauri applications
- `npm run tauri:build:desktop` - Build desktop applications
- `npm run tauri:build:mobile` - Build mobile applications

### Testing & Quality
- `npm run test` - Run tests
- `npm run test:ui` - Run tests with UI
- `npm run lint` - Run ESLint

## Configuration

### Environment Variables

Copy `.env.example` to `.env.development` and configure:

```env
VITE_API_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080
VITE_NODE_ENV=development
VITE_ENABLE_DEVTOOLS=true
```

### API Integration

The frontend connects to the ADX CORE backend services:

- **API Gateway**: `http://localhost:8080`
- **Auth Service**: `http://localhost:8081`
- **User Service**: `http://localhost:8082`
- **WebSocket**: `ws://localhost:8080/ws`

## Platform-Specific Features

### Web
- Progressive Web App (PWA)
- Service Worker for offline support
- Web Push notifications
- File System Access API

### Desktop (Tauri)
- Native OS integration
- File system access
- System notifications
- Menu bar/system tray
- Auto-updater
- Deep linking

### Mobile (Tauri)
- Native mobile UI
- Camera access
- GPS/location services
- Push notifications
- Biometric authentication
- App store distribution

## Internationalization

Supported languages:
- English (en)
- Spanish (es)
- French (fr)
- German (de)
- Japanese (ja)
- Chinese (zh)

Add new translations in `src/i18n/locales/[lang].json`.

## Theming

The app supports three theme modes:
- **Light**: Light theme
- **Dark**: Dark theme  
- **Auto**: Follows system preference

Themes are implemented with TailwindCSS dark mode classes.

## State Management

- **React Query**: Server state management
- **Zustand**: Client state management
- **React Context**: Global app state (auth, theme, tenant)

## Authentication

JWT-based authentication with:
- Login/logout
- Token refresh
- Role-based access control
- Multi-tenant support
- SSO integration ready

## Multi-tenant Support

- Tenant-specific branding
- Isolated data access
- Custom domains
- White-label support
- Tenant switching

## Development Guidelines

### Code Style
- TypeScript strict mode
- ESLint + Prettier
- Conventional commits
- Component-driven development

### Performance
- Code splitting with React.lazy
- Image optimization
- Bundle analysis
- Lighthouse optimization

### Security
- Content Security Policy
- XSS protection
- CSRF protection
- Secure token storage

## Deployment

### Web Deployment
```bash
npm run build
# Deploy dist/ folder to your web server
```

### Desktop Distribution
```bash
npm run tauri:build
# Installers generated in src-tauri/target/release/bundle/
```

### Mobile Distribution
```bash
# Android
npm run tauri android build
# APK/AAB generated in src-tauri/gen/android/

# iOS
npm run tauri ios build
# IPA generated in src-tauri/gen/ios/
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run linting and tests
6. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Support

- Documentation: [docs.adxcore.com](https://docs.adxcore.com)
- Issues: [GitHub Issues](https://github.com/adx-core/frontend/issues)
- Discord: [ADX CORE Community](https://discord.gg/adxcore)
- Email: support@adxcore.com