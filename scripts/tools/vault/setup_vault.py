#!/usr/bin/env python3
"""
Vault Setup Script - Main Entry Point
Refactored to use modular architecture for better maintainability
"""

import sys
from pathlib import Path

# Add the generate_vault package to Python path
sys.path.insert(0, str(Path(__file__).parent / "generate_vault"))

def main():
    """Main entry point for Vault setup"""
    try:
        # Import and run the modular setup
        from generate_vault import setup_vault
        setup_vault()
    except ImportError as e:
        print(f"❌ Failed to import modules: {e}")
        print("💡 Make sure all modules are properly installed")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()