# authenc-iam-api

IAM Administration REST API for Authenc identity provider.

## Purpose

This crate provides admin HTTP endpoints for the Portal IAM Microfrontend:

- **User Management**: CRUD operations, password reset, MFA setup
- **Realm Management**: Multi-realm configuration
- **Client Management**: OAuth2 client registration and configuration
- **Role Management**: Role and permission assignment
- **Federation Management**: External identity provider configuration
- **Audit Logs**: Query and export audit logs
- **System Configuration**: System-wide settings

## Endpoints

### User Management
- `GET /api/v1/iam/users` - List users with pagination
- `POST /api/v1/iam/users` - Create user
- `GET /api/v1/iam/users/{id}` - Get user details
- `PUT /api/v1/iam/users/{id}` - Update user
- `DELETE /api/v1/iam/users/{id}` - Delete user
- `POST /api/v1/iam/users/{id}/password/reset` - Reset password
- `POST /api/v1/iam/users/{id}/mfa/enable` - Enable MFA

### Realm Management
- `GET /api/v1/iam/realms` - List realms
- `POST /api/v1/iam/realms` - Create realm
- `PUT /api/v1/iam/realms/{id}` - Update realm
- `DELETE /api/v1/iam/realms/{id}` - Delete realm

### Client Management
- `GET /api/v1/iam/clients` - List OAuth2 clients
- `POST /api/v1/iam/clients` - Create client
- `PUT /api/v1/iam/clients/{id}` - Update client
- `DELETE /api/v1/iam/clients/{id}` - Delete client
- `POST /api/v1/iam/clients/{id}/secret/regenerate` - Regenerate secret

### Role Management
- `GET /api/v1/iam/roles` - List roles
- `POST /api/v1/iam/roles` - Create role
- `POST /api/v1/iam/users/{user_id}/roles/{role_id}` - Assign role

### Federation Management
- `GET /api/v1/iam/identity-providers` - List identity providers
- `POST /api/v1/iam/identity-providers` - Create identity provider

### Audit Logs
- `GET /api/v1/iam/audit-logs` - List audit logs with filters
- `GET /api/v1/iam/audit-logs/export` - Export audit logs

## Security

- Admin authentication middleware
- Permission-based authorization
- Audit logging for all admin actions

## Requirements

Implements requirements:
- REQ-API-002 (IAM Admin API)
- REQ-PORTAL-010 through REQ-PORTAL-016 (IAM admin features)
- REQ-AUDIT-003 (Audit log querying)
