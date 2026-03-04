# Check secreton migrations for hierarchy error
grep -rn "hierarchy" layanan/secreton/crates/storage/migrations/

# Check authenc migrations for token_hash error
grep -rn "token_hash" layanan/authenc/crates/storage/migrations/
