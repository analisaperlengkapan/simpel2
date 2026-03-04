with open("Cargo.toml", "r") as f:
    content = f.read()

content = content.replace('"layanan/authenc",\n', '')

with open("Cargo.toml", "w") as f:
    f.write(content)
