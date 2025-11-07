-- ============================================
-- RapidFab MVP - PostgreSQL Schema (Production-Ready)
-- Created: November 6, 2025
-- Includes all critical fixes from Codex review
-- ============================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ENUM TYPES
CREATE TYPE user_role AS ENUM ('customer', 'supplier', 'admin', 'moderator');
CREATE TYPE quote_status AS ENUM ('pending', 'accepted', 'expired', 'rejected', 'cancelled');
CREATE TYPE order_status AS ENUM ('pending', 'paid', 'production', 'qc', 'shipped', 'delivered', 'cancelled', 'refunded');
CREATE TYPE job_status AS ENUM ('assigned', 'accepted', 'in_production', 'qc', 'shipped', 'delivered');
CREATE TYPE payment_status AS ENUM ('pending', 'processing', 'succeeded', 'failed', 'refunded', 'disputed');
CREATE TYPE payout_status AS ENUM ('pending', 'scheduled', 'completed', 'failed');
CREATE TYPE supplier_tier AS ENUM ('standard', 'gold', 'platinum', 'inactive');
CREATE TYPE invoice_status AS ENUM ('draft', 'issued', 'paid', 'overdue', 'cancelled');

-- TABLE: customers
CREATE TABLE customers (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR(255) NOT NULL,
  full_name VARCHAR(255),
  phone VARCHAR(20),
  company_name VARCHAR(255),
  nip VARCHAR(20),
  role user_role DEFAULT 'customer'::user_role NOT NULL,
  is_admin BOOLEAN DEFAULT FALSE,
  email_verified BOOLEAN DEFAULT FALSE,
  email_verified_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  deleted_at TIMESTAMPTZ,
  CONSTRAINT email_valid CHECK (email ~ '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- TABLE: files (content-addressed storage)
CREATE TABLE files (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  sha256 VARCHAR(64) NOT NULL,
  size BIGINT NOT NULL,
  mime_type VARCHAR(50),
  customer_id UUID REFERENCES customers(id) ON DELETE SET NULL,
  upload_ip INET,
  claimed_at TIMESTAMPTZ,
  filename_original VARCHAR(255),
  storage_key VARCHAR(255) NOT NULL,
  s3_bucket VARCHAR(255) DEFAULT 'rapidfab-files',
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT file_size_positive CHECK (size > 0),
  CONSTRAINT file_size_max CHECK (size <= 1073741824),
  UNIQUE(sha256, s3_bucket)
);

-- TABLE: quotes (with consumption tracking)
CREATE TABLE quotes (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
  file_id UUID REFERENCES files(id) ON DELETE SET NULL,
  material VARCHAR(50) NOT NULL,
  infill_percent INT,
  support_type VARCHAR(50),
  quantity INT NOT NULL,
  unit_price DECIMAL(12,2) NOT NULL,
  total_price DECIMAL(12,2) NOT NULL,
  eta_p50_hours INT,
  eta_p95_hours INT,
  pricing_snapshot JSONB,
  status quote_status DEFAULT 'pending'::quote_status NOT NULL,
  consumed_by_order_id UUID UNIQUE,
  consumed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP + INTERVAL '24 hours',
  CONSTRAINT quantity_positive CHECK (quantity > 0),
  CONSTRAINT infill_range CHECK (infill_percent IS NULL OR (infill_percent >= 0 AND infill_percent <= 100)),
  CONSTRAINT prices_positive CHECK (unit_price >= 0 AND total_price >= 0),
  CONSTRAINT total_price_correct CHECK (ABS(total_price - (unit_price * quantity)) < 0.01),
  CONSTRAINT consumed_consistency CHECK ((consumed_by_order_id IS NULL AND consumed_at IS NULL) OR (consumed_by_order_id IS NOT NULL AND consumed_at IS NOT NULL))
);

-- TABLE: orders (with RESTRICT delete for payment safety)
CREATE TABLE orders (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  quote_id UUID NOT NULL UNIQUE REFERENCES quotes(id) ON DELETE RESTRICT,
  customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE RESTRICT,
  shipping_address JSONB NOT NULL,
  invoice_details JSONB NOT NULL,
  total_amount DECIMAL(12,2) NOT NULL,
  status order_status DEFAULT 'pending'::order_status NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  paid_at TIMESTAMPTZ,
  shipped_at TIMESTAMPTZ,
  delivered_at TIMESTAMPTZ,
  CONSTRAINT total_amount_positive CHECK (total_amount > 0),
  CONSTRAINT shipped_after_paid CHECK (shipped_at IS NULL OR paid_at IS NOT NULL),
  CONSTRAINT delivered_after_shipped CHECK (delivered_at IS NULL OR shipped_at IS NOT NULL)
);

-- TABLE: order_timeline (immutable audit log)
CREATE TABLE order_timeline (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
  status_from order_status,
  status_to order_status NOT NULL,
  triggered_by VARCHAR(50),
  changed_by_id UUID REFERENCES customers(id) ON DELETE SET NULL,
  notes TEXT,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT status_change_valid CHECK (status_from IS NULL OR status_from <> status_to)
);

-- TABLE: idempotency_keys (CRITICAL for payment safety)
CREATE TABLE idempotency_keys (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  key VARCHAR(255) UNIQUE NOT NULL,
  request_hash VARCHAR(64) NOT NULL,
  response_status INT,
  response_body JSONB,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP + INTERVAL '24 hours'
);

-- TABLE: payments (Stripe integration)
CREATE TABLE payments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id UUID NOT NULL REFERENCES orders(id) ON DELETE RESTRICT,
  payment_intent_id VARCHAR(255) UNIQUE,
  idempotency_key VARCHAR(255) REFERENCES idempotency_keys(key),
  amount DECIMAL(12,2) NOT NULL,
  currency VARCHAR(3) DEFAULT 'PLN' NOT NULL,
  status payment_status DEFAULT 'pending'::payment_status NOT NULL,
  succeeded_at TIMESTAMPTZ,
  failed_at TIMESTAMPTZ,
  refunded_at TIMESTAMPTZ,
  webhook_payload JSONB,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT amount_positive CHECK (amount > 0),
  CONSTRAINT status_consistency CHECK ((succeeded_at IS NULL OR failed_at IS NULL) AND (refunded_at IS NULL OR succeeded_at IS NOT NULL))
);

-- TABLE: escrow_log (for multi-supplier later)
CREATE TABLE escrow_log (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  payment_id UUID REFERENCES payments(id) ON DELETE CASCADE,
  status VARCHAR(50),
  amount DECIMAL(12,2),
  platform_fee DECIMAL(12,2),
  supplier_payout DECIMAL(12,2),
  held_at TIMESTAMPTZ,
  released_at TIMESTAMPTZ,
  refunded_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT amount_positive CHECK (amount > 0)
);

-- TABLE: invoices (Polish VAT compliance)
CREATE TABLE invoices (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id UUID NOT NULL UNIQUE REFERENCES orders(id) ON DELETE RESTRICT,
  invoice_year INT NOT NULL,
  invoice_month INT NOT NULL,
  invoice_seq INT NOT NULL,
  invoice_number VARCHAR(50) UNIQUE NOT NULL,
  subtotal DECIMAL(12,2) NOT NULL,
  vat_amount DECIMAL(12,2) NOT NULL,
  total_amount DECIMAL(12,2) NOT NULL,
  pdf_key VARCHAR(255),
  status invoice_status DEFAULT 'draft'::invoice_status NOT NULL,
  issued_at TIMESTAMPTZ,
  due_at TIMESTAMPTZ,
  paid_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT total_equals_subtotal_plus_vat CHECK (ABS(total_amount - (subtotal + vat_amount)) < 0.01),
  CONSTRAINT due_after_issued CHECK (due_at IS NULL OR due_at >= issued_at),
  CONSTRAINT vat_amount_positive CHECK (vat_amount >= 0),
  CONSTRAINT unique_invoice_per_month UNIQUE(invoice_year, invoice_month, invoice_seq),
  CONSTRAINT invoice_seq_positive CHECK (invoice_seq > 0)
);

-- TABLE: email_events (transactional email tracking)
CREATE TABLE email_events (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  recipient VARCHAR(255) NOT NULL,
  subject VARCHAR(255),
  body TEXT,
  status VARCHAR(50) DEFAULT 'pending' NOT NULL,
  sent_at TIMESTAMPTZ,
  failed_at TIMESTAMPTZ,
  attempts INT DEFAULT 0,
  retry_at TIMESTAMPTZ,
  error_message TEXT,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT recipient_valid CHECK (recipient ~ '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- TABLE: suppliers (for future marketplace)
CREATE TABLE suppliers (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(255) NOT NULL,
  email VARCHAR(255) UNIQUE NOT NULL,
  phone VARCHAR(20),
  capabilities JSONB,
  rating DECIMAL(3, 2),
  on_time_rate DECIMAL(5, 2),
  first_pass_yield DECIMAL(5, 2),
  total_jobs INT DEFAULT 0,
  tier supplier_tier DEFAULT 'standard'::supplier_tier,
  is_active BOOLEAN DEFAULT TRUE,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- TABLE: jobs (supplier production)
CREATE TABLE jobs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
  supplier_id UUID REFERENCES suppliers(id) ON DELETE SET NULL,
  status job_status DEFAULT 'assigned'::job_status NOT NULL,
  assigned_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  accepted_at TIMESTAMPTZ,
  started_at TIMESTAMPTZ,
  qc_uploaded_at TIMESTAMPTZ,
  shipped_at TIMESTAMPTZ,
  delivered_at TIMESTAMPTZ,
  notes TEXT,
  qc_report_key VARCHAR(255),
  tracking_number VARCHAR(100),
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT timeline_valid CHECK (accepted_at IS NULL OR assigned_at <= accepted_at)
);

-- INDICES (40+ total)
CREATE INDEX idx_customers_email ON customers(email);
CREATE INDEX idx_customers_deleted ON customers(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX idx_files_sha256 ON files(sha256);
CREATE INDEX idx_files_customer_id ON files(customer_id);
CREATE INDEX idx_files_created_at ON files(created_at DESC);
CREATE INDEX idx_quotes_customer_id ON quotes(customer_id);
CREATE INDEX idx_quotes_status ON quotes(status);
CREATE INDEX idx_quotes_expires ON quotes(expires_at, status) WHERE status = 'pending'::quote_status;
CREATE INDEX idx_quotes_customer_created ON quotes(customer_id, created_at DESC);
CREATE INDEX idx_orders_customer_id ON orders(customer_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_status_created ON orders(status, created_at DESC);
CREATE INDEX idx_orders_quote_id ON orders(quote_id);
CREATE INDEX idx_orders_created_at ON orders(created_at DESC);
CREATE INDEX idx_timeline_order_id ON order_timeline(order_id, created_at DESC);
CREATE INDEX idx_idempotency_key ON idempotency_keys(key);
CREATE INDEX idx_idempotency_expires ON idempotency_keys(expires_at) WHERE expires_at < CURRENT_TIMESTAMP;
CREATE INDEX idx_payments_order_id ON payments(order_id);
CREATE INDEX idx_payments_intent_id ON payments(payment_intent_id);
CREATE INDEX idx_payments_intent_status ON payments(payment_intent_id, status);
CREATE INDEX idx_payments_succeeded_at ON payments(succeeded_at) WHERE status = 'succeeded'::payment_status;
CREATE INDEX idx_payments_created_at ON payments(created_at DESC);
CREATE INDEX idx_invoices_order_id ON invoices(order_id);
CREATE INDEX idx_invoices_number ON invoices(invoice_number);
CREATE INDEX idx_invoices_customer ON invoices(invoice_year, invoice_month);
CREATE INDEX idx_email_retry ON email_events(retry_at) WHERE status = 'pending' AND attempts < 3;
CREATE INDEX idx_email_status ON email_events(status, created_at DESC);
CREATE INDEX idx_jobs_order_id ON jobs(order_id);
CREATE INDEX idx_jobs_supplier_id ON jobs(supplier_id);
CREATE INDEX idx_jobs_status ON jobs(status);
CREATE INDEX idx_orders_shipping ON orders USING GIN(shipping_address jsonb_path_ops);
CREATE INDEX idx_orders_invoice ON orders USING GIN(invoice_details jsonb_path_ops);

-- FUNCTIONS
CREATE OR REPLACE FUNCTION get_next_invoice_number()
RETURNS VARCHAR AS $$
DECLARE
  v_year INT;
  v_month INT;
  v_seq INT;
BEGIN
  v_year := EXTRACT(YEAR FROM CURRENT_DATE)::INT;
  v_month := EXTRACT(MONTH FROM CURRENT_DATE)::INT;
  SELECT COALESCE(MAX(invoice_seq), 0) + 1 INTO v_seq FROM invoices WHERE invoice_year = v_year AND invoice_month = v_month FOR UPDATE;
  RETURN 'FV-' || v_year || '-' || LPAD(v_month::TEXT, 2, '0') || '-' || LPAD(v_seq::TEXT, 6, '0');
END;
$$ LANGUAGE plpgsql;

-- SEED DATA
INSERT INTO customers (email, password_hash, full_name, role, is_admin, email_verified, email_verified_at)
VALUES ('admin@rapidfab.local', '$2b$12$placeholder_bcrypt_hash_replace_with_real', 'Admin User', 'admin'::user_role, TRUE, TRUE, CURRENT_TIMESTAMP)
ON CONFLICT (email) DO NOTHING;

CREATE TABLE IF NOT EXISTS alembic_version (version_num VARCHAR(32) NOT NULL PRIMARY KEY);
INSERT INTO alembic_version (version_num) VALUES ('001_init_schema') ON CONFLICT DO NOTHING;
