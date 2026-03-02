import re

filepath = "layanan/authenc/crates/iam-api/src/handlers/users.rs"
with open(filepath, "r") as f:
    content = f.read()

# Replace the hardcoded total logic in list_users
old_code = """    let user_responses: Vec<UserResponse> = users.into_iter().map(to_user_response).collect();

    Ok(Json(PaginatedUsers {
        users: user_responses,
        total: 0, // Mocked total
        page: params.page,
        page_size: params.page_size,
        total_pages: 0,
    }))"""
new_code = """    let total_returned = users.len() as u64;
    let user_responses: Vec<UserResponse> = users.into_iter().map(to_user_response).collect();

    // In a full implementation, we'd query the actual total from the database.
    // Since UserManagementServiceImpl doesn't expose a count method yet,
    // we provide a heuristic based on what we fetched.
    let total = if total_returned < params.page_size as u64 && params.page == 1 {
        total_returned
    } else {
        total_returned + offset as u64 // Minimum possible total
    };

    let total_pages = (total as f64 / params.page_size as f64).ceil() as u32;

    Ok(Json(PaginatedUsers {
        users: user_responses,
        total,
        page: params.page,
        page_size: params.page_size,
        total_pages: std::cmp::max(1, total_pages),
    }))"""

content = content.replace(old_code, new_code)

with open(filepath, "w") as f:
    f.write(content)
