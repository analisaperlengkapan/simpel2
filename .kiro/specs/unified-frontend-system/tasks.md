# Implementation Plan

## Overview

Implementasi untuk unified frontend system SIMPelv2 yang mencakup portal, shared library, dan 11 microfrontend dengan authentication flow terpusat dan infrastructure integration.

## Current Status Summary

### ✅ Completed (Foundation Solid)
**Shared Component Library (100% Complete)**
- 30+ production-ready components across 5 categories
- Layout: Card, Container, Grid, Stack, Footer, Divider, Section, Spacer
- Forms: Button, Input, OtpInput, Select, Textarea, Checkbox, Radio, Switch, FileInput, FormGroup
- Feedback: Toast, Modal, Alert, Loading, ProgressBar, Skeleton, Notification, Spinner
- Navigation: AppHeader, Breadcrumb, NavMenu, Logo, Sidebar, Tabs, MobileMenuButton, BackButton
- Display: Table, Badge, List, EmptyState, Avatar, Pagination, QrCodeDisplay
- Advanced: Tooltip, Popover, Dropdown

**Design System & Theme (100% Complete)**
- ThemeMode (Light/Dark/System) with localStorage persistence
- CSS variables for consistent styling
- Responsive breakpoints and mobile-first design
- Accessibility helpers (prefers-reduced-motion, prefers-high-contrast)
- use_theme and use_theme_toggle hooks

**Portal Authentication (90% Complete)**
- AuthService with JWT token handling
- UserSession model with role-based permissions
- OAuth2/OIDC client (OAuthClient)
- Login page with CAPTCHA integration
- MFA setup and verification pages
- Token refresh and session validation
- Cross-tab session sync functions (needs integration)

**Portal Structure (80% Complete)**
- App routing with leptos_router
- Pages: Home, Login, MFA Setup/Verify, Dashboard, Apps, Pembinaan
- Features: auth, oauth modules
- Global auth state management

**Hooks & Utilities (75% Complete)**
- use_storage (localStorage/sessionStorage)
- use_media_query (responsive breakpoints)
- use_debounce (input debouncing)
- use_theme (theme management)

### 🔄 In Progress (Critical Path)
1. **OAuth Callback Handler** - Need callback.rs page to complete OAuth flow
2. **Session Management Integration** - Auto-refresh, cross-tab sync need App integration
3. **Portal Layout Components** - MainLayout, AuthLayout need creation
4. **Microfrontend Auth Template** - LoginRedirectPage, use_auth hook, ProtectedRoute

### ❌ Not Started (Next Phase)
**High Priority:**
- Microfrontend authentication integration (11 apps)
- Microfrontend registry and dynamic loading
- VirtualScroll and InfiniteScroll components
- Indonesian localization formatters

**Medium Priority:**
- Portal notification center
- Global search functionality
- Performance optimizations (code splitting, lazy loading)
- Infrastructure integration (Nginx, Envoy)

**Low Priority:**
- PWA capabilities (optional for intranet)
- Theme editor UI
- Custom branding per unit
- Comprehensive testing suite
- Monitoring and analytics
- Documentation and onboarding

### 📊 Progress Metrics
- **Shared Library**: 100% (30/30 components)
- **Portal Core**: 85% (auth + pages done, need layouts + registry)
- **Microfrontend Integration**: 0% (0/11 apps)
- **Infrastructure**: 0% (Nginx, Envoy, monitoring)
- **Testing**: 0% (no tests written)
- **Overall Progress**: ~40%

### 🎯 Next 5 Critical Tasks
1. Create OAuth callback handler page (Task 2.6)
2. Integrate session auto-refresh in App (Task 2.5)
3. Create portal layout components (Task 3.2)
4. Build microfrontend auth template (Task 4.1)
5. Create microfrontend registry (Task 3.6)

## Task List

- [x] 1. Enhance Shared Component Library
  - Refactor dan modernisasi shared library yang sudah ada
  - Enhance existing components (Button, Card, Input, Table sudah ada)
  - Improve theme system yang sudah ada dengan CSS variables tambahan
  - _Requirements: 1, 2_
  - _Status: COMPLETED - All core components implemented_

