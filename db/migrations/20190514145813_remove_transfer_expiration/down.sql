ALTER TABLE transfers
    ADD transfer_expiry_date TIMESTAMP NULL;

UPDATE transfers SET transfer_expiry_date = '2099-05-14 00:00:00';

ALTER TABLE transfers
    ALTER COLUMN transfer_expiry_date SET NOT NULL;
