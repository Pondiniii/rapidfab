-- Create upload_quotas table
-- Tracks bandwidth usage per user/session per day for rate limiting
-- Foreign key: user_id references users(id) from services/api
CREATE TABLE IF NOT EXISTS upload_quotas (
    id SERIAL PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID,
    bytes_used BIGINT NOT NULL DEFAULT 0 CHECK (bytes_used >= 0),
    period_start DATE NOT NULL DEFAULT CURRENT_DATE,
    -- Ensure one quota record per user per period
    UNIQUE(user_id, period_start),
    -- Ensure one quota record per session per period
    UNIQUE(session_id, period_start),
    -- Ensure exactly one of user_id or session_id is set (authenticated XOR anonymous)
    CONSTRAINT check_quota_user_or_session CHECK (
        (user_id IS NOT NULL AND session_id IS NULL) OR
        (user_id IS NULL AND session_id IS NOT NULL)
    )
);

-- Index for authenticated user quota lookups
CREATE INDEX idx_quotas_user_period ON upload_quotas(user_id, period_start);

-- Index for anonymous session quota lookups
CREATE INDEX idx_quotas_session_period ON upload_quotas(session_id, period_start);
