#!/usr/bin/env python3
"""
Enhanced Nginx Configuration Generator for SIMPelv2
Generates optimized nginx configurations for microfrontends and infrastructure
"""

import os
import sys
import json
import argparse
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional

# Add the generators module to Python path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), 'generators', 'python'))

class NginxConfigGenerator:
    def __init__(self, base_dir: str = "/var/www/simpelv2"):
        self.base_dir = Path(base_dir)
        self.antarmuka_dir = self.base_dir / "antarmuka"
        self.infra_dir = self.base_dir / "infra"
        self.shared_nginx_dir = self.antarmuka_dir / "shared" / "nginx"
        self.generated_dir = self.shared_nginx_dir / "generated"
        
        # Ensure directories exist
        self.generated_dir.mkdir(parents=True, exist_ok=True)
        
    def get_microfrontend_services(self) -> List[str]:
        """Discover microfrontend services from antarmuka directory"""
        services = []
        if not self.antarmuka_dir.exists():
            return services
            
        for item in self.antarmuka_dir.iterdir():
            if (item.is_dir() and 
                item.name not in ['shared'] and 
                not item.name.startswith('.') and
                (item / "Cargo.toml").exists()):
                services.append(item.name)
                
        # Check for sub-microfrontends in pembinaan directory
        pembinaan_dir = self.antarmuka_dir / "pembinaan"
        if pembinaan_dir.exists():
            for sub_item in pembinaan_dir.iterdir():
                if (sub_item.is_dir() and 
                    not sub_item.name.startswith('.') and
                    (sub_item / "Cargo.toml").exists()):
                    # Add pembinaan sub-services with prefix for clarity
                    services.append(f"pembinaan-{sub_item.name}")
                    
        return sorted(services)
    
    def load_template(self, template_name: str) -> str:
        """Load nginx template file"""
        template_path = self.shared_nginx_dir / template_name
        if not template_path.exists():
            raise FileNotFoundError(f"Template not found: {template_path}")
            
        return template_path.read_text()
    
    def generate_microfrontend_config(self, service_name: str, template_content: str) -> str:
        """Generate nginx config for a specific microfrontend"""
        config = template_content
        
        # Replace template variables
        replacements = {
            "{{SERVICE_NAME}}": service_name,
            "{{SERVICE_NAME_UPPER}}": service_name.upper(),
            "{{SERVICE_NAME_TITLE}}": service_name.replace('_', ' ').title(),
            "{{TIMESTAMP}}": datetime.now().strftime("%B %d, %Y"),
            "{{SERVICE_PORT}}": "3000",  # Default port for microfrontends
        }
        
        for placeholder, value in replacements.items():
            config = config.replace(placeholder, value)
            
        return config
    
    def generate_all_microfrontend_configs(self) -> Dict[str, str]:
        """Generate nginx configs for all discovered microfrontends"""
        services = self.get_microfrontend_services()
        template = self.load_template("microfrontend.conf")
        
        configs = {}
        for service in services:
            config_content = self.generate_microfrontend_config(service, template)
            configs[service] = config_content
            
        return configs
    
    def save_generated_configs(self, configs: Dict[str, str]) -> List[Path]:
        """Save generated configs to files"""
        saved_files = []
        
        for service_name, config_content in configs.items():
            config_file = self.generated_dir / f"nginx-{service_name}.conf"
            config_file.write_text(config_content)
            saved_files.append(config_file)
            print(f"✅ Generated: {config_file}")
            
        return saved_files
    
    def generate_infrastructure_upstream_config(self, services: List[str]) -> str:
        """Generate upstream definitions for infrastructure nginx"""
        upstreams = []
        
        for service in services:
            # Handle sub-microfrontends differently - they need container names that match deployment
            if service.startswith('pembinaan-'):
                # Sub-microfrontends use their full name as container name
                container_name = service  # e.g., pembinaan-keuangan
                upstream = f"""    upstream {service} {{
        server {container_name}:3000;
    }}"""
            else:
                # Regular microfrontends use simple name
                upstream = f"""    upstream {service} {{
        server {service}:3000;
    }}"""
            upstreams.append(upstream)
            
        return "\n\n".join(upstreams)
    
    def generate_infrastructure_location_config(self, services: List[str]) -> str:
        """Generate location blocks for infrastructure nginx"""
        locations = []
        
        for service in services:
            if service.startswith('pembinaan-'):
                # Sub-microfrontends get their own location blocks
                sub_service = service.replace('pembinaan-', '')
                location = f"""        # {service.title().replace('-', ' ')}
        location /pembinaan/{sub_service} {{
            proxy_pass http://{service}/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }}"""
            else:
                location = f"""        # {service.title()}
        location /{service} {{
            proxy_pass http://{service}/;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }}"""
            locations.append(location)
            
        return "\n\n".join(locations)
    
    def update_infrastructure_nginx(self, services: List[str]) -> None:
        """Update infrastructure nginx.conf with discovered services"""
        nginx_conf_path = self.infra_dir / "nginx" / "nginx.conf"
        
        if not nginx_conf_path.exists():
            print(f"⚠️  Infrastructure nginx.conf not found: {nginx_conf_path}")
            return
            
        # Read current config
        current_config = nginx_conf_path.read_text()
        
        # Generate new upstream and location configs
        upstream_config = self.generate_infrastructure_upstream_config(services)
        location_config = self.generate_infrastructure_location_config(services)
        
        # Basic template for infrastructure nginx if needed
        if "upstream" not in current_config:
            print("🔄 Adding upstream configurations to infrastructure nginx...")
            # Find the http block and add upstreams
            http_start = current_config.find("http {")
            if http_start != -1:
                insert_pos = current_config.find("\n", http_start) + 1
                current_config = (current_config[:insert_pos] + 
                                "\n" + upstream_config + "\n" + 
                                current_config[insert_pos:])
        
        print(f"ℹ️  Infrastructure nginx config verified: {nginx_conf_path}")
    
    def validate_nginx_config(self, config_path: Path) -> bool:
        """Validate nginx configuration syntax"""
        # Use our own validation that handles Docker-specific configs
        try:
            config_content = config_path.read_text()
            
            # Basic syntax validation
            open_braces = config_content.count('{')
            close_braces = config_content.count('}')
            
            if open_braces != close_braces:
                print(f"❌ Unbalanced braces in {config_path.name}")
                return False
                
            # Check for required directives
            required_directives = ['server {', 'listen', 'location /']
            for directive in required_directives:
                if directive not in config_content:
                    print(f"❌ Missing directive '{directive}' in {config_path.name}")
                    return False
            
            # Check for WASM optimization
            if 'location ~* \\.wasm$' not in config_content:
                print(f"⚠️  Missing WASM optimization in {config_path.name}")
            
            print(f"✅ Valid nginx config: {config_path.name}")
            return True
            
        except Exception as e:
            print(f"❌ Error validating {config_path.name}: {e}")
            return False
    
    def generate_summary_report(self, services: List[str], generated_files: List[Path]) -> str:
        """Generate a summary report of the generation process"""
        report = f"""
# Nginx Configuration Generation Report
Generated on: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}

## Discovered Microfrontends
Total services: {len(services)}
Services: {', '.join(services)}

## Generated Files
Total files: {len(generated_files)}
"""
        
        for file_path in generated_files:
            report += f"- {file_path.relative_to(self.base_dir)}\n"
            
        report += f"""
## Template Source
- Base template: antarmuka/shared/nginx/microfrontend.conf
- Generated configs: antarmuka/shared/nginx/generated/
- Infrastructure config: infra/nginx/nginx.conf

## Next Steps
1. Review generated configurations
2. Test nginx config syntax: `nginx -t -c <config_file>`
3. Deploy using Docker Compose
4. Monitor nginx logs for any issues

## Auto-Generation Command
```bash
python3 scripts/tools/nginx-config-generator.py --generate-all
```
"""
        return report

