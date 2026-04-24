-- Payment tracking table
CREATE TABLE IF NOT EXISTS payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID REFERENCES organizations(id),
    subscription_id UUID REFERENCES subscriptions(id),
    invoice_id UUID REFERENCES invoices(id),
    truelayer_payment_id VARCHAR(255),
    truelayer_mandate_id VARCHAR(255),
    amount_cents BIGINT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'GBP',
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    description TEXT,
    truelayer_response JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_payments_organization ON payments(organization_id);
CREATE INDEX IF NOT EXISTS idx_payments_subscription ON payments(subscription_id);
CREATE INDEX IF NOT EXISTS idx_payments_truelayer_id ON payments(truelayer_payment_id);

-- Add mandate tracking to subscriptions
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS truelayer_mandate_id VARCHAR(255);
ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS mandate_status VARCHAR(50);
