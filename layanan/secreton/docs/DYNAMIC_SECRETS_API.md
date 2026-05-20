# Dynamic Secrets API Documentation

## Overview

The Dynamic Secrets Engine provides on-demand generation of database credentials with automatic lease management and revocation. This eliminates the need for long-lived credentials and provides better security through short-lived, automatically rotated credentials.

## Features

- **On-Demand Credential Generation**: Generate database credentials when needed
- **Automatic Lease Management**: Credentials automatically expire after TTL
- **Credential Rotation**: Rotate credentials without downtime
- **Multiple Database Support**: PostgreSQL, MySQL, MongoDB, and more
- **Audit Logging**: All credential operations are logged
- **Namespace Isolation**: Credentials scoped to namespaces
- **Rate Limiting**: Prevent credential generation abuse

## Supported Databases

- ✅ PostgreSQL (fully implemented)
- 🚧 MySQL (planned)
- 🚧 MongoDB (planned)
- 🚧 Redis (planned)
- 🚧 Cassandra (planned)
- 🚧 MSSQL (planned)

## REST API Endpoints

### Generate Database Credentials

Generate temporary database credentials for a role.

**Endpoint**: `GET /v1/dynamic/database/creds/:role`

**Query Parameters**:

- `ttl` (optional): Time-to-live in seconds (default: role's default_ttl)

**Response**:

```json
{
  "success": true,
  "data": {
    "lease_id": "lease_abc123",
    "lease_duration": 3600,
    "renewable": true,
    "data": {
      "username": "v-readonly-a1b2c3d4",
      "password": "super-secure-random-password",
      "connection_url": "postgresql://v-readonly-a1b2c3d4:super-secure-random-password@localhost:5432/mydb",
      "database": "mydb",
      "role": "readonly"
    }
  }
}
```

**Example**:

```bash
curl -X GET http://localhost:8200/v1/dynamic/database/creds/readonly?ttl=7200 \
  -H "Authorization: Bearer $TOKEN"
```

### Create Database Role

Create a new database role configuration.

**Endpoint**: `POST /v1/dynamic/database/roles/:role`

**Request Body**:

```json
{
  "db_name": "postgres-prod",
  "default_ttl": 3600,
  "max_ttl": 86400,
  "creation_statements": [
    "CREATE USER '{{username}}' WITH PASSWORD '{{password}}'",
    "GRANT SELECT ON ALL TABLES IN SCHEMA public TO '{{username}}'"
  ],
  "revocation_statements": [
    "DROP USER IF EXISTS '{{username}}'"
  ],
  "rotation_statements": [
    "ALTER USER '{{username}}' WITH PASSWORD '{{password}}'"
  ],
  "renew_statements": []
}
```

**Response**:

```json
{
  "success": true,
  "data": {
    "name": "readonly",
    "db_name": "postgres-prod",
    "default_ttl": 3600,
    "max_ttl": 86400
  }
}
```

**Example**:

```bash
curl -X POST http://localhost:8200/v1/dynamic/database/roles/readonly \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @role-config.json
```

### List Database Roles

List all configured database roles.

**Endpoint**: `GET /v1/dynamic/database/roles`

**Response**:

```json
{
  "success": true,
  "data": ["readonly", "readwrite", "admin"]
}
```

### Get Database Role

Get details of a specific database role.

**Endpoint**: `GET /v1/dynamic/database/roles/:role`

**Response**:

```json
{
  "success": true,
  "data": {
    "name": "readonly",
    "db_name": "postgres-prod",
    "default_ttl": 3600,
    "max_ttl": 86400
  }
}
```

### Update Database Role

Update an existing database role configuration.

**Endpoint**: `PUT /v1/dynamic/database/roles/:role`

**Request Body**: Same as Create Database Role

### Delete Database Role

Delete a database role and revoke all active credentials.

**Endpoint**: `DELETE /v1/dynamic/database/roles/:role`

**Response**:

```json
{
  "success": true,
  "data": {
    "deleted": true,
    "role": "readonly"
  }
}
```

### Configure Database Connection

Configure a database connection for use with dynamic secrets.

**Endpoint**: `POST /v1/dynamic/database/config/:name`

**Request Body**:

```json
{
  "db_type": "postgresql",
  "connection_url": "postgresql://admin:password@localhost:5432/mydb",
  "max_open_connections": 4,
  "max_idle_connections": 2,
  "max_connection_lifetime": 3600,
  "verify_connection": true,
  "root_rotation_statements": [
    "ALTER USER admin WITH PASSWORD '{{password}}'"
  ]
}
```

**Response**:

```json
{
  "success": true,
  "data": {
    "name": "postgres-prod",
    "db_type": "postgresql",
    "verified": true
  }
}
```

**Example**:

```bash
curl -X POST http://localhost:8200/v1/dynamic/database/config/postgres-prod \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @connection-config.json
```

### Get Database Connection

Get details of a database connection.

**Endpoint**: `GET /v1/dynamic/database/config/:name`

### Delete Database Connection

Delete a database connection and all associated roles.

**Endpoint**: `DELETE /v1/dynamic/database/config/:name`

## gRPC API

All REST endpoints have corresponding gRPC methods:

- `GenerateDatabaseCredentials`
- `CreateDatabaseRole`
- `GetDatabaseRole`
- `ListDatabaseRoles`
- `UpdateDatabaseRole`
- `DeleteDatabaseRole`
- `ConfigureDatabaseConnection`

See `infra/proto/secreton.proto` for message definitions.

## PostgreSQL Examples

### Read-Only Role

```json
{
  "db_name": "postgres-prod",
  "default_ttl": 3600,
  "max_ttl": 86400,
  "creation_statements": [
    "CREATE USER {{username}} WITH PASSWORD '{{password}}'",
    "GRANT CONNECT ON DATABASE mydb TO {{username}}",
    "GRANT USAGE ON SCHEMA public TO {{username}}",
    "GRANT SELECT ON ALL TABLES IN SCHEMA public TO {{username}}",
    "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO {{username}}"
  ],
  "revocation_statements": [
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM {{username}}",
    "REVOKE USAGE ON SCHEMA public FROM {{username}}",
    "REVOKE CONNECT ON DATABASE mydb FROM {{username}}",
    "DROP USER IF EXISTS {{username}}"
  ]
}
```

### Read-Write Role

```json
{
  "db_name": "postgres-prod",
  "default_ttl": 1800,
  "max_ttl": 7200,
  "creation_statements": [
    "CREATE USER {{username}} WITH PASSWORD '{{password}}'",
    "GRANT CONNECT ON DATABASE mydb TO {{username}}",
    "GRANT USAGE ON SCHEMA public TO {{username}}",
    "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO {{username}}",
    "GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO {{username}}",
    "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO {{username}}",
    "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT USAGE, SELECT ON SEQUENCES TO {{username}}"
  ],
  "revocation_statements": [
    "REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public FROM {{username}}",
    "REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA public FROM {{username}}",
    "REVOKE USAGE ON SCHEMA public FROM {{username}}",
    "REVOKE CONNECT ON DATABASE mydb FROM {{username}}",
    "DROP USER IF EXISTS {{username}}"
  ],
  "rotation_statements": [
    "ALTER USER {{username}} WITH PASSWORD '{{password}}'"
  ]
}
```

### Admin Role (with time-based access)

```json
{
  "db_name": "postgres-prod",
  "default_ttl": 900,
  "max_ttl": 3600,
  "creation_statements": [
    "CREATE USER {{username}} WITH PASSWORD '{{password}}' SUPERUSER",
    "GRANT ALL PRIVILEGES ON DATABASE mydb TO {{username}}"
  ],
  "revocation_statements": [
    "REVOKE ALL PRIVILEGES ON DATABASE mydb FROM {{username}}",
    "DROP USER IF EXISTS {{username}}"
  ]
}
```

## Security Considerations

### SQL Injection Prevention

The API performs basic SQL injection detection:

- Blocks statements containing `;--`, `/*`, `*/`
- Blocks dangerous keywords: `DROP DATABASE`, `DROP TABLE`, `TRUNCATE`, `DELETE FROM`
- Blocks stored procedure execution: `xp_`, `sp_`, `EXEC`, `EXECUTE`

**Important**: Always validate and sanitize creation/revocation statements before deployment.

### Placeholder Requirements

- Creation statements MUST contain `{{username}}` or `{{password}}` placeholders
- Revocation statements MUST contain `{{username}}` placeholder
- Rotation statements MUST contain `{{username}}` and `{{password}}` placeholders

### TTL Validation

- `default_ttl` must be greater than 0
- `default_ttl` must be less than or equal to `max_ttl`
- Requested TTL cannot exceed role's `max_ttl`

### Credential Format

- Username format: `v-{role}-{random}` (e.g., `v-readonly-a1b2c3d4`)
- Password: 32 characters, cryptographically secure random (using OsRng)
- Character set: A-Z, a-z, 0-9, special characters (!@#$%^&*)

## Lease Management

### Automatic Expiration

Credentials automatically expire after TTL. The lease manager runs a background task that:

1. Checks for expired leases every 60 seconds
2. Revokes expired credentials automatically
3. Executes revocation statements to drop database users
4. Logs all revocation events

### Lease Renewal

Credentials can be renewed before expiration:

```bash
curl -X POST http://localhost:8200/v1/sys/leases/renew \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"lease_id": "lease_abc123", "increment": 1800}'
```

### Manual Revocation

Credentials can be manually revoked:

```bash
curl -X POST http://localhost:8200/v1/sys/leases/revoke \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"lease_id": "lease_abc123"}'
```

## Audit Logging

All dynamic secrets operations are logged:

- `database_connection_configured`: Connection created/updated
- `database_connection_deleted`: Connection deleted
- `database_role_created`: Role created
- `database_role_updated`: Role updated
- `database_role_deleted`: Role deleted
- `dynamic_secret_generated`: Credentials generated
- `dynamic_secret_revoked`: Credentials revoked

Audit logs include:

- Timestamp
- User ID
- Operation
- Resource path
- Metadata (role name, TTL, lease ID, etc.)

## Metrics

The following Prometheus metrics are exposed:

- `secreton_dynamic_credentials_generated_total`: Total credentials generated
- `secreton_dynamic_credentials_revoked_total`: Total credentials revoked
- `secreton_dynamic_active_leases`: Current active leases
- `secreton_dynamic_credential_generation_duration_seconds`: Time to generate credentials
- `secreton_dynamic_role_operations_total`: Role CRUD operations

## Rate Limiting

Credential generation is rate-limited to prevent abuse:

- Default: 10 requests per minute per user
- Configurable via API configuration
- Returns HTTP 429 (Too Many Requests) when limit exceeded

## Namespace Isolation

Dynamic secrets are scoped to namespaces:

- Roles are created within a namespace
- Credentials can only be generated for roles in accessible namespaces
- Hierarchical access control based on satker/wilayah structure

## Error Handling

### Common Errors

**400 Bad Request**:

- Invalid role configuration
- Missing required fields
- SQL injection detected
- Invalid TTL values

**403 Forbidden**:

- Insufficient permissions
- Namespace access denied

**404 Not Found**:

- Role not found
- Connection not found

**500 Internal Server Error**:

- Database connection failed
- Credential generation failed
- Storage error

### Error Response Format

```json
{
  "success": false,
  "error": {
    "code": "INVALID_CONFIG",
    "message": "Creation statements must contain {{username}} or {{password}} placeholders",
    "details": {}
  }
}
```

## Best Practices

1. **Use Short TTLs**: Keep TTLs as short as practical (e.g., 1 hour for read-only, 15 minutes for admin)
2. **Principle of Least Privilege**: Grant only necessary permissions in creation statements
3. **Test Statements**: Test creation/revocation statements in a dev environment first
4. **Monitor Usage**: Track credential generation patterns for anomalies
5. **Rotate Root Credentials**: Regularly rotate database root credentials
6. **Use Connection Pooling**: Configure appropriate connection pool sizes
7. **Enable Audit Logging**: Always enable audit logging for compliance
8. **Implement Alerts**: Set up alerts for unusual credential generation patterns

## Troubleshooting

### Credentials Not Working

1. Check if credentials have expired (check lease expiration)
2. Verify creation statements executed successfully (check audit logs)
3. Test database connectivity from Secreton server
4. Verify user has necessary permissions in database

### Connection Failures

1. Verify connection URL is correct
2. Check network connectivity to database
3. Verify database credentials are valid
4. Check firewall rules

### Role Creation Fails

1. Verify database connection exists
2. Check creation statements for syntax errors
3. Verify placeholders are present
4. Check for SQL injection patterns

## Migration from Static Credentials

1. **Audit Current Access**: Document all applications using static credentials
2. **Create Roles**: Create dynamic secret roles matching current permissions
3. **Update Applications**: Modify applications to fetch credentials from Secreton
4. **Implement Renewal**: Add lease renewal logic to applications
5. **Test Thoroughly**: Test in dev/staging before production
6. **Gradual Rollout**: Migrate applications one at a time
7. **Revoke Static Credentials**: After successful migration, revoke old static credentials

## Future Enhancements

- [ ] MySQL support
- [ ] MongoDB support
- [ ] Redis support
- [ ] Cassandra support
- [ ] MSSQL support
- [ ] Cloud database support (AWS RDS, Azure SQL, Google Cloud SQL)
- [ ] Certificate-based authentication
- [ ] Custom username templates
- [ ] Credential caching
- [ ] Batch credential generation
- [ ] Webhook notifications on credential events

## References

- [HashiCorp Secret Vault Database Secrets Engine](https://www.engineproject.io/docs/secrets/databases)
- [PostgreSQL User Management](https://www.postgresql.org/docs/current/user-manag.html)
- [Secreton API Documentation](./API.md)
- [Lease Management](./LEASE_MANAGEMENT.md)
