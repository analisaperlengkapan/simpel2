# Kubernetes YAML Generator

Modular Kubernetes YAML generator that converts Docker Compose files to Kubernetes manifests.

## Structure

```
scripts/
├── generate_k8s              # Main executable script
└── k8s_generator/           # Package directory
    ├── __init__.py          # Package initialization
    ├── config.py            # Configuration and constants
    ├── utils.py             # Utility functions
    ├── generators.py        # Kubernetes manifest generators
    ├── main.py              # Main workflow orchestration
    └── README.md            # This file
```

## Modules

### `config.py`
Contains all configuration constants, paths, and environment-specific settings:
- Path definitions
- Environment variables
- Service-specific configurations
- Monitoring and health check settings

### `utils.py`
Utility functions for common operations:
- YAML handling with custom formatting
- Port detection and validation
- Service name sanitization
- Volume parsing
- Health check parsing
- Error handling utilities

### `generators.py`
All Kubernetes manifest generation functions:
- `generate_namespace()` - Namespace manifest
- `generate_secret_from_env()` - Secrets from .env files
- `generate_ingress_tls_secret()` - TLS certificates
- `generate_db_pvc()` - Database persistent volume claim
- `generate_deployment()` - Deployment manifests
- `generate_service()` - Service manifests
- `generate_ingress()` - Ingress with routing rules
- `generate_monitoring_resources()` - Prometheus & Grafana configs
- `seal_env_to_sealed_secret()` - Sealed secrets generation

### `main.py`
Main workflow orchestration:
- Directory setup
- Dependency checking
- Service loading
- Manifest generation coordination
- Error handling and validation

## Usage

### Command Line Interface

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

### Programmatic Usage

```python
from scripts.k8s_generator import main

# Run the generator
success = main()
if not success:
    print("Generation failed")
```

## Features

### ✅ Modular Design
- Separated concerns into logical modules
- Easy to maintain and extend
- Clear separation of configuration, utilities, and generators

### ✅ Error Handling
- Comprehensive error handling throughout
- Graceful degradation on individual service failures
- Detailed error reporting and logging

### ✅ Configuration Management
- Environment-specific configurations
- Centralized constants and settings
- Easy to modify for different environments

### ✅ Robust Parsing
- Advanced port detection from various formats
- Flexible volume mount parsing
- Health check URL parsing
- Service name validation and sanitization

### ✅ Monitoring Integration
- Dynamic Prometheus target generation
- Configurable scrape intervals and timeouts
- Grafana datasource configuration

### ✅ Security Features
- Sealed secrets generation
- TLS certificate handling
- Input validation and sanitization

## Migration from Monolithic Script

The original `generate_k8s.py` has been refactored into this modular structure:

1. **Configuration**: All constants moved to `config.py`
2. **Utilities**: Helper functions moved to `utils.py`
3. **Generators**: Manifest generation functions moved to `generators.py`
4. **Workflow**: Main orchestration moved to `main.py`
5. **Entry Point**: New CLI script at `scripts/generate_k8s`

### Benefits of Modular Structure

- **Maintainability**: Each module has a single responsibility
- **Testability**: Individual modules can be tested in isolation
- **Reusability**: Functions can be imported and used independently
- **Readability**: Code is organized logically and easier to understand
- **Extensibility**: New features can be added without affecting existing code

## Dependencies

- Python 3.6+
- `pyyaml` - YAML parsing and generation
- `python-dotenv` - Environment variable loading
- `kubeseal` - Sealed secrets generation (external binary)

## Error Handling

The generator uses a global error collection system:
- Errors are collected during generation
- Generation continues even if individual services fail
- All errors are reported at the end
- Script exits with error code 1 if any errors occur

## Logging

Comprehensive logging with different levels:
- `INFO`: General progress information
- `WARNING`: Non-critical issues
- `ERROR`: Critical errors that may affect generation

## Future Enhancements

- Unit tests for individual modules
- Configuration validation
- Support for more volume types
- Enhanced health check configurations
- Custom resource definitions (CRDs)
- Helm chart generation 