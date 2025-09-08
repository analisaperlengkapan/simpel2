# Admin policy - full access to all secrets and admin operations
path "*" {
  capabilities = ["create", "read", "update", "delete", "list", "sudo"]
}
