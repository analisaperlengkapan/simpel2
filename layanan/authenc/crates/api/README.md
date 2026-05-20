# authenc-api

Public REST API for Authenc identity provider.

## Purpose

This crate provides public HTTP endpoints for microfrontends and external clients:

- **Authentication Endpoints**: Login, logout, token refresh
- **Token Validation**: Validate JWT tokens for other microfrontends
- **OAuth2/OIDC**: Authorization, token, userinfo, discovery endpoints
- **User Profile**: Get current user profile
- **Session Management**: List and revoke sessions

## Endpoints

### Authentication

- `POST /api/v1/auth/login` - Username/password login
- `POST /api/v1/auth/logout` - Session invalidation
- `POST /api/v1/auth/refresh` - Refresh token exchange
- `GET /api/v1/auth/me` - Get current user profile

### Token Validation

- `POST /api/v1/auth/validate` - Validate JWT token

### OAuth2/OIDC

- `GET /api/v1/oauth2/authorize` - Authorization endpoint
- `POST /api/v1/oauth2/token` - Token endpoint
- `GET /api/v1/oauth2/.well-known/openid-configuration` - Discovery
- `GET /api/v1/oauth2/userinfo` - UserInfo endpoint

## Security

- CORS configuration for microfrontends
- Rate limiting (per-IP and per-user)
- JWT validation middleware
- HTTPS enforcement

## Requirements

Implements requirements:

- REQ-API-001 (Public REST API)
- REQ-AUTH-001 through REQ-AUTH-005 (Authentication endpoints)
- REQ-OAUTH-001 through REQ-OAUTH-003 (OAuth2 endpoints)
- REQ-OIDC-001, REQ-OIDC-002 (OIDC endpoints)
- REQ-SEC-006, REQ-SEC-007 (Rate limiting, CORS)
