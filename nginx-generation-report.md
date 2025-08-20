
# Nginx Configuration Generation Report
Generated on: 2025-08-20 07:07:15

## Discovered Microfrontends
Total services: 12
Services: badiklat, datun, intel, pembinaan-keuangan, pembinaan-perencanaan, pembinaan-perlengkapan, pemulihan_aset, pengawasan, pidmil, pidsus, pidum, portal

## Generated Files
Total files: 12
- antarmuka/shared/nginx/generated/nginx-badiklat.conf
- antarmuka/shared/nginx/generated/nginx-datun.conf
- antarmuka/shared/nginx/generated/nginx-intel.conf
- antarmuka/shared/nginx/generated/nginx-pembinaan-keuangan.conf
- antarmuka/shared/nginx/generated/nginx-pembinaan-perencanaan.conf
- antarmuka/shared/nginx/generated/nginx-pembinaan-perlengkapan.conf
- antarmuka/shared/nginx/generated/nginx-pemulihan_aset.conf
- antarmuka/shared/nginx/generated/nginx-pengawasan.conf
- antarmuka/shared/nginx/generated/nginx-pidmil.conf
- antarmuka/shared/nginx/generated/nginx-pidsus.conf
- antarmuka/shared/nginx/generated/nginx-pidum.conf
- antarmuka/shared/nginx/generated/nginx-portal.conf

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
