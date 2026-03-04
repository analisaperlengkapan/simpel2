with open("layanan/secreton/migrations/20250101000004_create_namespaces.sql", "r") as f:
    content = f.read()

# Fix the recursive CTE type mismatch for `ancestors` array
# Base case: ARRAY[id]::VARCHAR[] AS ancestors
content = content.replace("ARRAY[id] AS ancestors", "ARRAY[id]::VARCHAR[] AS ancestors")
# Recursive case: h.ancestors || n.id::VARCHAR
content = content.replace("h.ancestors || n.id", "h.ancestors || n.id::VARCHAR")

with open("layanan/secreton/migrations/20250101000004_create_namespaces.sql", "w") as f:
    f.write(content)

with open("layanan/secreton/migrations/20250101000006_create_leases.sql", "r") as f:
    content = f.read()

# Fix the recursive CTE type mismatch for `ancestors` array and `path`
content = content.replace("ARRAY[id] AS ancestors", "ARRAY[id]::VARCHAR[] AS ancestors")
content = content.replace("h.ancestors || l.id", "h.ancestors || l.id::VARCHAR")
content = content.replace("id::TEXT AS path", "id::VARCHAR AS path")
content = content.replace("h.path || ' -> ' || l.id", "(h.path || ' -> ' || l.id)::VARCHAR")

with open("layanan/secreton/migrations/20250101000006_create_leases.sql", "w") as f:
    f.write(content)
