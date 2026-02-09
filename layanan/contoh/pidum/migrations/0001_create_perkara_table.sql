-- Pidum Service - Initial Schema Migration
CREATE SCHEMA IF NOT EXISTS pidum;

-- Create perkara (case) table
CREATE TABLE IF NOT EXISTS pidum.perkara (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    nomor_perkara VARCHAR(100) UNIQUE NOT NULL,
    tersangka VARCHAR(255) NOT NULL,
    pasal TEXT,
    jaksa_penuntut VARCHAR(255),
    status VARCHAR(50) DEFAULT 'active',
    tanggal_masuk DATE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_pidum_perkara_status ON pidum.perkara(status);
CREATE INDEX IF NOT EXISTS idx_pidum_perkara_created ON pidum.perkara(created_at);
CREATE INDEX IF NOT EXISTS idx_pidum_perkara_nomor ON pidum.perkara(nomor_perkara);
