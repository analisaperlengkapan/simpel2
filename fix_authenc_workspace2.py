with open("Cargo.toml", "r") as f:
    root_content = f.read()

deps = root_content.split("[workspace.dependencies]")[1].split("\n[")[0]
pkg = root_content.split("[workspace.package]")[1].split("\n[")[0]

with open("layanan/authenc/Cargo.toml", "r") as f:
    content = f.read()

# Replace empty workspace with one having package and dependencies
content = content.replace("[workspace]", "[workspace]\n\n[workspace.package]\n" + pkg + "\n\n[workspace.dependencies]\n" + deps)

# Replace all local paths with correct upward paths
content = content.replace('path = "layanan/', 'path = "../../layanan/')
content = content.replace('path = "lib/', 'path = "../../lib/')

with open("layanan/authenc/Cargo.toml", "w") as f:
    f.write(content)