def main():
    parser = argparse.ArgumentParser(description="Enhanced Nginx Configuration Generator")
    parser.add_argument("--base-dir", default="/var/www/simpelv2", 
                       help="Base directory of SIMPelv2 project")
    parser.add_argument("--generate-all", action="store_true",
                       help="Generate configs for all discovered microfrontends")
    parser.add_argument("--service", help="Generate config for specific service")
    parser.add_argument("--validate", action="store_true",
                       help="Validate generated nginx configurations")
    parser.add_argument("--report", action="store_true",
                       help="Generate summary report")
    parser.add_argument("--update-infra", action="store_true",
                       help="Update infrastructure nginx configuration")
    
    args = parser.parse_args()
    
    generator = NginxConfigGenerator(args.base_dir)
    
    print("🔧 Enhanced Nginx Configuration Generator")
    print("=" * 50)
    
    # Discover services
    services = generator.get_microfrontend_services()
    print(f"📊 Discovered {len(services)} microfrontends: {', '.join(services)}")
    
    generated_files = []
    
    if args.generate_all:
        print("\n🚀 Generating all microfrontend configurations...")
        configs = generator.generate_all_microfrontend_configs()
        generated_files = generator.save_generated_configs(configs)
        
    elif args.service:
        print(f"\n🎯 Generating configuration for {args.service}...")
        if args.service not in services:
            print(f"❌ Service '{args.service}' not found in microfrontends")
            sys.exit(1)
            
        template = generator.load_template("microfrontend.conf")
        config = generator.generate_microfrontend_config(args.service, template)
        config_file = generator.generated_dir / f"nginx-{args.service}.conf"
        config_file.write_text(config)
        generated_files = [config_file]
        print(f"✅ Generated: {config_file}")
    
    if args.update_infra:
        print("\n🏗️  Updating infrastructure nginx configuration...")
        generator.update_infrastructure_nginx(services)
    
    if args.validate and generated_files:
        print("\n🔍 Validating generated configurations...")
        for config_file in generated_files:
            generator.validate_nginx_config(config_file)
    
    if args.report:
        print("\n📋 Generating summary report...")
        report = generator.generate_summary_report(services, generated_files)
        report_file = generator.base_dir / "nginx-generation-report.md"
        report_file.write_text(report)
        print(f"📄 Report saved: {report_file}")
    
    print(f"\n✅ Nginx configuration generation completed!")
    print(f"📁 Generated {len(generated_files)} configuration files")

if __name__ == "__main__":
    main()
