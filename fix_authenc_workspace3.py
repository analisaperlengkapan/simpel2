with open("layanan/authenc/Cargo.toml", "r") as f:
    content = f.read()

members = """
members = [
    "crates/api",
    "crates/core",
    "crates/crypto",
    "crates/federation",
    "crates/grpc",
    "crates/iam-api",
    "crates/mfa",
    "crates/storage",
    "crates/types",
    "crates/webauthn"
]
"""

if "members = [" not in content:
    content = content.replace("[workspace]", "[workspace]\n" + members)

with open("layanan/authenc/Cargo.toml", "w") as f:
    f.write(content)
