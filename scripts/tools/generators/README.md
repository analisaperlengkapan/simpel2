# 📋 GitLab CI Configuration Guide

This directory contains the configuration template for GitLab CI/CD pipeline generation.

## 📁 Files

- **`ci_config.example.json`** - Template configuration file with all available options
- **`python/gitlab_ci_generator.py`** - Main GitLab CI generator script

## 🚀 Usage

### 1. Create Your Configuration
```bash
# Copy example to create your config
cp scripts/tools/generators/ci_config.example.json ci_config.json

# Edit according to your needs
nano ci_config.json
```

### 2. Generate GitLab CI with Custom Config
```bash
# Generate with your custom configuration
python3 scripts/tools/generators/main-generator.py gitlab-ci --config ci_config.json

# Or using the unified generator
./scripts/dev-tools.sh generate-cicd --config ci_config.json
```

### 3. Validate Configuration
```bash
# Test configuration syntax
python -m json.tool ci_config.json

# Dry run to test generation
python3 scripts/tools/generators/main-generator.py gitlab-ci --config ci_config.json --dry-run
```

## ⚙️ Configuration Options

The `ci_config.example.json` file contains all available configuration options:

- **`ci_file`** - Target CI file name (default: `.gitlab-ci.yml`)
- **`base_image`** - Docker base image for CI jobs
- **`security_includes`** - Security scanning templates to include
- **`stages`** - Pipeline stages definition
- **`custom_variables`** - Custom GitLab CI variables
- **`deployment_config`** - Multi-environment deployment settings
- **`vault_config`** - HashiCorp Vault integration settings
- **`testing_config`** - Testing and coverage settings
- **`monitoring_config`** - Monitoring integration settings

## 🎯 Best Practices

1. **Keep configurations in project root** for easy access
2. **Use descriptive names** like `ci_config.production.json`, `ci_config.development.json`
3. **Validate JSON syntax** before using with CI generator
4. **Test with `--dry-run`** before applying configuration
5. **Version control your configurations** but exclude sensitive values

## 🔗 Related Commands

```bash
# Development tools integration
./scripts/dev-tools.sh generate-cicd --config ci_config.json

# Test suite integration  
./scripts/test-suite.sh validate-yaml ci_config.json
```

---
**Note:** The configuration file should be placed in the project root or specify the full path when using the `--config` option.
