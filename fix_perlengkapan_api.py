with open("layanan/perlengkapan/crates/api/src/kebutuhan_bmn/handlers.rs", "r") as f:
    content = f.read()

content = content.replace("SortField::from_str", "SortField::from_string")
content = content.replace("SortDirection::from_str", "SortDirection::from_string")

with open("layanan/perlengkapan/crates/api/src/kebutuhan_bmn/handlers.rs", "w") as f:
    f.write(content)
