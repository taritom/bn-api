-- Cancel any pending expired transfers. The logic blocks multiple pending
-- transfers from occuring but allows pending if the previous was expired so
-- with no expiration date we'd otherwise have multiple pending transfers in flight.
UPDATE transfers
    SET status = 'Cancelled'
    WHERE transfer_expiry_date < NOW()
    AND status = 'Pending';

ALTER TABLE transfers
    DROP transfer_expiry_date;
