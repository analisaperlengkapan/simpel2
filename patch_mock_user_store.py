import re

with open("layanan/authenc/crates/core/src/services/user_management_service.rs", "r") as f:
    content = f.read()

# We need to implement search_users and count_search_users in MockUserStore because we call it
# Check if it exists first
if "async fn search_users" not in content.split("impl UserStore for MockUserStore")[1]:
    mock_impl = """
        async fn email_exists(&self, email: &str, realm_id: RealmId) -> Result<bool> {
            let users_by_email = self.users_by_email.lock().await;
            Ok(users_by_email.contains_key(&(email.to_string(), realm_id)))
        }

        async fn count_users(&self, realm_id: RealmId) -> Result<i64> {
            let users = self.users.lock().await;
            Ok(users.values().filter(|u| u.realm_id == Some(*realm_id.as_uuid())).count() as i64)
        }

        async fn count_enabled_users(&self, realm_id: RealmId) -> Result<i64> {
            let users = self.users.lock().await;
            Ok(users.values().filter(|u| u.realm_id == Some(*realm_id.as_uuid()) && u.enabled).count() as i64)
        }

        async fn list_users_filtered(&self, realm_id: RealmId, enabled: Option<bool>, offset: usize, limit: usize) -> Result<Vec<User>> {
            let users = self.users.lock().await;
            let mut realm_users: Vec<User> = users
                .values()
                .filter(|u| u.realm_id == Some(*realm_id.as_uuid()) && enabled.map_or(true, |e| u.enabled == e))
                .cloned()
                .collect();
            realm_users.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            Ok(realm_users.into_iter().skip(offset).take(limit).collect())
        }

        async fn count_users_filtered(&self, realm_id: RealmId, enabled: Option<bool>) -> Result<i64> {
            let users = self.users.lock().await;
            Ok(users.values().filter(|u| u.realm_id == Some(*realm_id.as_uuid()) && enabled.map_or(true, |e| u.enabled == e)).count() as i64)
        }

        async fn search_users(
            &self,
            realm_id: RealmId,
            query: &str,
            offset: usize,
            limit: usize,
        ) -> Result<Vec<User>> {
            let users = self.users.lock().await;
            let query_lower = query.to_lowercase();
            let mut matched_users: Vec<User> = users
                .values()
                .filter(|u| u.realm_id == Some(*realm_id.as_uuid()))
                .filter(|u| {
                    u.username.to_lowercase().contains(&query_lower) ||
                    u.email.to_lowercase().contains(&query_lower) ||
                    u.nip.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                    u.nama.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false)
                })
                .cloned()
                .collect();

            matched_users.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            Ok(matched_users.into_iter().skip(offset).take(limit).collect())
        }

        async fn count_search_users(&self, realm_id: RealmId, query: &str) -> Result<i64> {
            let users = self.users.lock().await;
            let query_lower = query.to_lowercase();
            Ok(users
                .values()
                .filter(|u| u.realm_id == Some(*realm_id.as_uuid()))
                .filter(|u| {
                    u.username.to_lowercase().contains(&query_lower) ||
                    u.email.to_lowercase().contains(&query_lower) ||
                    u.nip.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false) ||
                    u.nama.as_ref().map(|n| n.to_lowercase().contains(&query_lower)).unwrap_or(false)
                })
                .count() as i64)
        }
"""
    old_impl = """
        async fn email_exists(&self, email: &str, realm_id: RealmId) -> Result<bool> {
            let users_by_email = self.users_by_email.lock().await;
            Ok(users_by_email.contains_key(&(email.to_string(), realm_id)))
        }
"""
    content = content.replace(old_impl, mock_impl)

    with open("layanan/authenc/crates/core/src/services/user_management_service.rs", "w") as f:
        f.write(content)
