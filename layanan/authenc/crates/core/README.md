# authenc-core

Core business logic for Authenc identity provider.

## Purpose

This crate implements the core business logic and service implementations for Authenc, including:

- **Authentication Service**: Username/password authentication, MFA verification, session management
- **User Management Service**: User CRUD operations, profile management, email verification
- **Realm Management Service**: Multi-realm support, realm configuration
- **OAuth2/OIDC Service**: Authorization Code flow, Client Credentials flow, Refresh Token flow
- **Brute Force Protector**: Failed login tracking, account lockout, CAPTCHA integration
- **Event Publishing**: Domain event publishing for audit and cache invalidation

## Dependencies

- `authenc-types`: Shared types and traits
- `authenc-crypto`: Cryptographic operations (JWT, password hashing)
- `authenc-storage`: Database layer

## Architecture

The core crate coordinates between different stores and implements business logic without direct database access. All database operations are delegated to storage implementations via traits defined in `authenc-types`.

## Requirements

Implements requirements:
- REQ-AUTH-001 through REQ-AUTH-005 (Authentication)
- REQ-USER-001 through REQ-USER-004 (User Management)
- REQ-REALM-001 through REQ-REALM-003 (Realm Management)
- REQ-OAUTH-001 through REQ-OAUTH-003 (OAuth2)
- REQ-ARCH-004 (Core business logic)
