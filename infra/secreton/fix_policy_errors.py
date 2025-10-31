#!/usr/bin/env python3
"""Fix policy.rs specific errors"""

import re
from pathlib import Path

def fix_paginated_response(content):
    """Fix PaginatedResponse structure"""
    # Replace data/total/limit/offset with items/pagination
    content = re.sub(
        r'PaginatedResponse\s*\{\s*data:\s*([^,]+),\s*total:\s*([^,]+),\s*limit:\s*([^,]+),\s*offset:\s*([^,\}]+)\s*\}',
        r'PaginatedResponse::new(\1, \2 as u64, \3 as u32, \4 as u32)',
        content
    )

    return content

def fix_pagination_query(content):
    """Fix PaginationQuery field access"""
    # limit and offset are not Option
    content = re.sub(r'query\.pagination\.limit\.unwrap_or\((\d+)\)', r'\1', content)
    content = re.sub(r'query\.pagination\.offset\.unwrap_or\((\d+)\)', r'\1', content)

    return content

def fix_api_response_data(content):
    """Fix ApiResponse data field"""
    # data field should be Option
    content = re.sub(
        r'ApiResponse\s*\{\s*success:\s*true,\s*data:\s*([^,\n]+),\s*error:',
        r'ApiResponse { success: true, data: Some(\1), error:',
        content
    )

    return content

def main():
    file_path = Path("crates/api/src/handlers/policy.rs")
    if not file_path.exists():
        print(f"File not found: {file_path}")
        return

    content = file_path.read_text()

    # Apply fixes
    content = fix_paginated_response(content)
    content = fix_pagination_query(content)
    content = fix_api_response_data(content)

    # Write back
    file_path.write_text(content)
    print(f"Fixed: {file_path}")

if __name__ == "__main__":
    main()
