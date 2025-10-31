# Implementation Plan: Microfrontend OAuth2 Integration

## Epic 1: Shared Library OAuth2 Components

- [ ] 1. Create OAuth2 types and utilities
  - Create `OAuth2Config` struct dengan authenc_url, portal_url, client_id, redirect_uri, scopes
  - Create `OAuth2State` struct untuk CSRF protection
  - Create `OAuth2Error` enum untuk error handling
  - Create utility functions: `generate_random_state()`, `save_to_session_storage()`, `load_from_session_storage()`
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 11.2_

- [ ] 2. Implement OAuth2LoginButton component
  - Create component di `antarmuka/shared/src/components/auth/oauth2_login_button.rs`
  - Implement state generation dengan crypto-random (32 characters)
  - Implement state saving ke sessionStorage
  - Implement authorization URL building dengan proper encoding
  - Implement redirect ke Portal `/oidc/authorize`
  - Add loading state dan accessibility labels
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 9.1, 15.1_

- [ ] 3. Implement OAuth2CallbackPage component
  - Create component di `antarmuka/shared/src/components/auth/oauth2_callback.rs`
  - Implement URL parameter extraction (code, state)
  - Implement state validation (match, expiration check)
  - Implement token exchange dengan Authenc `/oidc/token`
  - Implement JWT decoding ke UserSession
  - Implement session saving ke localStorage dan auth context
  - Add error handling untuk semua failure scenarios
  - Add loading states dengan proper messages
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 8.2, 9.2, 9.3_

- [ ] 4. Enhance LoginRedirectPage component
  - Update component untuk support OAuth2 flow
  - Add OAuth2LoginButton integration
  - Add custom message support
  - Add loading state during redirect
  - Maintain backward compatibility
  - _Requirements: 1.1, 9.1, 13.1, 15.1_

- [ ] 5. Export OAuth2 components dari shared library
  - Update `antarmuka/shared/src/components/auth/mod.rs`
  - Export OAuth2LoginButton, OAuth2CallbackPage
  - Update prelude untuk easy imports
  - Update documentation
  - _Requirements: 13.1, 13.2_

## Epic 2: Backend OAuth2 Enhancement

- [ ] 6. Implement authorization code storage
  - Create `OAuth2CodeStore` di `infra/authenc/src/services/oauth2_code_store.rs`
  - Implement Redis storage untuk fast access (10 min TTL)
  - Implement database storage untuk persistence
  - Implement `store_code()` method
  - Implement `get_and_consume_code()` method (one-time use)
  - Implement code expiration check
  - Add metrics untuk code generation dan usage
  - _Requirements: 4.2, 4.3, 4.5, 11.2, 14.2_

- [ ] 7. Create authorization_codes database table
  - Create migration `infra/authenc/migrations/XXX_authorization_codes.sql`
  - Add columns: code, user_id, client_id, redirect_uri, scopes, code_challenge, expires_at, used, created_at
  - Add index on code column
  - Add index on expires_at for cleanup
  - Add foreign key constraint ke users table
  - _Requirements: 4.2, 4.3_

- [ ] 8. Enhance /oidc/authorize endpoint
  - Update handler di `infra/authenc/src/handlers/oauth2_comprehensive.rs`
  - Add client_id validation
  - Add redirect_uri validation against registered URIs
  - Add session check (authenticated or not)
  - If not authenticated, redirect to Portal `/login` dengan return_url
  - If authenticated, generate authorization code
  - Store code dengan OAuth2CodeStore
  - Redirect to client callback dengan code dan state
  - Add logging untuk audit trail
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 11.1, 12.1_

- [ ] 9. Enhance /oidc/token endpoint untuk authorization_code grant
  - Update handler untuk support authorization_code grant type
  - Validate authorization code dengan OAuth2CodeStore
  - Verify code not expired dan not used
  - Verify client_id dan redirect_uri match
  - Mark code as used (one-time use)
  - Generate access_token, refresh_token, id_token
  - Set SSO cookie dengan proper domain
  - Return token response
  - Add logging dan metrics
  - _Requirements: 2.4, 2.5, 4.5, 5.1, 5.2, 5.3, 12.1, 14.3_

- [ ] 10. Implement authorization code cleanup job
  - Create background job untuk cleanup expired codes
  - Run every 1 hour
  - Delete codes older than 10 minutes
  - Add metrics untuk cleanup operations
  - _Requirements: 4.2, 14.2_

## Epic 3: Portal OAuth2 Authorization UI

