CREATE TABLE subscription_tokens (
    id TEXT NOT NULL,
    PRIMARY KEY (id),
    subscription_id uuid NOT NULL REFERENCES subscriptions (id)
);