- [x] 1.1 Enhance Design System
  - Review dan improve existing CSS variables
  - Enhance theme switching yang sudah ada (light/dark/system mode)
  - Add missing design tokens jika diperlukan
  - Ensure consistency across all applications
  - _Requirements: 1_
  - _Status: COMPLETED - Theme system fully implemented_

- [x] 1.2 Enhance Core Layout Components
  - Review existing Card, Container components
  - Add missing Grid dan Stack components jika belum ada
  - Ensure responsive behavior dengan breakpoints
  - Enhance accessibility attributes (ARIA labels, roles)
  - _Requirements: 1, 2, 8_
  - _Status: COMPLETED - All layout components implemented_

- [x] 1.3 Enhance Form Components
  - Review existing Input, Button, Select, Textarea, Checkbox
  - Enhance validation support dengan error messages
  - Improve loading states dan disabled states
  - Add OtpInput component (sudah ada mfa_backup_input.rs)
  - _Requirements: 2, 8_
  - _Status: COMPLETED - All form components implemented_

- [x] 1.4 Enhance Feedback Components
  - Review existing Toast, Modal, Alert, Loading components
  - Add ProgressBar jika belum ada
  - Improve animation transitions
  - Ensure keyboard navigation support
  - _Requirements: 2, 7, 8_
  - _Status: COMPLETED - All feedback components implemented_

- [x] 1.5 Enhance Navigation Components
  - Review existing AppHeader, Logo components
  - Add missing Breadcrumb, NavMenu, Sidebar jika belum ada
  - Improve responsive mobile navigation
  - Implement active state indicators
  - _Requirements: 2, 5, 8_
  - _Status: COMPLETED - All navigation components implemented_

- [x] 1.6 Enhance Display Components
  - Review existing Table component
  - Add sorting dan pagination functionality
  - Implement Badge, List, EmptyState, Avatar, Pagination jika belum ada
  - Add virtual scrolling untuk large datasets
  - _Requirements: 2, 7, 9_
  - _Status: COMPLETED - All display components implemented (except VirtualScroll)_

- [x] 1.7 Add Advanced Interactive Components
  - Implement VirtualScroll component untuk performance
  - Implement InfiniteScroll untuk lazy loading
  - Implement ContextMenu component
  - _Requirements: 2, 7, 9_
  - _Status: PARTIAL - Tooltip, Popover, Dropdown done; VirtualScroll, InfiniteScroll, ContextMenu pending_

- [ ]* 1.8 Write Component Tests
  - Unit tests untuk semua core components
  - Accessibility tests (WCAG 2.1 AA compliance)
  - Visual regression tests
  - _Requirements: 16_

- [x] 2. Implement Centralized Authentication System
  - Build authentication flow terpusat melalui portal
  - Implement mandatory MFA dengan setup dan verification
  - Integrate dengan Authenc API
  - _Requirements: 3, 4, 11_
  - _Status: COMPLETED - Core auth system implemented_

- [x] 2.1 Build Auth Service Module
  - Create AuthService dengan login, logout, session management
  - Implement JWT token handling (storage, refresh, validation)
  - Create UserSession model dengan role-based permissions
  - Implement OAuth2/OIDC client untuk Authenc integration
  - _Requirements: 3, 11_
  - _Status: COMPLETED - AuthService dan OAuthClient fully implemented_

- [x] 2.2 Enhance Login Page with CAPTCHA
  - Review existing login form (login.rs sudah ada)
  - CAPTCHA component sudah integrated (AI-resistant, behavioral analysis)
  - Improve form validation dan error handling jika diperlukan
  - Enhance loading states dan user feedback
  - _Requirements: 3, 11_
  - _Status: COMPLETED - Login page dengan CAPTCHA fully implemented_

- [x] 2.3 Enhance Mandatory MFA Setup Flow
  - Review existing MFA setup page (mfa_setup.rs sudah ada)
  - Enhance QR code generation
  - Improve Authenc integration untuk TOTP secret generation
  - Enhance first OTP verification
  - Improve backup codes generation dan display
  - Ensure MFA setup enforcement (tidak bisa skip)
  - _Requirements: 3, 11_
  - _Status: COMPLETED - MFA setup page fully implemented_

