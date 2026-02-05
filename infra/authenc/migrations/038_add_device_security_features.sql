-- Add security_features column to devices table
ALTER TABLE devices
ADD COLUMN IF NOT EXISTS security_features JSONB;

-- Comment on column
COMMENT ON COLUMN devices.security_features IS 'Security features available on the device (biometrics, encryption, etc.)';