- [ ] 11. Enhance /oidc/authorize handler
  - Update route handler di Portal
  - Check authentication status
  - If not authenticated, redirect to `/login` dengan return_url
  - If authenticated, show consent screen (optional) atau auto-approve
  - Call Authenc backend untuk generate code
  - Redirect to microfrontend callback
  - _Requirements: 3.1, 3.2, 3.10, 4.1_

- [ ] 12. Update login flow untuk support return_url
  - Update LoginPage untuk accept return_url parameter
  - After successful login (CAPTCHA + MFA), redirect to return_url
  - If no return_url, redirect to dashboard (default behavior)
  - Maintain return_url through MFA setup/verify flow
  - _Requirements: 3.2, 3.10_

- [ ] 13. Add OAuth2 consent screen (optional)
  - Create ConsentPage component
  - Show client information (name, logo, permissions)
  - Show "Allow" dan "Deny" buttons
  - If Allow, proceed dengan code generation
  - If Deny, redirect back dengan error
  - Add "Remember my choice" checkbox
  - _Requirements: 4.1_

## Epic 4: Microfrontend OAuth2 Implementation

- [ ] 14. Implement OAuth2 callback route di Badiklat (pilot)
  - Add `/callback` route di `antarmuka/badiklat/src/app.rs`
  - Use OAuth2CallbackPage component dari shared
  - Configure OAuth2Config dengan client_id="badiklat"
  - Handle success: redirect to home atau original URL
  - Handle error: show error page dengan retry button
  - Test complete flow
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_

- [ ] 15. Update Badiklat app untuk show login button
  - Update main app component
  - Check authentication dengan `use_auth()`
  - If not authenticated, show LoginRedirectPage dengan OAuth2LoginButton
  - If authenticated, show app content
  - Test SSO cookie detection
  - _Requirements: 1.1, 5.5, 10.1, 10.2, 10.3, 10.4_

- [ ] 16. Test Badiklat OAuth2 flow end-to-end
  - Test login dari Badiklat → Portal → back to Badiklat
  - Test CAPTCHA completion
  - Test MFA setup (new user)
  - Test MFA verification (existing user)
  - Test SSO cookie sharing
  - Test logout propagation
  - Fix any issues found
  - _Requirements: All from Req 1-7_

- [ ] 17. Rollout OAuth2 to remaining microfrontends
- [ ] 17.1 Implement OAuth2 callback di Datun
  - Add `/callback` route
  - Configure OAuth2Config dengan client_id="datun"
  - Test complete flow
  - _Requirements: 2.1-2.7_

- [ ] 17.2 Implement OAuth2 callback di Intel
  - Add `/callback` route
  - Configure OAuth2Config dengan client_id="intel"
  - Test complete flow
  - _Requirements: 2.1-2.7_

- [ ] 17.3 Implement OAuth2 callback di Pembinaan modules
  - Add `/callback` route di Keuangan, Perencanaan, Perlengkapan
  - Configure OAuth2Config untuk each module
  - Test complete flow
  - _Requirements: 2.1-2.7_

- [ ] 17.4 Implement OAuth2 callback di remaining microfrontends
  - Pemulihan Aset, Pengawasan, Pidmil, Pidsus, Pidum
  - Add `/callback` route untuk each
  - Configure OAuth2Config untuk each
  - Test complete flow
  - _Requirements: 2.1-2.7_

## Epic 5: Cross-Tab Logout Enhancement

- [ ] 18. Enhance logout propagation
  - Verify storage event listener di semua microfrontend
  - Test logout dari Portal propagate ke microfrontend
  - Test logout dari microfrontend propagate ke Portal
  - Test logout propagate ke multiple tabs
  - Add logging untuk logout events
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_

- [ ] 19. Implement "logout from all devices" feature
  - Add endpoint `/oidc/logout-all` di backend
  - Invalidate all sessions across all devices
  - Add UI button di Portal settings
  - Test functionality
  - _Requirements: 6.1, 6.2, 6.3_

## Epic 6: Token Refresh Enhancement

- [ ] 20. Implement automatic token refresh
  - Add token expiration check di shared library
  - Implement automatic refresh before expiration (5 min buffer)
  - Call `/oidc/refresh` endpoint dengan refresh_token
  - Update tokens di localStorage
  - Handle refresh failure (redirect to login)
  - Add loading state during refresh (optional)
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_

