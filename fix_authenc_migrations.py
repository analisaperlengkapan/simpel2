import re

with open("layanan/authenc/migrations/012_user_sessions.sql", "r") as f:
    content = f.read()

# Add missing token_hash and refresh_token_hash column additions to the user_sessions alter table block
# Let's insert it before the DO $$ END block finishes
addition = """
    -- Add token_hash column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'token_hash') THEN
        ALTER TABLE user_sessions ADD COLUMN token_hash VARCHAR(64);
    END IF;

    -- Add refresh_token_hash column
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'user_sessions' AND column_name = 'refresh_token_hash') THEN
        ALTER TABLE user_sessions ADD COLUMN refresh_token_hash VARCHAR(64);
    END IF;
"""

if "token_hash VARCHAR(64)" not in content and "ADD COLUMN token_hash" not in content:
    content = content.replace("END $$;", addition + "\nEND $$;")

with open("layanan/authenc/migrations/012_user_sessions.sql", "w") as f:
    f.write(content)