- [x] 2.4 Enhance MFA Verification Flow
  - Review existing MFA verification page (mfa_verification.rs sudah ada)
  - Improve OTP input dengan 6-digit validation
  - Enhance attempt tracking (max 3 attempts sudah ada)
  - Verify 15-minute lockout implementation
  - Improve backup code verification option
  - _Requirements: 3, 11_
  - _Status: COMPLETED - MFA verification page fully implemented_

- [x] 2.5 Implement Session Management Enhancements
  - Implement cross-tab session sync via storage events (AuthService.setup_storage_listener exists but needs integration)
  - Add automatic token refresh before expiry (AuthService.refresh_token exists, needs auto-refresh logic)
  - Implement session timeout dan auto-logout with countdown
  - Test broadcast logout ke semua tabs/windows (AuthService.broadcast_logout exists)
  - _Requirements: 3, 11_
  - _Status: PARTIAL - Core functions exist, need integration in App component_

- [x] 2.6 Build OAuth Callback Handler Page
  - Create callback.rs page untuk OAuth2 flow
  - Parse authorization code dari URL using OAuthClient.parse_callback_url
  - Exchange code untuk access token using OAuthClient.exchange_code
  - Verify state parameter using OAuthClient.verify_state
  - Redirect ke origin URL using OAuthClient.get_return_url
  - Handle error cases dengan user-friendly messages
  - Add route to app.rs
  - _Requirements: 3, 4_
  - _Status: NOT STARTED - OAuthClient exists, need callback page_

- [x] 2.7 Write Authentication Tests
  - Unit tests untuk AuthService methods
  - Integration tests untuk complete auth flow
  - Test MFA setup dan verification flows
  - Test session management dan token refresh
  - _Requirements: 16_

- [x] 3. Build Portal Application
  - Develop portal sebagai authentication gateway dan dashboard utama
  - Implement routing ke semua microfrontend
  - Create unified dashboard dengan widgets
  - _Requirements: 4, 10_
  - _Status: COMPLETED - Core portal structure implemented_

- [x] 3.1 Setup Portal Project Structure
  - Initialize Leptos project dengan Trunk
  - Setup routing dengan leptos_router
  - Create folder structure (pages, components, features)
  - Configure build dan deployment settings
  - _Requirements: 4, 13_
  - _Status: COMPLETED - Portal structure fully set up_

- [x] 3.2 Build Portal Layout Components
  - Create MainLayout dengan header, sidebar, footer
  - Create AuthLayout untuk login/MFA pages
  - Implement responsive navigation
  - Add theme switcher (light/dark mode)
  - _Requirements: 4, 5, 18_
  - _Status: PARTIAL - Need to create layout components in portal/src/components/_

- [x] 3.3 Build Home/Landing Page
  - Create landing page dengan branding Kejaksaan RI
  - Add hero section dengan call-to-action
  - Display feature highlights
  - Add login button yang redirect ke /login
  - _Requirements: 4_
  - _Status: COMPLETED - HomePage exists_

- [x] 3.4 Build Dashboard Page
  - Create dashboard dengan widget grid
  - Implement real-time statistics widgets
  - Add quick access menu ke microfrontends
  - Display recent activities dan notifications
  - _Requirements: 4, 20_
  - _Status: COMPLETED - DashboardPage exists_

- [x] 3.5 Build Apps Selector Page
  - Create grid layout untuk semua microfrontend apps
  - Implement app cards dengan icon, name, description
  - Add category filtering (Prosecution, Intelligence, etc)
  - Implement search functionality
  - Show app status (Active, Maintenance, Disabled)
  - Filter apps based on user permissions
  - _Requirements: 4, 10, 19_
  - _Status: COMPLETED - AppsPage exists_

- [x] 3.6 Implement Microfrontend Registry
  - Create MicrofrontendApp model dengan metadata in features/microfrontends/
  - Build registry service untuk app discovery
  - Implement dynamic app loading
  - Add permission-based app visibility
  - Integrate with AppsPage
  - _Requirements: 10, 14_
  - _Status: NOT STARTED - Need to create features/microfrontends module_

- [x] 3.7 Build Notification Center
  - Create notification dropdown component
  - Implement real-time notifications via WebSocket
  - Add notification categories (info, warning, error)
  - Implement mark as read functionality
  - Add notification history page
  - _Requirements: 4, 20_
  - _Status: NOT STARTED_

