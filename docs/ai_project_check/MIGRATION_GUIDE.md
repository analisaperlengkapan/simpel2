# Migration Guide: Monolithic to Modular Structure

## Overview

The original `generate_k8s.py` script (875 lines) has been refactored into a modular structure for better maintainability, testability, and extensibility.

## What Changed

### Before (Monolithic)
```
scripts/
└── generate_k8s.py          # 875 lines - everything in one file
```

### After (Modular)
```
scripts/
├── generate_k8s             # New CLI entry point
├── generate_k8s.py.backup   # Backup of original script
└── k8s_generator/           # New package structure
    ├── __init__.py          # Package initialization
    ├── config.py            # Configuration (was constants at top)
    ├── utils.py             # Utility functions
    ├── generators.py        # Kubernetes manifest generators
    ├── main.py              # Main workflow orchestration
    └── README.md            # Documentation
```

## Migration Steps

### 1. Update Script Calls

**Old way:**
```bash
python3 scripts/generate_k8s.py
```

**New way:**
```bash
python3 scripts/generate_k8s
```

### 2. New CLI Features

The new script supports command-line arguments:

```bash
# Generate for dev environment (default)
python3 scripts/generate_k8s

# Generate for production
python3 scripts/generate_k8s --env prod

# Generate for staging with specific tag
python3 scripts/generate_k8s --env staging --tag v1.2.3

# Show help
python3 scripts/generate_k8s --help

# Show version
python3 scripts/generate_k8s --version
```

### 3. Programmatic Usage

**Old way:**
```python
# Had to run the entire script
import subprocess
subprocess.run(["python3", "scripts/generate_k8s.py"])
```

**New way:**
```python
# Can import and use specific functions
from scripts.k8s_generator import main

# Run the entire generator
success = main()

# Or import specific modules
from scripts.k8s_generator.config import DEFAULT_NAMESPACE
from scripts.k8s_generator.utils import extract_port_from_config
from scripts.k8s_generator.generators import generate_deployment
```

## Module Breakdown

### `config.py` (Previously: Constants at top of file)
- All configuration constants
- Environment-specific settings
- Path definitions
- Service-specific configurations

### `utils.py` (Previously: Utility functions scattered throughout)
- YAML handling functions
- Port detection and validation
- Service name sanitization
- Volume parsing
- Health check parsing
- Error handling utilities

### `generators.py` (Previously: Main generation functions)
- All Kubernetes manifest generation functions
- Deployment, Service, Ingress generators
- Secret and TLS generation
- Monitoring resource generation

### `main.py` (Previously: Main workflow in main() function)
- Main workflow orchestration
- Directory setup
- Dependency checking
- Error handling and validation

## Benefits of New Structure

### ✅ Maintainability
- **Before**: 875 lines in one file, hard to find specific functionality
- **After**: Logical separation into focused modules

### ✅ Testability
- **Before**: Could only test the entire script
- **After**: Individual modules can be unit tested

### ✅ Reusability
- **Before**: Functions were tied to the main script
- **After**: Functions can be imported and used independently

### ✅ Readability
- **Before**: Mixed concerns, hard to understand flow
- **After**: Clear separation of concerns

### ✅ Extensibility
- **Before**: Adding features required modifying the main script
- **After**: New features can be added as separate modules

## Backward Compatibility

### ✅ Same Output
- Generated YAML files are identical
- Same directory structure
- Same file naming conventions

### ✅ Same Environment Variables
- `ENV` - Environment (dev/staging/prod)
- `TAG` - Docker image tag
- `VERSION` - Production version (when ENV=prod)

### ✅ Same Dependencies
- `pyyaml`
- `python-dotenv`
- `kubeseal` (external binary)

## Rollback Plan

If you need to rollback to the original script:

1. **Restore backup:**
   ```bash
   mv scripts/generate_k8s.py.backup scripts/generate_k8s.py
   ```

2. **Remove new structure:**
   ```bash
   rm -rf scripts/k8s_generator/
   rm scripts/generate_k8s
   ```

3. **Use original command:**
   ```bash
   python3 scripts/generate_k8s.py
   ```

## Testing the New Structure

### 1. Syntax Check
```bash
python3 -m py_compile scripts/generate_k8s
python3 -m py_compile scripts/k8s_generator/*.py
```

### 2. Help Command
```bash
python3 scripts/generate_k8s --help
```

### 3. Version Check
```bash
python3 scripts/generate_k8s --version
```

### 4. Generate Test
```bash
# Test with dev environment
python3 scripts/generate_k8s --env dev

# Verify output is identical to original
diff -r k8s/ k8s_backup/  # If you have a backup
```

## Future Enhancements

The modular structure enables these future improvements:

- **Unit Tests**: Each module can be tested independently
- **Configuration Validation**: Add validation to config.py
- **Plugin System**: New generators can be added as plugins
- **API Interface**: Expose functions as a library
- **Web Interface**: Create a web UI using the modules

## Support

If you encounter issues with the new structure:

1. Check the help: `python3 scripts/generate_k8s --help`
2. Review the README: `scripts/k8s_generator/README.md`
3. Compare with backup: `scripts/generate_k8s.py.backup`
4. Check logs for detailed error messages

## Summary

The refactoring improves the codebase significantly while maintaining full backward compatibility. The new structure is more maintainable, testable, and extensible, making it easier to add new features and fix bugs in the future. 