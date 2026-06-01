-- SMS OTP tokens for phone-based authentication
CREATE TABLE IF NOT EXISTS sms_otp_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    phone VARCHAR(20) NOT NULL,
    code_hash VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    ip_address VARCHAR(45)
);

CREATE INDEX IF NOT EXISTS idx_sms_otp_tokens_hash ON sms_otp_tokens(code_hash);
CREATE INDEX IF NOT EXISTS idx_sms_otp_tokens_expires ON sms_otp_tokens(expires_at) WHERE consumed_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_sms_otp_tokens_phone ON sms_otp_tokens(phone);

-- Add phone column to users table (nullable, unique when set)
ALTER TABLE users ADD COLUMN IF NOT EXISTS phone VARCHAR(20);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_phone_unique ON users(phone) WHERE phone IS NOT NULL;
