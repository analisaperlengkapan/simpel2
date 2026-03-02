import re

filepath = "layanan/authenc/crates/iam-api/src/handlers/users.rs"
with open(filepath, "r") as f:
    content = f.read()

# Replace the roles UUID parsing
old_code = "let roles_uuids = req.roles.map(|rs| rs.into_iter().filter_map(|r| Uuid::parse_str(&r).ok()).collect());"
new_code = """let mut roles_uuids = None;
    if let Some(rs) = req.roles {
        let mut parsed = Vec::new();
        for r in rs {
            match Uuid::parse_str(&r) {
                Ok(uuid) => parsed.push(uuid),
                Err(_) => return Err(crate::error::ApiError(AuthencError::validation(format!("Invalid UUID for role: {}", r)))),
            }
        }
        roles_uuids = Some(parsed);
    }"""

content = content.replace(old_code, new_code)

with open(filepath, "w") as f:
    f.write(content)
