#!/usr/bin/env python3
"""Comprehensive fix for all policy.rs syntax errors"""

import re
from pathlib import Path

def fix_file(content):
    """Fix all syntax errors in policy.rs"""

    # Fix 1: Remove orphaned })?; after map_err lines
    lines = content.split('\n')
    result = []
    i = 0
    while i < len(lines):
        line = lines[i]
        result.append(line)

        # If this line ends with ))?; and next lines are orphaned closing braces
        if line.strip().endswith('))?;'):
            # Check next lines
            j = i + 1
            while j < len(lines) and lines[j].strip() in ['', '}', '})?;']:
                if lines[j].strip() in ['}', '})?;']:
                    # Skip this orphaned brace
                    i = j
                j += 1

        i += 1

    content = '\n'.join(result)

    # Fix 2: Fix ApiResponse syntax errors
    # Pattern: data: Some(policy) Some("message")
    content = re.sub(
        r'data: Some\(([^)]+)\) Some\(',
        r'data: Some(\1),\n        error: None,\n        metadata: None,\n    })\n}\n\n// FIXME: Remove duplicate Some(',
        content
    )

    # Fix 3: Fix resource: "policy".to_string()})?;
    content = re.sub(
        r'resource: "policy"\.to_string\(\)\}\)\?;',
        r'resource: "policy".to_string()\n        })?;',
        content
    )

    # Fix 4: Fix validation error with dependent_policies
    content = re.sub(
        r'CoreError::Validation \{ message: "Invalid input"\.to_string\(\) \}\'.*?\n.*?dependent_policies\.join.*?\n.*?\),\n.*?\}\);',
        'CoreError::Validation { message: "Cannot delete policy".to_string() });',
        content,
        flags=re.DOTALL
    )

    # Fix 5: Fix PaginatedResponse structure
    content = re.sub(
        r'Ok\(Json\(PaginatedResponse \{\s*data: ([^,]+),\s*total: ([^,]+),\s*limit: ([^,]+),\s*offset: ([^,\}]+)\s*\}\)\)',
        r'Ok(Json(PaginatedResponse::new(\1, \2, \3, \4)))',
        content
    )

    return content

def main():
    file_path = Path("crates/api/src/handlers/policy.rs")
    if not file_path.exists():
        print(f"File not found: {file_path}")
        return

    content = file_path.read_text()
    content = fix_file(content)
    file_path.write_text(content)
    print(f"Fixed: {file_path}")

if __name__ == "__main__":
    main()
