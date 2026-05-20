# Secreton Policy Examples

This directory contains example policy files demonstrating various access control patterns in Secreton.

## Policy File Format

Secreton policies are defined in TOML format. Each policy file contains:

- **name**: Unique identifier for the policy
- **description**: (Optional) Human-readable description
- **namespace**: Namespace scope (default: "default")
- **rules**: Array of policy rules defining access control

### Policy Rule Structure

Each rule contains:

- **effect**: Either "allow" or "deny"
- **path**: Resource path pattern (supports wildcards with `*`)
- **capabilities**: Array of allowed operations
- **condition**: (Optional) Additional constraints
- **mfa**: (Optional) Require multi-factor authentication

### Available Capabilities

- `create` - Create new resources
- `read` - Read existing resources
- `update` - Modify existing resources
- `delete` - Remove resources
- `list` - List resources in a path
- `sudo` - Administrative operations
- `*` - All capabilities (cannot be combined with others)

### Path Patterns

Paths use forward slashes and support wildcards:

- `secret/data/myapp` - Exact path match
- `secret/data/myapp/*` - All paths under myapp
- `secret/*` - All paths under secret

### Conditions

Rules can include optional conditions:

```toml
[rules.condition]
# Time-based access
time_range = { start = "2025-01-01T00:00:00Z", end = "2025-12-31T23:59:59Z" }

# IP-based access (supports CIDR notation)
allowed_ips = ["192.168.1.0/24", "10.0.0.1"]

# Custom claims (JSON object)
required_claims = { department = "engineering" }
```

## Example Policies

### read-only.toml

Basic read-only access to secrets. Useful for applications that only need to retrieve secrets.

### admin.toml

Full administrative access with sudo capability. For engine administrators.

### database-secrets.toml

Scoped access to database credentials with MFA requirement. Demonstrates path-specific access.

### transit-only.toml

Access limited to encryption/decryption operations. For applications using Secreton as a cryptographic service.

### namespace-scoped.toml

Demonstrates namespace isolation for multi-tenant environments.

## Using Policy Files

### Apply a policy

```bash
secreton policy write my-policy examples/policies/read-only.toml
```

### Validate before applying

```bash
secreton policy validate examples/policies/read-only.toml
```

### Format a policy file

```bash
secreton policy fmt examples/policies/my-policy.toml
```

### Test policy evaluation

```bash
secreton policy test my-policy --path secret/data/myapp --action read
```

## Best Practices

1. **Principle of Least Privilege**: Grant only the minimum required access
2. **Use Wildcards Carefully**: Overly broad wildcards can grant unintended access
3. **Combine Allow and Deny**: Use deny rules to create exceptions in broader allow rules
4. **Enable MFA for Sensitive Paths**: Require MFA for administrative or sensitive operations
5. **Use Namespaces**: Isolate different teams or applications using namespaces
6. **Document Policies**: Include clear descriptions explaining the policy's purpose
7. **Test Before Applying**: Always validate and test policies before production use

## Policy Evaluation Order

1. Deny rules are evaluated first
2. If any deny rule matches, access is denied
3. Allow rules are then evaluated
4. If any allow rule matches, access is granted
5. If no rules match, access is denied (default deny)

## Additional Resources

- [Secreton Documentation](../../README.md)
- [Policy Management CLI Guide](../../crates/cli/README.md)
- [Security Best Practices](../../docs/SECURITY.md)
