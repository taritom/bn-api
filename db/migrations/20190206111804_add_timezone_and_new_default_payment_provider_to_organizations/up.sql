ALTER TABLE organizations
  ALTER allowed_payment_providers DROP DEFAULT;

ALTER TABLE organizations
  ADD timezone Text,
  ALTER allowed_payment_providers SET DEFAULT '{"Stripe"}';

Update organizations
set allowed_payment_providers = '{"Stripe"}' where allowed_payment_providers = '{"stripe"}';