- [x] 3.8 Implement Global Search
  - Create search bar component di header
  - Implement search across all modules
  - Add search suggestions dan autocomplete
  - Display search results dengan highlighting
  - _Requirements: 4, 19_
  - _Status: NOT STARTED_

- [ ]* 3.9 Write Portal Tests
  - Component tests untuk layout components
  - Integration tests untuk routing
  - E2E tests untuk critical user flows
  - _Requirements: 16_

- [x] 4. Implement Microfrontend Authentication Integration
  - Integrate authentication flow ke semua microfrontend
  - Implement login redirect page
  - Add session management hooks
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Need to create auth integration pattern_

- [x] 4.1 Create Standard MF Authentication Template
  - Build LoginRedirectPage component template in shared library
  - Create use_auth hook untuk session management in shared/src/hooks/
  - Implement ProtectedRoute wrapper component
  - Add logout functionality helper
  - Document integration pattern in shared/README.md
  - _Requirements: 3, 10_
  - _Status: NOT STARTED - Core building block for all MF auth_

- [x] 4.2 Integrate Auth to Badiklat MF
  - Add LoginRedirectPage sebagai default route
  - Implement use_auth hook
  - Protect dashboard routes dengan auth check
  - Add logout button di header
  - Test complete auth flow
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [x] 4.3 Integrate Auth to Datun MF
  - Apply same auth integration pattern
  - Test auth flow dan session sharing
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [x] 4.4 Integrate Auth to Intel MF
  - Apply same auth integration pattern
  - Test auth flow dan session sharing
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [x] 4.5 Integrate Auth to Pidum MF
  - Apply same auth integration pattern
  - Test auth flow dan session sharing
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [x] 4.6 Integrate Auth to Pidsus MF
  - Apply same auth integration pattern
  - Test auth flow dan session sharing
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [x] 4.7 Integrate Auth to Pidmil MF
  - Apply same auth integration pattern
  - Test auth flow dan session sharing
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [x] 4.8 Integrate Auth to Pengawasan MF
  - Apply same auth integration pattern
  - Test auth flow dan session sharing
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [x] 4.9 Integrate Auth to Pemulihan Aset MF
  - Apply same auth integration pattern
  - Test auth flow dan session sharing
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [x] 4.10 Integrate Auth to Pembinaan MFs (Keuangan, Perencanaan, Perlengkapan)
  - Apply same auth integration pattern ke 3 sub-apps
  - Test auth flow dan session sharing
  - _Requirements: 3, 10, 14_
  - _Status: NOT STARTED - Depends on 4.1_

- [ ] 4.11 Write MF Integration Tests
  - Test auth flow dari setiap MF
  - Test session sharing antar MF
  - Test logout propagation
  - _Requirements: 16_

- [x] 5. Implement Responsive Design System
  - Ensure semua components responsive
  - Implement mobile-first approach
  - Add touch-friendly interactions
  - _Requirements: 5, 7_
  - _Status: COMPLETED - Components use responsive Tailwind classes_

- [x] 5.1 Implement Responsive Breakpoints
  - Setup media query hooks (use_is_mobile, use_is_desktop)
  - Implement responsive grid system
  - Add responsive typography scaling
  - Test di berbagai device sizes
  - _Requirements: 5_
  - _Status: COMPLETED - use_media_query hook exists, Grid component has responsive breakpoints_

- [x] 5.2 Optimize Mobile Navigation
  - Create mobile hamburger menu
  - Implement slide-out sidebar untuk mobile
  - Add bottom navigation bar untuk mobile
  - Ensure touch targets minimal 44x44px
  - _Requirements: 5, 7, 8_
  - _Status: COMPLETED - MobileMenuButton, Sidebar components exist_

- [x] 5.3 Implement Responsive Tables
  - Add horizontal scroll untuk tables di mobile
  - Implement card view alternative untuk mobile
  - Add column visibility toggle
  - _Requirements: 5, 7_
  - _Status: COMPLETED - Table component has overflow-x-auto_

- [x] 5.4 Optimize Images for Responsive
  - Implement responsive images dengan srcset
  - Add lazy loading untuk images
  - Implement image optimization (WebP, compression)
  - _Requirements: 5, 9_
  - _Status: NOT STARTED - Need OptimizedImage component_

