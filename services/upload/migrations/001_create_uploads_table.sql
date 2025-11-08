-- Create uploads table
-- Tracks file upload sessions (either authenticated users or anonymous sessions)
-- Foreign key: user_id references users(id) from services/api
CREATE TABLE IF NOT EXISTS uploads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Ensure exactly one of user_id or session_id is set (authenticated XOR anonymous)
    CONSTRAINT check_user_or_session CHECK (
        (user_id IS NOT NULL AND session_id IS NULL) OR
        (user_id IS NULL AND session_id IS NOT NULL)
    )
);

-- Index for authenticated user lookups
CREATE INDEX idx_uploads_user_id ON uploads(user_id);

-- Index for anonymous session lookups
CREATE INDEX idx_uploads_session_id ON uploads(session_id);

-- Index for status-based queries (e.g., cleanup jobs)
CREATE INDEX idx_uploads_status ON uploads(status);