- [ ] 21. Add token refresh retry logic
  - Implement exponential backoff untuk retry
  - Maximum 3 retry attempts
  - If all retries fail, logout user
  - Add metrics untuk refresh success/failure rate
  - _Requirements: 7.2, 7.5, 12.6_

## Epic 7: Error Handling and UX

- [ ] 22. Implement comprehensive error handling
  - Create OAuth2Error enum dengan all error types
  - Implement user-friendly error messages (Indonesian)
  - Create OAuth2ErrorPage component
  - Add "Try Again" functionality
  - Add error logging untuk debugging
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7_

- [ ] 23. Add loading states
  - Add loading spinner during OAuth2 redirect
  - Add loading spinner during callback processing
  - Add loading spinner during token exchange
  - Add progress indicators untuk multi-step flow
  - Add accessibility labels untuk screen readers
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 15.2_

- [ ] 24. Improve session persistence
  - Verify localStorage saving works correctly
  - Verify SSO cookie reading works correctly
  - Add session validation on page load
  - Add session refresh on page focus
  - Handle edge cases (localStorage full, cookies disabled)
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

## Epic 8: Testing and Validation

- [ ] 25. Write unit tests
  - Test OAuth2 state generation dan validation
  - Test authorization URL building
  - Test callback parameter extraction
  - Test token exchange logic
  - Test error handling
  - Target: > 80% code coverage
  - _Requirements: All_

- [ ] 26. Write integration tests
  - Test complete OAuth2 flow (microfrontend → Portal → back)
  - Test CAPTCHA integration
  - Test MFA integration
  - Test SSO cookie sharing
  - Test cross-tab logout
  - Test token refresh
  - Test error scenarios
  - _Requirements: All_

- [ ] 27. Performance testing
  - Load test OAuth2 endpoints (1000+ concurrent)
  - Measure OAuth2 flow duration
  - Measure callback processing time
  - Measure token exchange time
  - Verify meets performance targets
  - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5, 14.6_

- [ ] 28. Security testing
  - Test CSRF protection (invalid state)
  - Test authorization code reuse prevention
  - Test redirect URI validation
  - Test token expiration
  - Test XSS protection
  - Penetration testing
  - _Requirements: 11.1-11.10_

## Epic 9: Documentation and Deployment

- [ ] 29. Create documentation
  - OAuth2 integration guide untuk developers
  - User guide untuk login flow
  - Troubleshooting guide
  - API documentation
  - Architecture diagrams
  - _Requirements: All_

- [ ] 30. Deploy to staging
  - Deploy backend changes
  - Deploy Portal changes
  - Deploy shared library changes
  - Deploy pilot microfrontend (Badiklat)
  - Run integration tests
  - Fix any issues
  - _Requirements: All_

- [ ] 31. Deploy to production
  - Deploy backend (with feature flag)
  - Deploy Portal
  - Deploy shared library
  - Gradual rollout to microfrontends (one per week)
  - Monitor metrics
  - Gather user feedback
  - _Requirements: All_

## Task Execution Guidelines

### Priority Levels
- **P0 (Critical)**: Epic 1-2 (Core OAuth2 implementation)
- **P1 (High)**: Epic 3-4 (Portal and pilot microfrontend)
- **P2 (Medium)**: Epic 5-7 (Enhancements and UX)
- **P3 (Low)**: Epic 8-9 (Testing and deployment)

### Execution Order
1. **Phase 1 (Week 1)**: Epic 1-2 (Shared library + Backend)
2. **Phase 2 (Week 2)**: Epic 3 (Portal enhancement)
3. **Phase 3 (Week 3)**: Epic 4 (Pilot microfrontend)
4. **Phase 4 (Week 4-6)**: Epic 5-7 (Enhancements)
5. **Phase 5 (Week 7-8)**: Epic 8-9 (Testing + Deployment)

### Dependencies
- Epic 2 depends on Epic 1 (shared library types)
- Epic 3 depends on Epic 2 (backend endpoints)
- Epic 4 depends on Epic 1, 2, 3 (all components ready)
- Epic 5-7 can run in parallel after Epic 4
- Epic 8 depends on Epic 1-7 (all features implemented)
- Epic 9 depends on Epic 8 (testing completed)

### Testing Strategy
- Unit tests written alongside implementation
- Integration tests after each epic
- End-to-end tests before deployment
- Performance tests before production
- Security audit before production

### Rollback Plan
- Each epic implemented in feature branch
- Merge to main after code review and testing
- Tag each epic completion
- Feature flags untuk gradual rollout
- Quick rollback capability (< 5 minutes)
