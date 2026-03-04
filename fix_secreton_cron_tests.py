import re
import glob

files = glob.glob("layanan/secreton/crates/backup/src/*.rs") + glob.glob("layanan/secreton/crates/backup/tests/*.rs")

# The cron library `cron` in Rust expects 7 fields (Seconds, Minutes, Hours, Days of month, Months, Days of week, Years)
# "0 2 * * *" is 5 fields, let's change it to "0 0 2 * * * *" or similar
for file in files:
    with open(file, "r") as f:
        content = f.read()

    content = content.replace('"0 2 * * *"', '"0 0 2 * * * *"')

    with open(file, "w") as f:
        f.write(content)
