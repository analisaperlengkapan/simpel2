#!/usr/bin/env python3
"""Fix all syntax errors in policy.rs"""

import re
from pathlib import Path

def fix_validation_errors(content):
    """Fix CoreError::Validation syntax errors"""

    # Pattern 1: Fix lines like:
    # return Err(CoreError::Validation { message: "Invalid input".to_string() }].field", idx),
    # Should be:
    # return Err(CoreError::Validation { message: "Invalid input".to_string() });

    pattern1 = r'return Err\(CoreError::Validation \{ message: "Invalid input"\.to_string\(\) \}\]\..*?",\s*(?:idx|rule_idx)\),'
    content = re.sub(pattern1, 'return Err(CoreError::Validation { message: "Invalid input".to_string() });', content)

    # Pattern 2: Fix incomplete Validation blocks with extra fields
    # return Err(CoreError::Validation { message: "Invalid input".to_string() });
    #     reason: "something",
    # });

    lines = content.split('\n')
    result = []
    i = 0
    while i < len(lines):
        line = lines[i]

        # Check if this is a Validation error line
        if 'return Err(CoreError::Validation { message:' in line and line.strip().endswith(');'):
            result.append(line)
            # Skip next lines if they look like orphaned fields
            i += 1
            while i < len(lines):
                next_line = lines[i].strip()
                if next_line.startswith('reason:') or next_line.startswith('field:'):
                    i += 1  # Skip this line
                elif next_line == '});':
                    i += 1  # Skip closing brace
                    break
                else:
                    break
        else:
            result.append(line)
            i += 1

    return '\n'.join(result)

def fix_incomplete_validation_blocks(content):
    """Fix validation blocks that are missing opening braces"""

    # Fix pattern like:
    # return Err(CoreError::Validation { message: "Invalid input".to_string() });
    #     reason: format!(...),
    # });

    lines = content.split('\n')
    result = []
    skip_until = None

    for i, line in enumerate(lines):
        if skip_until and i < skip_until:
            continue
        skip_until = None

        # If we see a Validation error that ends with });
        if 'CoreError::Validation { message:' in line and '});' in line:
            result.append(line)
            # Check if next line has orphaned fields
            if i + 1 < len(lines):
                next_line = lines[i + 1].strip()
                if next_line.startswith('reason:') or next_line.startswith('field:'):
                    # Skip orphaned fields
                    j = i + 1
                    while j < len(lines):
                        if lines[j].strip() == '});':
                            skip_until = j + 1
                            break
                        j += 1
        else:
            result.append(line)

    return '\n'.join(result)

def main():
    file_path = Path("crates/api/src/handlers/policy.rs")
    if not file_path.exists():
        print(f"File not found: {file_path}")
        return

    content = file_path.read_text()

    # Apply fixes
    content = fix_validation_errors(content)
    content = fix_incomplete_validation_blocks(content)

    # Write back
    file_path.write_text(content)
    print(f"Fixed: {file_path}")

if __name__ == "__main__":
    main()
