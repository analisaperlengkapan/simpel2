-- Pidum Service - Timeline and Comments
CREATE TABLE IF NOT EXISTS pidum.timeline_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    perkara_id UUID NOT NULL REFERENCES pidum.perkara(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    description TEXT,
    created_by VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS pidum.comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    perkara_id UUID NOT NULL REFERENCES pidum.perkara(id) ON DELETE CASCADE,
    comment_text TEXT NOT NULL,
    author VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_pidum_timeline_perkara ON pidum.timeline_events(perkara_id);
CREATE INDEX IF NOT EXISTS idx_pidum_comments_perkara ON pidum.comments(perkara_id);
CREATE INDEX IF NOT EXISTS idx_pidum_timeline_created ON pidum.timeline_events(created_at);
