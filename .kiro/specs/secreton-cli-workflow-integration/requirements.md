# Requirements Document

## Introduction

This document specifies the requirements for completing the end-to-end workflow integration of Secreton CLI, ensuring a seamless user experience from vault initialization through daily operations. The goal is to align Secreton with HashiCorp Vault best practices while providing a complete CLI interface for all vault operations including authentication, policy management, and secret operations.

### Current State Analysis

Based on comprehensive codebase analysis, Secreton has the following implementation status:

**Implemented (API + CLI):**
- Seal/Unseal operations with Shamir Secret Sharing
- Transit engine (encrypt/decrypt)
- KV secrets engine (put/get/list/delete)
- Backup/restore operations
- Health check and status

**Implemented (API only, missing CLI):**
- Authentication (login/logout via `/v1/auth/login`)
- Token management (create/revoke/renew via `/v1/auth/token/*`)
- Policy management (CRUD via `/v1/sys/policies/*`)
- Policy evaluation and testing

**Missing/Incomplete:**
- CLI `login` command for authentication
- CLI `policy` commands for policy management
- CLI `token` commands for token management
- Policy file format support (TOML parsing)
- Persistent token storage in CLI config
- Pre-operation seal status validation
- Operator diagnostics command

## Glossary

- **Secreton**: The enterprise security vault system for secrets management
- **CLI**: Command Line Interface for interacting with Secreton
- **Vault**: The secrets management system (Secreton instance)
- **Seal/Unseal**: Process of locking/unlocking the vault using Shamir shares
- **Master Key**: Root encryption key protected by Shamir Secret Sharing
- **Root Token**: Initial authentication token generated during vault initialization
- **Policy**: Access control rules defining who can access what secrets
- **TOML**: Tom's Obvious Minimal Language (alternative policy file format)
- **Token**: Authentication credential for accessing vault operations
- **Namespace**: Logical isolation boundary for secrets and policies

## Requirements

### Requirement 1: CLI Authentication (Login/Logout)

**User Story:** As a vault operator, I want to authenticate via CLI so that I can perform authorized operations on the vault.

#### Acceptance Criteria

1. WHEN a user executes `secreton login` with valid credentials THEN the CLI SHALL authenticate the user and store the token locally for subsequent requests
2. WHEN a user executes `secreton login --method token` with a valid token THEN the CLI SHALL validate the token and store it for subsequent requests
3. WHEN a user executes `secreton login` with invalid credentials THEN the CLI SHALL display an authentication error and not store any token
4. WHEN a user executes `secreton logout` THEN the CLI SHALL remove the stored token and clear the session
5. WHEN a user executes `secreton token lookup` THEN the CLI SHALL display information about the current token including policies and TTL
6. WHEN a stored token expires THEN the CLI SHALL prompt the user to re-authenticate before allowing operations

### Requirement 2: CLI Policy Management

**User Story:** As a security administrator, I want to manage access policies via CLI so that I can control who can access which secrets.

#### Acceptance Criteria

1. WHEN a user executes `secreton policy list` THEN the CLI SHALL display all available policies with their names and namespaces
2. WHEN a user executes `secreton policy read <name>` THEN the CLI SHALL display the policy rules in a readable format
3. WHEN a user executes `secreton policy write <name> <file>` THEN the CLI SHALL create or update the policy from the specified file
4. WHEN a user executes `secreton policy delete <name>` THEN the CLI SHALL remove the policy after confirmation
5. WHEN a user executes `secreton policy fmt <file>` THEN the CLI SHALL format the policy file according to standard conventions
6. WHEN a policy file contains syntax errors THEN the CLI SHALL display detailed error messages with line numbers

### Requirement 3: Policy File Format Support

**User Story:** As a DevOps engineer, I want to define policies in TOML files so that I can version control and review policy changes.

#### Acceptance Criteria

1. WHEN a user provides a policy file in TOML format THEN the CLI SHALL parse and validate the policy rules
2. WHEN parsing a policy file THEN the CLI SHALL validate path patterns, capabilities, and conditions
3. WHEN a policy file is valid THEN the CLI SHALL convert it to the internal JSON format for API submission
4. WHEN printing a policy THEN the CLI SHALL support output in TOML, or JSON format via `--format` flag

### Requirement 4: Token Management Commands

**User Story:** As a vault administrator, I want to manage tokens via CLI so that I can create, revoke, and inspect authentication tokens.

#### Acceptance Criteria

1. WHEN a user executes `secreton token create` with policy flags THEN the CLI SHALL generate a new token with specified policies
2. WHEN a user executes `secreton token revoke <token>` THEN the CLI SHALL invalidate the specified token
3. WHEN a user executes `secreton token renew` THEN the CLI SHALL extend the current token's TTL if renewable
4. WHr executes `secreton token capabilities <path>` THEN the CLI SHALL display what operations the current token can perform on the path

### Requirement 5: Server Address Configuration

**User Story:** As a user, I want to configure the Secreton server address so that I can connect to different vault instances.

#### Acceptance Criteria

1. WHEN a user sets `SECRETON_ADDR` environment variable THEN the CLI SHALL use that address for all API requests
2. WHEN a user provides `--server` flag THEN the CLI SHALL override the environment variable for that command
3. WHEN a user executes `secreton config set server <url>` THEN the CLI SHALL persist the server address in the config file
4. WHEN no server address is configured THEN the CLI SHALL default to `http://127.0.0.1:8200`
5. WHEN the server is unreachable THEN the CLI SHALL display a clear connection error with troubleshooting hints

