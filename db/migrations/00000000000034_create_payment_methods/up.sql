CREATE TABLE payment_methods (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL REFERENCES users(id),
    name TEXT NOT NULL,
    is_default BOOLEAN NOT NULL,
    provider TEXT NOT NULL,
    provider_data json NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_payment_methods_user_id ON payment_methods (user_id);
CREATE UNIQUE INDEX index_payment_methods_user_id_name on payment_methods(user_id, name);