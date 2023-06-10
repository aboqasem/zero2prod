CREATE TYPE subscription_status AS ENUM ('PENDING_CONFIRMATION', 'CONFIRMED');

-- To cater for rolling updates, `status` will default to `CONFIRMED` mimicking the previous behaviour.
-- Default should be removed once we make sure all deployed instances provide the `status`.
ALTER TABLE subscriptions
ADD COLUMN status subscription_status NOT NULL DEFAULT 'CONFIRMED'::subscription_status;
