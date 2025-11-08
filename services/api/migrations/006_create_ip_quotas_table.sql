-- Create ip_quotas table
-- Tracks IP-based quota for anonymous uploads (rate limiting per IP)
-- This is separate from upload_quotas which tracks per user/session
CREATE TABLE IF NOT EXISTS ip_quotas (
    id SERIAL PRIMARY KEY,
    ip_address VARCHAR(45) NOT NULL, -- IPv6 support
    bytes_used BIGINT NOT NULL DEFAULT 0 CHECK (bytes_used >= 0),
    period_start DATE NOT NULL DEFAULT CURRENT_DATE,
    -- Ensure one quota record per IP per period
    UNIQUE(ip_address, period_start)
);

-- Index for IP quota lookups by IP and period
CREATE INDEX idx_ip_quotas_ip_period ON ip_quotas(ip_address, period_start);
