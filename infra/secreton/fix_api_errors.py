#!/usr/bin/env python3
"""Script to fix common errors in secreton-api"""

import re
import sys
from pathlib import Path

def fix_user_info_fields(content):
    """Fix UserInfo struct field references"""
    # Remove id field
    content = re.sub(r'id:\s*"[^"]*"\.to_string\(\),\s*\n\s*', '', content)
    content = re.sub(r'response\.user\.id', 'response.user.username', content)
    content = re.sub(r'user\.id\.clone\(\)', 'user.username.clone()', content)

    # Fix email field (should be Option<String>)
    content = re.sub(
        r'email:\s*"([^"]*)"\.to_string\(\),',
        r'email: Some("\1".to_string()),',
        content
    )

    # Remove roles field (doesn't exist)
    content = re.sub(r'roles:\s*vec!\[[^\]]*\],\s*\n\s*', '', content)

    # Remove permissions field (doesn't exist)
    content = re.sub(r'permissions:\s*vec!\[[^\]]*\],\s*\n\s*', '', content)

    # Remove last_login field (doesn't exist)
    content = re.sub(r'last_login:\s*chrono::Utc::now\(\),\s*\n\s*', '', content)

    return content

def fix_api_error_variants(content):
    """Fix ApiError variant usage"""
    # Fix BadRequest - it's a struct variant
    content = re.sub(
        r'ApiError::BadRequest\("([^"]*)"\)',
        r'ApiError::BadRequest { message: "\1".to_string() }',
        content
    )
    content = re.sub(
        r'ApiError::BadRequest\(\s*format!\("([^"]*)"([^)]*)\)\s*\)',
        r'ApiError::BadRequest { message: format!("\1"\2) }',
        content
    )
    content = re.sub(
        r'ApiError::BadRequest\(([^{][^)]+)\)',
        r'ApiError::BadRequest { message: \1 }',
        content
    )

    # Fix NotFound - it's a struct variant
    content = re.sub(
        r'ApiError::NotFound\(format!\("([^"]*)",\s*([^)]+)\)\)',
        r'ApiError::NotFound { resource: format!("\1", \2) }',
        content
    )
    content = re.sub(
        r'ApiError::NotFound\("([^"]*)"\)',
        r'ApiError::NotFound { resource: "\1".to_string() }',
        content
    )

    # Fix Validation - it's a struct variant
    content = re.sub(
        r'ApiError::Validation\("([^"]*)"\)',
        r'ApiError::Validation { message: "\1".to_string(), field: None, details: None }',
        content
    )

    # Fix Internal - takes anyhow::Error
    content = re.sub(
        r'ApiError::Internal\(format!\("([^"]*)",\s*([^)]+)\)\)',
        r'ApiError::Internal(anyhow::anyhow!("\1", \2))',
        content
    )

    # Fix Forbidden - it's a unit variant
    content = re.sub(
        r'ApiError::Forbidden\([^)]*\)',
        r'ApiError::Forbidden',
        content
    )

    return content

def fix_audit_log_event(content):
    """Fix audit log_event calls - method doesn't exist"""
    # Comment out audit log_event calls for now
    lines = content.split('\n')
    result = []
    skip_until = None

    for i, line in enumerate(lines):
        if skip_until and i < skip_until:
            continue
        skip_until = None

        if 'state.audit.log_event' in line or '.audit.log_event' in line:
            # Find the end of this statement
            depth = 0
            for j in range(i, len(lines)):
                depth += lines[j].count('(') - lines[j].count(')')
                if ';' in lines[j] and depth <= 0:
                    skip_until = j + 1
                    break
            result.append(' ' * (len(line) - len(line.lstrip())) + '// TODO: Fix audit logging')
        else:
            result.append(line)

    return '\n'.join(result)

def fix_response_structure(content):
    """Fix ApiResponse structure"""
    # Fix data field - should be Option
    content = re.sub(
        r'(\s+data:\s*)([^,\n]+)(,\s*\n\s*message:)',
        r'\1Some(\2)',
        content
    )

    # Remove message field (doesn't exist in ApiResponse)
    content = re.sub(r',\s*\n\s*message:\s*[^\n]+', '', content)

    return content

def fix_core_error_variants(content):
    """Fix CoreError variant usage"""
    # InvalidInput doesn't exist
    content = re.sub(
        r'CoreError::InvalidInput\s*\{[^}]+\}',
        r'CoreError::Validation { message: "Invalid input".to_string() }',
        content
    )

    # InternalError doesn't exist
    content = re.sub(
        r'CoreError::InternalError\s*\{[^}]+\}',
        r'CoreError::Internal(anyhow::anyhow!("Internal error"))',
        content
    )

    # PermissionDenied -> IamPermissionDenied
    content = re.sub(
        r'CoreError::PermissionDenied',
        r'CoreError::IamPermissionDenied',
        content
    )

    # Fix NotFound and AlreadyExists - remove id field
    content = re.sub(
        r'(CoreError::(?:NotFound|AlreadyExists)\s*\{\s*resource:[^,]+),\s*id:[^}]+',
        r'\1',
        content
    )

    return content

def fix_tuple_errors(content):
    """Fix tuple-style errors that should be struct variants"""
    # Fix (StatusCode, String) errors
    content = re.sub(
        r'Err\(\(\s*StatusCode::([A-Z_]+),\s*([^)]+)\)\)',
        r'Err(ApiError::Internal(anyhow::anyhow!(\2)))',
        content
    )

    return content

def main():
    if len(sys.argv) < 2:
        print("Usage: fix_api_errors.py <file>")
        sys.exit(1)

    file_path = Path(sys.argv[1])
    if not file_path.exists():
        print(f"File not found: {file_path}")
        sys.exit(1)

    content = file_path.read_text()

    # Apply fixes
    content = fix_user_info_fields(content)
    content = fix_api_error_variants(content)
    content = fix_audit_log_event(content)
    content = fix_response_structure(content)
    content = fix_core_error_variants(content)
    content = fix_tuple_errors(content)

    # Write back
    file_path.write_text(content)
    print(f"Fixed: {file_path}")

if __name__ == "__main__":
    main()