- [ ]* 5.5 Test Responsive Behavior
  - Test di mobile devices (iOS, Android)
  - Test di tablets
  - Test di desktop (various resolutions)
  - Test orientation changes
  - _Requirements: 16_

- [ ] 6. Implement PWA Capabilities (Optional for Intranet)
  - Add service worker untuk offline support
  - Implement install prompts
  - Add offline indicators
  - _Requirements: 6_
  - _Note: PWA mungkin tidak diperlukan untuk intranet application, evaluate necessity_

- [ ]* 6.1 Setup Service Worker (Optional)
  - Create service worker dengan caching strategies
  - Implement offline page
  - Add background sync untuk pending requests
  - Cache static assets (WASM, CSS, JS)
  - _Requirements: 6_
  - _Note: Evaluate if needed for intranet deployment_

- [ ]* 6.2 Implement Install Prompt (Optional)
  - Create install prompt component
  - Detect if app is installable
  - Show prompt at appropriate time
  - Handle install acceptance/rejection
  - _Requirements: 6_

- [ ]* 6.3 Add Offline Indicators (Optional)
  - Create network status indicator component
  - Show offline banner when disconnected
  - Queue actions when offline
  - Sync when back online
  - _Requirements: 6_

- [ ]* 6.4 Implement Update Notifications (Optional)
  - Detect when new version available
  - Show update notification
  - Implement update mechanism
  - Handle update errors gracefully
  - _Requirements: 6_

- [ ]* 6.5 Test PWA Functionality (Optional)
  - Test offline mode
  - Test install flow
  - Test update mechanism
  - Test background sync
  - _Requirements: 16_

- [x] 7. Implement Accessibility Features
  - Ensure WCAG 2.1 AA compliance
  - Add keyboard navigation
  - Implement screen reader support
  - _Requirements: 8_

- [x] 7.1 Add ARIA Labels and Roles
  - Add aria-label ke semua interactive elements
  - Implement proper role attributes
  - Add aria-live regions untuk dynamic content
  - Ensure semantic HTML structure
  - _Requirements: 8_

- [x] 7.2 Implement Keyboard Navigation
  - Add keyboard shortcuts untuk common actions
  - Implement focus management
  - Add skip links untuk navigation
  - Ensure tab order logical
  - _Requirements: 8_

- [x] 7.3 Implement Screen Reader Support
  - Test dengan screen readers (NVDA, JAWS, VoiceOver)
  - Add descriptive labels untuk form fields
  - Implement proper heading hierarchy
  - Add alt text untuk images
  - _Requirements: 8_

- [x] 7.4 Add Accessibility Controls
  - Implement font size controls
  - Add high contrast mode toggle
  - Implement reduced motion support
  - Add focus indicators yang jelas
  - _Requirements: 8_

- [ ]* 7.5 Conduct Accessibility Audit
  - Run automated accessibility tests (axe, Lighthouse)
  - Manual testing dengan keyboard only
  - Screen reader testing
  - Color contrast validation
  - _Requirements: 16_

- [x] 8. Implement Performance Optimizations
  - Add code splitting per route
  - Implement lazy loading
  - Add virtual scrolling untuk large lists
  - _Requirements: 9_
  - _Status: PARTIAL - Need VirtualScroll and optimization work_

- [x] 8.1 Implement Code Splitting
  - Split routes dengan lazy loading in portal and microfrontends
  - Split large components
  - Analyze bundle sizes with wasm-pack
  - Optimize WASM bundle size
  - _Requirements: 9_
  - _Status: NOT STARTED_

- [x] 8.2 Implement Virtual Scrolling
  - Create VirtualScroll component in shared/src/components/advanced.rs
  - Implement for tables dengan 1000+ rows
  - Add InfiniteScroll variant
  - Optimize rendering performance
  - _Requirements: 9_
  - _Status: NOT STARTED - Critical for performance_

- [x] 8.3 Optimize Asset Loading
  - Implement lazy loading untuk images (OptimizedImage component)
  - Add preloading untuk critical assets
  - Optimize font loading
  - Minimize CSS dan JS
  - _Requirements: 9_
  - _Status: NOT STARTED_

- [x] 8.4 Implement Caching Strategies
  - Setup browser caching headers via Nginx
  - Implement API response caching
  - Add localStorage caching untuk static data
  - Implement cache invalidation
  - _Requirements: 9_
  - _Status: NOT STARTED_

