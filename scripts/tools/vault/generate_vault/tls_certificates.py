"""
TLS Certificate generation module for Vault
Handles certificate creation, validation, and management
"""

import os
import socket
import ipaddress
from pathlib import Path
from cryptography import x509
from cryptography.x509.oid import NameOID
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import rsa
import datetime
from .config import (
    VAULT_DIR, CERT_ORGANIZATION, CERT_COUNTRY, CERT_STATE, 
    CERT_LOCALITY, CERT_COMMON_NAME, CERT_VALIDITY_DAYS,
    CERT_FILE_PERMISSIONS
)

def generate_tls_certificates():
    """Generate TLS certificates for Vault"""
    print("🔐 Generating TLS certificates...")
    
    certs_dir = VAULT_DIR / "certs"
    certs_dir.mkdir(exist_ok=True)
    
    # Generate private key
    private_key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
    )
    
    # Generate certificate
    subject = issuer = x509.Name([
        x509.NameAttribute(NameOID.COUNTRY_NAME, CERT_COUNTRY),
        x509.NameAttribute(NameOID.STATE_OR_PROVINCE_NAME, CERT_STATE),
        x509.NameAttribute(NameOID.LOCALITY_NAME, CERT_LOCALITY),
        x509.NameAttribute(NameOID.ORGANIZATION_NAME, CERT_ORGANIZATION),
        x509.NameAttribute(NameOID.COMMON_NAME, CERT_COMMON_NAME),
    ])
    
    cert = x509.CertificateBuilder().subject_name(
        subject
    ).issuer_name(
        issuer
    ).public_key(
        private_key.public_key()
    ).serial_number(
        x509.random_serial_number()
    ).not_valid_before(
        datetime.datetime.utcnow()
    ).not_valid_after(
        datetime.datetime.utcnow() + datetime.timedelta(days=CERT_VALIDITY_DAYS)
    ).add_extension(
        x509.SubjectAlternativeName([
            x509.DNSName(CERT_COMMON_NAME),
            x509.DNSName("vault.simpelv2.svc"),
            x509.DNSName("vault.simpelv2.svc.cluster.local"),
            x509.DNSName("localhost"),
            x509.IPAddress(ipaddress.IPv4Address("127.0.0.1")),
        ]),
        critical=False,
    ).sign(private_key, hashes.SHA256())
    
    # Save certificate and private key
    cert_path = certs_dir / "server.crt"
    key_path = certs_dir / "server.key"
    
    with open(cert_path, "wb") as f:
        f.write(cert.public_bytes(serialization.Encoding.PEM))
    
    with open(key_path, "wb") as f:
        f.write(private_key.private_bytes(
            encoding=serialization.Encoding.PEM,
            format=serialization.PrivateFormat.PKCS8,
            encryption_algorithm=serialization.NoEncryption()
        ))
    
    # Set proper permissions
    os.chmod(cert_path, CERT_FILE_PERMISSIONS)
    os.chmod(key_path, 0o600)  # Private key should be more restrictive
    
    print("✅ TLS certificates generated successfully")
    return cert_path, key_path

def validate_certificates(cert_path, key_path):
    """Validate generated certificates"""
    try:
        # Check if files exist
        if not cert_path.exists():
            print("❌ Certificate file not found")
            return False
        
        if not key_path.exists():
            print("❌ Private key file not found")
            return False
        
        # Check file permissions
        cert_stat = cert_path.stat()
        key_stat = key_path.stat()
        
        if oct(cert_stat.st_mode)[-3:] != str(CERT_FILE_PERMISSIONS)[-3:]:
            print("❌ Certificate file permissions incorrect")
            return False
        
        if oct(key_stat.st_mode)[-3:] != "600":
            print("❌ Private key file permissions incorrect")
            return False
        
        print("✅ Certificate validation passed")
        return True
        
    except Exception as e:
        print(f"❌ Certificate validation failed: {e}")
        return False 