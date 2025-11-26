//! Key generation utility for Authenc signing keys
//!
//! This tool generates cryptographic signing keys for production deployment.
//! Keys must be persistent across service restarts to prevent token invalidation.

use base64ct::Base64;
use base64ct::Encoding;
use clap::{Parser, ValueEnum};
use ed25519_dalek::SigningKey as Ed25519SigningKey;
use p256::ecdsa::SigningKey as P256SigningKey;
use p384::ecdsa::SigningKey as P384SigningKey;
use p521::ecdsa::SigningKey as P521SigningKey;
use rand::rngs::OsRng;

#[derive(Parser)]
#[command(name = "generate-signing-keys")]
#[command(about = "Generate signing keys for Authenc IAM service", long_about = None)]
struct Cli {
    /// Algorithm to use for key generation
    #[arg(short, long, value_enum, default_value_t = Algorithm::Ed25519)]
    algorithm: Algorithm,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::EnvVar)]
    format: OutputFormat,

    /// Output file path (optional)
    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Algorithm {
    /// Ed25519 (Recommended - Fast & Secure)
    Ed25519,
    /// ECDSA P-256
    P256,
    /// ECDSA P-384 (High Security)
    P384,
    /// ECDSA P-521 (Maximum Security)
    P521,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    /// Environment variable format
    EnvVar,
    /// Docker secret format
    DockerSecret,
    /// Kubernetes secret YAML
    K8sSecret,
    /// Raw file output
    File,
}

fn main() {
    let cli = Cli::parse();

    match cli.algorithm {
        Algorithm::Ed25519 => generate_ed25519(&cli),
        Algorithm::P256 => generate_p256(&cli),
        Algorithm::P384 => generate_p384(&cli),
        Algorithm::P521 => generate_p521(&cli),
    }
}

fn generate_ed25519(cli: &Cli) {
    println!("🔐 Generating Ed25519 signing keypair...\n");

    let signing_key = Ed25519SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let private_key_bytes = signing_key.to_bytes();
    let public_key_bytes = verifying_key.to_bytes();

    let private_key_base64 = Base64::encode_string(&private_key_bytes);
    let public_key_base64 = Base64::encode_string(&public_key_bytes);

    match cli.format {
        OutputFormat::EnvVar => {
            println!("📋 Environment Variable Configuration:");
            println!("───────────────────────────────────────");
            println!(
                "export ED25519_PRIVATE_KEY_BASE64=\"{}\"",
                private_key_base64
            );
            println!();
            println!("Add this to your .env file or deployment configuration.");
        }
        OutputFormat::DockerSecret => {
            println!("🐳 Docker Secret Configuration:");
            println!("───────────────────────────────────────");
            println!("# Create secret:");
            println!(
                "echo '{}' | docker secret create authenc_ed25519_key -",
                private_key_base64
            );
            println!();
            println!("# docker-compose.yml:");
            println!("services:");
            println!("  authenc:");
            println!("    secrets:");
            println!("      - authenc_ed25519_key");
            println!("    environment:");
            println!("      - ED25519_PRIVATE_KEY_BASE64=/run/secrets/authenc_ed25519_key");
        }
        OutputFormat::K8sSecret => {
            println!("☸️  Kubernetes Secret YAML:");
            println!("───────────────────────────────────────");
            println!("apiVersion: v1");
            println!("kind: Secret");
            println!("metadata:");
            println!("  name: authenc-signing-keys");
            println!("  namespace: default");
            println!("type: Opaque");
            println!("data:");
            println!("  ed25519-private-key: {}", private_key_base64);
            println!();
            println!("# Deployment usage:");
            println!("env:");
            println!("- name: ED25519_PRIVATE_KEY_BASE64");
            println!("  valueFrom:");
            println!("    secretKeyRef:");
            println!("      name: authenc-signing-keys");
            println!("      key: ed25519-private-key");
        }
        OutputFormat::File => {
            let filename = cli.output.as_deref().unwrap_or("ed25519-private.key");
            std::fs::write(filename, private_key_bytes).expect("Failed to write key file");
            println!("✅ Private key written to: {}", filename);
            println!();
            println!("Use with: export ED25519_PRIVATE_KEY_PATH={}", filename);
        }
    }

    println!();
    println!("🔑 Key Information:");
    println!("───────────────────────────────────────");
    println!("Algorithm:    Ed25519");
    println!("Key Size:     256 bits (32 bytes)");
    println!("Public Key:   {}", public_key_base64);
    println!();
    println!("⚠️  SECURITY WARNING:");
    println!("• Keep the private key SECRET and SECURE");
    println!("• Never commit keys to version control");
    println!("• Backup keys in multiple secure locations");
    println!("• Rotate keys every 90-180 days");
}