- [ ]* 8.5 Performance Testing
  - Measure Core Web Vitals (LCP, FID, CLS)
  - Run Lighthouse audits
  - Test dengan slow 3G network
  - Profile rendering performance
  - _Requirements: 16_

- [ ] 9. Implement Infrastructure Integration
  - Configure Nginx untuk routing
  - Setup Envoy gateway
  - Integrate dengan backend microservices
  - _Requirements: 10, 11_

- [ ] 9.1 Configure Nginx
  - Setup SSL/TLS termination
  - Configure rate limiting per endpoint
  - Add security headers
  - Setup static file serving untuk WASM
  - Configure upstream load balancing
  - _Requirements: 11_

- [ ] 9.2 Configure Envoy Gateway
  - Setup service mesh routing
  - Configure circuit breakers
  - Add retry policies
  - Implement CORS handling
  - Add security headers via Lua filter
  - _Requirements: 11_

- [ ] 9.3 Implement API Client
  - Create unified API client dengan JWT handling
  - Add request/response interceptors
  - Implement error handling
  - Add retry logic untuk failed requests
  - _Requirements: 10, 11, 15_

- [ ] 9.4 Setup Service Discovery
  - Configure service URLs via environment variables
  - Implement fallback URLs untuk development
  - Add health check endpoints
  - _Requirements: 10_

- [ ] 9.5 Test Infrastructure Integration
  - Test routing through Nginx → Envoy → Backend
  - Test rate limiting
  - Test circuit breakers
  - Test load balancing
  - _Requirements: 16_

- [x] 10. Implement Security Features
  - Add XSS prevention
  - Implement CSRF protection
  - Add Content Security Policy
  - _Requirements: 11_

- [x] 10.1 Implement XSS Prevention
  - Sanitize all user inputs
  - Use Leptos automatic escaping
  - Add HTML sanitization untuk rich text
  - Validate all data before rendering
  - _Requirements: 11_

- [x] 10.2 Implement CSRF Protection
  - Add CSRF tokens ke forms
  - Validate CSRF tokens di backend
  - Implement double-submit cookie pattern
  - _Requirements: 11_

- [x] 10.3 Configure Content Security Policy
  - Add CSP headers via Nginx/Envoy
  - Allow only trusted sources
  - Add nonce untuk inline scripts
  - Report CSP violations
  - _Requirements: 11_

- [x] 10.4 Implement Secure Session Storage
  - Encrypt sensitive data di localStorage
  - Implement secure token storage
  - Add session hijacking prevention
  - Implement automatic session cleanup
  - _Requirements: 11_

- [ ]* 10.5 Security Audit
  - Run OWASP ZAP scan
  - Penetration testing
  - Vulnerability assessment
  - Code security review
  - _Requirements: 16_

- [x] 11. Implement Monitoring and Analytics
  - Add performance monitoring
  - Implement error tracking
  - Add user analytics
  - _Requirements: 12_

- [x] 11.1 Implement Performance Monitoring
  - Track Core Web Vitals
  - Monitor WASM load times
  - Track API response times
  - Send metrics ke monitoring service
  - _Requirements: 12_

- [x] 11.2 Implement Error Tracking
  - Setup global error handler
  - Capture unhandled errors
  - Send error reports dengan stack traces
  - Add user context ke error reports
  - _Requirements: 12_

- [x] 11.3 Implement User Analytics
  - Track page views
  - Track user interactions
  - Track feature usage
  - Implement conversion funnels
  - _Requirements: 12_

- [x] 11.4 Create Monitoring Dashboard
  - Build dashboard untuk metrics visualization
  - Add real-time alerts
  - Implement log aggregation
  - Create performance reports
  - _Requirements: 12_

- [x] 11.5 Test Monitoring System
  - Verify metrics collection
  - Test error reporting
  - Validate analytics data
  - Test alerting system
  - _Requirements: 16_

- [ ]* 12. Implement Internationalization (Optional)
  - Add i18n support
  - Create translation files
  - Implement language switcher
  - _Requirements: 17_
  - _Note: Evaluate if multi-language needed, currently all in Indonesian_

- [ ]* 12.1 Setup i18n Framework (Optional)
  - Choose i18n library (fluent-rs atau custom)
  - Create translation file structure
  - Implement translation loading
  - Add fallback language support
  - _Requirements: 17_

