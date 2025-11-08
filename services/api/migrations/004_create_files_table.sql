-- Create files table
-- Stores metadata for individual files within an upload
-- Each upload can contain multiple files
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    upload_id UUID NOT NULL REFERENCES uploads(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    s3_key VARCHAR(512) NOT NULL UNIQUE,
    size_bytes BIGINT NOT NULL CHECK (size_bytes > 0),
    mime_type VARCHAR(100),
    sha256_hash VARCHAR(64), -- For deduplication (hex-encoded SHA256)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fetching all files in an upload
CREATE INDEX idx_files_upload_id ON files(upload_id);

-- Index for S3 key lookups and cleanup operations
CREATE INDEX idx_files_s3_key ON files(s3_key);
