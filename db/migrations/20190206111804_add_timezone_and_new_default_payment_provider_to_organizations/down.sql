ALTER TABLE organizations
  ALTER allowed_payment_providers DROP DEFAULT;

ALTER TABLE organizations
  DROP timezone,
  ALTER allowed_payment_providers SET DEFAULT '{"stripe"}';

Update organizations
set allowed_payment_providers = '{"stripe"}' where allowed_payment_providers = '{"Stripe"}';