# authenc-mfa

Multi-factor authentication for Authenc identity provider.

## Purpose

This crate implements multi-factor authentication features:

- **TOTP Service**: Time-based One-Time Password setup and verification
- **Backup Codes Service**: Generate and validate one-time backup codes
- **MFA Policy**: Enforce MFA requirements based on realm configuration
- **Secreton Integration**: Store TOTP secrets securely in Secreton

## Features

### TOTP

- QR code generation for authenticator apps
- 6-digit code validation
- 30-second time window with ±1 period tolerance
- RFC 6238 compliant

### Backup Codes

- 10 one-time use codes per user
- Regeneration on demand
- Notification on backup code usage

### Policy Enforcement

- Realm-level MFA requirements
- User-level MFA enablement
- MFA verification in authentication flow

## Requirements

Implements requirements:

- REQ-AUTH-002 (MFA support)
- REQ-MFA-001 through REQ-MFA-003 (TOTP and backup codes)
