import re
import glob

files = glob.glob("layanan/secreton/crates/backup/tests/*.rs")

for file in files:
    with open(file, "r") as f:
        content = f.read()

    # Fix the syntax error `});` that we accidentally caused by the regex
    content = content.replace("})\n    });", "});")
    content = content.replace("})\n        });", "});")
    content = content.replace("})\n            });", "});")
    content = content.replace("})\n    })?;", "});")
    content = content.replace("})\n        })?;", "});")
    content = content.replace("})\n            })?;", "});")

    # Actually wait let's just use regex to fix `path: ... })\n    });` or similar
    content = re.sub(r"\}\)\n\s*\}\)(;|\?)", "});", content)

    with open(file, "w") as f:
        f.write(content)
