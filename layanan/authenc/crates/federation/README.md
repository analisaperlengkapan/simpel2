# authenc-federation

SSO and external identity provider integration for Authenc.

## Purpose

This crate implements federation and SSO features:

- **Federation Service**: Orchestrate SSO flows with external identity providers
- **OIDC Provider**: Integration with OpenID Connect providers (Google, Microsoft, etc.)
- **SAML Provider**: Integration with SAML 2.0 providers
- **LDAP Provider**: Integration with LDAP/Active Directory (optional)
- **User Linking**: Link external identities to local user accounts
- **Attribute Mapping**: Map external user attributes to local user profile
- **Just-in-Time Provisioning**: Automatically create users on first login

## Supported Providers

### OIDC

- Google
- Microsoft Azure AD
- Generic OIDC providers

### SAML

- SAML 2.0 compliant identity providers
- SP-initiated and IdP-initiated SSO
- Single Logout (SLO)

### LDAP (Optional)

- Active Directory
- OpenLDAP

## Requirements

Implements requirements:

- REQ-FED-001 (External identity providers)
- REQ-FED-002 (Identity brokering)
- REQ-FED-003 (SSO flows)
