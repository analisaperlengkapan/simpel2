#!/usr/bin/env python3
"""Fix StorageError usages to include source field"""

import re

# Read the file
with open('crates/storage/src/backends/postgres.rs', 'r') as f:
    content = f.read()

# Replace patterns - add source: None, after the opening brace and before message
replacements = [
    (r'(StorageError::ConnectionFailed\s*\{\s*)(message:)', r'\1source: None,\n            \2'),
    (r'(StorageError::QueryFailed\s*\{\s*)(message:)', r'\1source: None,\n            \2'),
    (r'(StorageError::TransactionFailed\s*\{\s*)(message:)', r'\1source: None,\n            \2'),
    (r'(StorageError::SerializationError\s*\{\s*)(message:)', r'\1source: None,\n            \2'),
]

for pattern, replacement in replacements:
    content = re.sub(pattern, replacement, content)

# Write back
with open('crates/storage/src/backends/postgres.rs', 'w') as f:
    f.write(content)

print("Fixed StorageError usages in postgres.rs")
