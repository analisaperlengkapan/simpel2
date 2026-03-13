import re

# Bug 1: SQL LIKE pattern not escaping metacharacters
with open("layanan/authenc/crates/storage/src/stores/user_store.rs", "r") as f:
    content = f.read()

# Fix search_users
old_search_users = """
        let pattern = format!("%{}%", query_str.to_lowercase());
        let query = r#"
            SELECT *
"""
new_search_users = """
        let escaped = query_str.to_lowercase().replace('\\\\', "\\\\\\\\").replace('%', "\\\\%").replace('_', "\\\\_");
        let pattern = format!("%{}%", escaped);
        let query = r#"
            SELECT *
"""
if old_search_users in content:
    content = content.replace(old_search_users, new_search_users)

# Fix count_search_users
old_count_search_users = """
        let pattern = format!("%{}%", query_str.to_lowercase());
        let query = r#"
            SELECT COUNT(*)
"""
new_count_search_users = """
        let escaped = query_str.to_lowercase().replace('\\\\', "\\\\\\\\").replace('%', "\\\\%").replace('_', "\\\\_");
        let pattern = format!("%{}%", escaped);
        let query = r#"
            SELECT COUNT(*)
"""
if old_count_search_users in content:
    content = content.replace(old_count_search_users, new_count_search_users)

# Bug 6: list_users does not filter deleted_at IS NULL
old_list_users = """
        let query = r#"
            SELECT *
            FROM users
            WHERE realm_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
        "#;
"""
new_list_users = """
        let query = r#"
            SELECT *
            FROM users
            WHERE realm_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
        "#;
"""
if old_list_users in content:
    content = content.replace(old_list_users, new_list_users)

with open("layanan/authenc/crates/storage/src/stores/user_store.rs", "w") as f:
    f.write(content)


# Bug 3 & 4: Fix division to use satker_code and return None instead of "Bagian Umum"
with open("antarmuka/portal/src/features/auth.rs", "r") as f:
    content = f.read()

old_div_frontend = """
            division: claims
                .jabatan
                .unwrap_or_else(|| "Bagian Umum".to_string()),
"""
new_div_frontend = """
            division: claims
                .satker_code
                .unwrap_or_else(|| "Bagian Umum".to_string()),
"""
if old_div_frontend in content:
    content = content.replace(old_div_frontend, new_div_frontend)

with open("antarmuka/portal/src/features/auth.rs", "w") as f:
    f.write(content)

with open("layanan/authenc/crates/api/src/handlers/auth.rs", "r") as f:
    content = f.read()

old_div_backend1 = """
                    division: user.jabatan.clone().or(Some("Bagian Umum".to_string())),
"""
new_div_backend1 = """
                    division: Some(user.satker_code.clone()).filter(|s| !s.is_empty()),
"""
if old_div_backend1 in content:
    content = content.replace(old_div_backend1, new_div_backend1)

# Because we did multiple replace earlier, we need to check if there's any other "division: user.jabatan"
old_div_backend2 = """
        division: user.jabatan.clone().or(Some("Bagian Umum".to_string())),
"""
new_div_backend2 = """
        division: Some(user.satker_code.clone()).filter(|s| !s.is_empty()),
"""
if old_div_backend2 in content:
    content = content.replace(old_div_backend2, new_div_backend2)

with open("layanan/authenc/crates/api/src/handlers/auth.rs", "w") as f:
    f.write(content)
