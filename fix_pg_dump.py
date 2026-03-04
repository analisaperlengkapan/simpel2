import re
import glob

files = glob.glob("layanan/secreton/crates/backup/src/*.rs") + glob.glob("layanan/secreton/crates/backup/tests/*.rs")

for file in files:
    with open(file, "r") as f:
        content = f.read()

    # Check if there is a command running pg_dump, let's substitute it with `echo "mock pg_dump"` or handle the panic gracefully in tests
    # Wait, the best way is to only disable the pg_dump execution in tests or just wrap it.
    pass