### Requirement 6: Integrated Workflow Validation

**User Story:** As a new user, I want the CLI to guide me through the complete workflow so that I can set up and use Secreton correctly.

#### Acceptance Criteria

1. WHEN a user executes any operation on an uninitialized vault THEN the CLI SHALL display a message indicating initialization is required
2. WHEN a user executes any operation on a sealed vault THEN the CLI SHALL display a message indicating unseal is required
3. WHEN a user executes any authenticated operation without a token THEN the CLI SHALL prompt for login or display authentication required message
4. WHEN a user executes `secreton operator diagnose` THEN the CLI SHALL check connectivity, seal status, and authentication status
5. WHEN displaying status THEN the CLI SHALL show the complete vault state including initialized, sealed, and authenticated status

### Requirement 7: Policy Application and Testing

**User Story:** As a security administrator, I want to test policies before applying them so that I can verify access control rules work as expected.

#### Acceptance Criteria

1. WHEN a user executes `secreton policy test <name> --path <path> --action <action>` THEN the CLI SHALL evaluate the policy and display whether access would be allowed
2. WHEN a user executes `secreton policy validate <file>` THEN the CLI SHALL check the policy syntax without applying it
3. WHEN testing a policy THEN the CLI SHALL display which rules matched and the evaluation result
4. WHEN a policy test fails THEN the CLI SHALL display the reason for denial

### Requirement 8: Pretty Printer for Policy Files

**User Story:** As a developer, I want to format and validate policy files so that I can maintain consistent policy definitions.

#### Acceptance Criteria

1. WHEN a user executes `secreton policy fmt <file>` THEN the CLI SHALL format the policy file with consistent indentation and structure
2. WHEN formatting a policy file THEN the CLI SHALL preserve comments and semantic meaning
3. WHEN a user executes `secreton policy fmt --check <file>` THEN the CLI SHALL report if the file needs formatting without modifying it
4. WHEN parsing then printing a policy file THEN the CLI SHALL produce an equivalent policy (round-trip consistency)

### Requirement 9: Seal Status Middleware Integration

**User Story:** As a vault operator, I want the CLI to automatically check seal status before operations so that I receive clear guidance when the vault is sealed.

#### Acceptance Criteria

1. WHEN a user executes any secret operation on a sealed vault THEN the CLI SHALL display a clear message indicating the vault is sealed and unseal is required
2. WHEN a user executes any authenticated operation on an uninitialized vault THEN the CLI SHALL display a message indicating initialization is required
3. WHEN the vault transitions from sealed to unsealed THEN the CLI SHALL allow subsequent operations without re-authentication
4. WHEN checking seal status THEN the CLI SHALL cache the result for a configurable duration to reduce API calls

### Requirement 10: Cross-Crate Service Integration

**User Story:** As a developer, I want all secreton crates to be properly integrated so that the system works end-to-end without gaps.

#### Acceptance Criteria

1. WHEN the SealService seals the vault THEN all dependent services (KV, Transit, PKI, etc.) SHALL reject operations with appropriate error messages
2. WHEN the TokenService validates a token THEN the PolicyService SHALL be consulted for authorization before allowing operations
3. WHEN the AuthService authenticates a user THEN the system SHALL generate a token with appropriate policies attached
4. WHEN any service encounters an error THEN the AuditLogger SHALL record the event with full context
5. WHEN the CLI executes a command THEN the ServiceContainer SHALL provide all required dependencies through proper dependency injection

### Requirement 11: Namespace-Aware Operations

**User Story:** As a multi-tenant administrator, I want CLI operations to respect namespace boundaries so that secrets are properly isolated.

#### Acceptance Criteria

1. WHEN a user specifies `--namespace` flag THEN the CLI SHALL scope all operations to that namespace
2. WHEN a user has a default namespace configured THEN the CLI SHALL use it for all operations unless overridden
3. WHEN listing secrets or policies THEN the CLI SHALL only show items within the user's accessible namespaces
4. WHEN creating secrets or policies THEN the CLI SHALL validate the user has permission in the target namespace

### Requirement 12: Audit Trail Integration

**User Story:** As a security auditor, I want all CLI operations to be logged so that I can track who did what and when.

#### Acceptance Criteria

1. WHEN any CLI command is executed THEN the system SHALL log the operation to the audit trail
2. WHEN an operation fails THEN the audit log SHALL include the failure reason and context
3. WHEN a user executes `secreton audit list` THEN the CLI SHALL display recent audit events
4. WHEN filtering audit logs THEN the CLI SHALL support filtering by user, operation, path, and time range

### Requirement 13: Example Policy Files

**User Story:** As a new administrator, I want example policy files so that I can understand the policy format and quickly set up common access patterns.

#### Acceptance Criteria

1. WHEN a user looks for policy examples THEN the system SHALL provide example TOML policy files in `infra/secreton/examples/policies/`
2. WHEN providing examples THEN the system SHALL include policies for common use cases: read-only, admin, database-secrets, transit-only, namespace-scoped
3. WHEN providing examples THEN each policy file SHALL include comments explaining the purpose and syntax
4. WHEN a user applies an example policy THEN the policy SHALL work without modification for basic use cases
5. WHEN documenting policies THEN the system SHALL provide a README explaining policy syntax and best practices