fn generate_p256(cli: &Cli) {
    println!("🔐 Generating ECDSA P-256 signing keypair...\n");

    let signing_key = P256SigningKey::random(&mut OsRng);
    let private_key_bytes = signing_key.to_bytes();
    let private_key_base64 = Base64::encode_string(&private_key_bytes);

    match cli.format {
        OutputFormat::EnvVar => {
            println!("📋 Environment Variable Configuration:");
            println!("───────────────────────────────────────");
            println!(
                "export ECDSA_P256_PRIVATE_KEY_BASE64=\"{}\"",
                private_key_base64
            );
        }
        OutputFormat::DockerSecret => {
            println!("🐳 Docker Secret Configuration:");
            println!("───────────────────────────────────────");
            println!(
                "echo '{}' | docker secret create authenc_p256_key -",
                private_key_base64
            );
        }
        OutputFormat::K8sSecret => {
            println!("☸️  Kubernetes Secret YAML:");
            println!("───────────────────────────────────────");
            println!("apiVersion: v1");
            println!("kind: Secret");
            println!("metadata:");
            println!("  name: authenc-signing-keys");
            println!("type: Opaque");
            println!("data:");
            println!("  p256-private-key: {}", private_key_base64);
        }
        OutputFormat::File => {
            let filename = cli.output.as_deref().unwrap_or("p256-private.key");
            std::fs::write(filename, private_key_bytes).expect("Failed to write key file");
            println!("✅ Private key written to: {}", filename);
        }
    }

    println!();
    println!("🔑 Key Information:");
    println!("───────────────────────────────────────");
    println!("Algorithm:    ECDSA P-256 (prime256v1)");
    println!("Key Size:     256 bits");
}

fn generate_p384(cli: &Cli) {
    println!("🔐 Generating ECDSA P-384 signing keypair...\n");

    let signing_key = P384SigningKey::random(&mut OsRng);
    let private_key_bytes = signing_key.to_bytes();
    let private_key_base64 = Base64::encode_string(&private_key_bytes);

    match cli.format {
        OutputFormat::EnvVar => {
            println!("📋 Environment Variable Configuration:");
            println!("───────────────────────────────────────");
            println!(
                "export ECDSA_P384_PRIVATE_KEY_BASE64=\"{}\"",
                private_key_base64
            );
        }
        OutputFormat::DockerSecret => {
            println!("🐳 Docker Secret Configuration:");
            println!("───────────────────────────────────────");
            println!(
                "echo '{}' | docker secret create authenc_p384_key -",
                private_key_base64
            );
        }
        OutputFormat::K8sSecret => {
            println!("☸️  Kubernetes Secret YAML:");
            println!("───────────────────────────────────────");
            println!("apiVersion: v1");
            println!("kind: Secret");
            println!("metadata:");
            println!("  name: authenc-signing-keys");
            println!("type: Opaque");
            println!("data:");
            println!("  p384-private-key: {}", private_key_base64);
        }
        OutputFormat::File => {
            let filename = cli.output.as_deref().unwrap_or("p384-private.key");
            std::fs::write(filename, private_key_bytes).expect("Failed to write key file");
            println!("✅ Private key written to: {}", filename);
        }
    }

    println!();
    println!("🔑 Key Information:");
    println!("───────────────────────────────────────");
    println!("Algorithm:    ECDSA P-384");
    println!("Key Size:     384 bits");
}

fn generate_p521(cli: &Cli) {
    println!("🔐 Generating ECDSA P-521 signing keypair...\n");

    let signing_key = P521SigningKey::random(&mut OsRng);
    let private_key_bytes = signing_key.to_bytes();
    let private_key_base64 = Base64::encode_string(&private_key_bytes);

    match cli.format {
        OutputFormat::EnvVar => {
            println!("📋 Environment Variable Configuration:");
            println!("───────────────────────────────────────");
            println!(
                "export ECDSA_P521_PRIVATE_KEY_BASE64=\"{}\"",
                private_key_base64
            );
        }
        OutputFormat::DockerSecret => {
            println!("🐳 Docker Secret Configuration:");
            println!("───────────────────────────────────────");
            println!(
                "echo '{}' | docker secret create authenc_p521_key -",
                private_key_base64
            );
        }
        OutputFormat::K8sSecret => {
            println!("☸️  Kubernetes Secret YAML:");
            println!("───────────────────────────────────────");
            println!("apiVersion: v1");
            println!("kind: Secret");
            println!("metadata:");
            println!("  name: authenc-signing-keys");
            println!("type: Opaque");
            println!("data:");
            println!("  p521-private-key: {}", private_key_base64);
        }
        OutputFormat::File => {
            let filename = cli.output.as_deref().unwrap_or("p521-private.key");
            std::fs::write(filename, private_key_bytes).expect("Failed to write key file");
            println!("✅ Private key written to: {}", filename);
        }
    }

    println!();
    println!("🔑 Key Information:");
    println!("───────────────────────────────────────");
    println!("Algorithm:    ECDSA P-521");
    println!("Key Size:     521 bits (Maximum Security)");
}
