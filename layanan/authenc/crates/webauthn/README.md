# authenc-webauthn

WebAuthn/Passkeys (FIDO2) authentication for Authenc - **PRIMARY AUTHENTICATION METHOD**.

## Purpose

This crate implements WebAuthn/FIDO2 passkey authentication as the PRIMARY authentication method for Authenc:

- **Passkey Registration**: Register new passkeys (platform authenticators or security keys)
- **Passkey Authentication**: Authenticate with passkeys (including usernameless authentication)
- **Credential Management**: List, delete, and update passkey nicknames
- **Replay Attack Prevention**: Credential counter validation
- **Origin Binding**: Enforce origin-bound credentials
- **Multi-Device Support**: Sync passkeys via platform providers (iCloud Keychain, Google Password Manager)

## Security Features

### Phishing-Resistant Authentication

- Public key cryptography (no shared secrets)
- Origin-bound credentials (cannot be used on different domains)
- Replay attack prevention via credential counter

### Platform Authenticator Support

- **iOS/iPadOS**: Touch ID, Face ID, iCloud Keychain sync
- **macOS**: Touch ID, iCloud Keychain sync
- **Android**: Biometric authentication, Google Password Manager sync
- **Windows**: Windows Hello (biometric or PIN)

### Security Key Support

- FIDO2 security keys (YubiKey, Titan Key, etc.)
- USB, NFC, and Bluetooth security keys
- Multiple security keys per user

## WebAuthn Flow

### Registration Flow

1. Client calls `start_registration()` to get creation challenge
2. Browser calls `navigator.credentials.create()` with challenge
3. User verifies with biometric or PIN
4. Client calls `finish_registration()` with attestation response
5. Server verifies and stores credential

### Authentication Flow

1. Client calls `start_authentication()` to get authentication challenge
2. Browser calls `navigator.credentials.get()` with challenge
3. User verifies with biometric or PIN
4. Client calls `finish_authentication()` with assertion response
5. Server verifies signature and counter, updates last used timestamp

## Browser Compatibility

- Chrome 67+ (WebAuthn Level 1)
- Firefox 60+ (WebAuthn Level 1)
- Safari 13+ (WebAuthn Level 1)
- Edge 18+ (WebAuthn Level 1)
- Chrome 93+ (WebAuthn Level 2 - conditional UI)
- Safari 16+ (WebAuthn Level 3 - passkey sync)

## Requirements

Implements requirements:

- REQ-AUTH-005 (Passwordless authentication - MANDATORY)
- REQ-WEBAUTHN-001 through REQ-WEBAUTHN-010 (WebAuthn features)
- REQ-SEC-010 (Phishing-resistant authentication)
- REQ-SEC-011 (Origin binding)
- REQ-SEC-012 (Replay attack prevention)