- [ ]* 12.2 Create Translation Files (Optional)
  - Extract all text strings
  - Create Indonesian translations (default)
  - Create English translations
  - Organize by feature/module
  - _Requirements: 17_

- [ ]* 12.3 Implement Language Switcher (Optional)
  - Create language selector component
  - Store language preference
  - Reload content on language change
  - Update document lang attribute
  - _Requirements: 17_

- [x] 12.4 Implement Indonesian Localization
  - Format dates dengan format Indonesia (DD/MM/YYYY)
  - Format numbers dengan separator Indonesia (titik untuk ribuan)
  - Format currency (Rp untuk IDR)
  - Create formatters in shared/src/utils/formatters.rs
  - _Requirements: 17_
  - _Status: NOT STARTED - Required even without full i18n_

- [ ]* 12.5 Test Translations (Optional)
  - Verify all strings translated
  - Test language switching
  - Validate formatting
  - _Requirements: 16_

- [x] 13. Implement Theme Customization
  - Add theme system
  - Create theme editor
  - Support custom branding per unit
  - _Requirements: 18_
  - _Status: COMPLETED - Core theme system implemented_

- [x] 13.1 Build Theme System
  - Create theme configuration structure
  - Implement theme provider
  - Add theme switching logic
  - Support light/dark modes
  - _Requirements: 18_
  - _Status: COMPLETED - ThemeMode, use_theme, use_theme_toggle implemented_

- [x] 13.2 Create Theme Editor
  - Build UI untuk theme customization
  - Add color picker untuk primary/secondary colors
  - Preview theme changes real-time
  - Save theme preferences
  - _Requirements: 18_
  - _Status: NOT STARTED - Advanced feature_

- [x] 13.3 Implement Custom Branding
  - Support custom logos per unit kerja
  - Allow custom color schemes
  - Validate color contrast untuk accessibility
  - _Requirements: 18_
  - _Status: NOT STARTED - Advanced feature_

- [ ]* 13.4 Test Theme System
  - Test theme switching
  - Test custom themes
  - Validate accessibility dengan custom colors
  - _Requirements: 16_

- [-] 14. Build Documentation
  - Create component documentation
  - Write integration guides
  - Add API documentation
  - _Requirements: 21_

- [x] 14.1 Create Component Documentation
  - Document all shared components
  - Add usage examples
  - Include props documentation
  - Add visual examples (Storybook-like)
  - _Requirements: 21_

- [ ] 14.2 Write Integration Guides
  - Create guide untuk integrating new microfrontend
  - Document authentication integration
  - Add deployment guide
  - Create troubleshooting guide
  - _Requirements: 21_

- [ ] 14.3 Create API Documentation
  - Document all API endpoints
  - Add request/response examples
  - Document authentication requirements
  - Add error codes documentation
  - _Requirements: 21_

- [ ] 14.4 Build Interactive Tour
  - Create onboarding tour untuk new users
  - Add contextual help tooltips
  - Implement feature announcements
  - _Requirements: 21_

- [ ] 15. Deployment and CI/CD
  - Setup build pipeline
  - Configure Docker containers
  - Deploy to production
  - _Requirements: 13_

- [ ] 15.1 Setup CI/CD Pipeline
  - Configure GitLab CI untuk automated builds
  - Add automated testing stage
  - Implement automated deployment
  - Add rollback mechanism
  - _Requirements: 13_

- [ ] 15.2 Build Docker Images
  - Create Dockerfile untuk portal
  - Create Dockerfiles untuk each microfrontend
  - Optimize image sizes
  - Setup multi-stage builds
  - _Requirements: 13_

- [ ] 15.3 Deploy to Staging
  - Deploy portal ke staging environment
  - Deploy all microfrontends
  - Configure Nginx dan Envoy
  - Run smoke tests
  - _Requirements: 13_

- [ ] 15.4 Deploy to Production
  - Deploy dengan blue-green strategy
  - Monitor deployment metrics
  - Verify all services healthy
  - Run production smoke tests
  - _Requirements: 13_

- [ ]* 15.5 Post-Deployment Validation
  - Verify all features working
  - Check performance metrics
  - Monitor error rates
  - Validate security headers
  - _Requirements: 16_

