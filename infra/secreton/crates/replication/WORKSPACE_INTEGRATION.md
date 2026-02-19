# Workspace Integration Instructions

The `secreton-replication` crate has been created but needs to be added to the root workspace.

## Required Changes

Add the following line to `/home/anbud02/simpel2/Cargo.toml` in the `[workspace.members]` section:

```toml
[workspace.members]
# ... existing members ...
"infra/secreton/crates/replication",
```

Also add to the `[workspace.dependencies]` section (around line 254):

```toml
# Secreton Internal Dependencies (for sub-crates)
secreton-core = { path = "infra/secreton/crates/core" }
secreton-api = { path = "infra/secreton/crates/api" }
secreton-storage = { path = "infra/secreton/crates/storage" }
secreton-replication = { path = "infra/secreton/crates/replication" }  # ADD THIS LINE
```

## Verification

After making these changes, verify the crate compiles:

```bash
cd /home/anbud02/simpel2
cargo check -p secreton-replication
```

## Testing

Run the unit tests:

```bash
cargo test -p secreton-replication
```
