import re

files_to_update = [
    'layanan/perlengkapan/Cargo.toml',
    'layanan/secreton/Cargo.toml',
    'layanan/integrasi/Cargo.toml',
    'Cargo.toml'
]

for file in files_to_update:
    try:
        with open(file, 'r') as f:
            content = f.read()

        content = re.sub(r'prometheus = "0.13(.\d+)?"', 'prometheus = "0.13"', content)
        content = re.sub(r'prometheus = \{ version = "0.13(.\d+)?".*?\}', 'prometheus = { version = "0.13", features = ["process"] }', content)

        with open(file, 'w') as f:
            f.write(content)
    except FileNotFoundError:
        pass
