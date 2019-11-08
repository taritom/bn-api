--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
-- PROCESS SETTLEMENT FOR EVENT
--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
DROP FUNCTION IF EXISTS process_settlement_for_event(settlement_id UUID, event_id UUID, start TIMESTAMP, "end" TIMESTAMP);
CREATE OR REPLACE FUNCTION process_settlement_for_event(settlement_id UUID, event_id UUID, start TIMESTAMP, "end" TIMESTAMP) RETURNS void AS $$
DECLARE
  -- Override to handle swapping from rolling to post event in scenarios where settlements were manually processed prior (initial launch)
  start_override timestamp;
BEGIN

SELECT CASE WHEN s.only_finished_events = true THEN NULL ELSE s.start_time END
FROM settlements s
JOIN organizations o ON o.id = (SELECT organization_id FROM settlements WHERE id = $1)
ORDER BY s.created_at
LIMIT 1
INTO start_override;

CREATE TEMP TABLE order_item_ids (
   id UUID,
   refund_id UUID
);

INSERT INTO order_item_ids(id, refund_id)
SELECT oi.id, NULL
FROM order_items oi
INNER JOIN orders o on oi.order_id = o.id
LEFT JOIN holds h ON oi.hold_id = h.id
LEFT JOIN order_items oi_promo_code ON (oi_promo_code.item_type = 'Discount' AND oi.id = oi_promo_code.parent_id)
WHERE ($3 IS NULL OR o.paid_at >= $3)
AND (start_override IS NULL OR o.paid_at >= start_override)
AND ($4 IS NULL OR o.paid_at <= $4)
AND oi.event_id = $2
AND (oi.item_type <> 'EventFees' OR oi.client_fee_in_cents > 0)
AND oi.item_type <> 'CreditCardFees'
AND o.settlement_id IS NULL
AND o.status = 'Paid'
AND oi.parent_id IS NULL
AND (oi.unit_price_in_cents + COALESCE(oi_promo_code.unit_price_in_cents, 0)) > 0
AND o.box_office_pricing IS FALSE;

-- Add refund items to the order items temp table
INSERT INTO order_item_ids(id, refund_id)
SELECT DISTINCT COALESCE(oi.parent_id, oi.id), r.id
FROM refunds r
INNER JOIN refund_items ri ON ri.refund_id = r.id
INNER JOIN order_items oi ON oi.id = ri.order_item_id
INNER JOIN orders o on oi.order_id = o.id
WHERE oi.event_id = $2
AND (oi.item_type <> 'EventFees' OR oi.client_fee_in_cents > 0)
AND oi.item_type <> 'CreditCardFees'
AND (start_override IS NULL OR r.created_at >= start_override)
AND ($3 IS NULL OR r.created_at >= $3)
AND ($4 IS NULL OR r.created_at <= $4)
AND o.settlement_id is distinct from $1
AND ri.amount > 0
AND r.settlement_id IS NULL
AND o.box_office_pricing IS FALSE;

INSERT INTO settlement_entries (settlement_id, event_id, ticket_type_id, face_value_in_cents, revenue_share_value_in_cents, online_sold_quantity, fee_sold_quantity, total_sales_in_cents, settlement_entry_type)
SELECT -- Group result set by face price to prevent multiple records for holds that match code discounts
  entries.settlement_id,
  entries.event_id,
  entries.ticket_type_id,
  entries.face_value_in_cents,
  entries.revenue_share_value_in_cents,
  SUM(online_sold_quantity),
  SUM(fee_sold_quantity),
  SUM(online_sold_quantity) * entries.face_value_in_cents + SUM(fee_sold_quantity) * entries.revenue_share_value_in_cents,
  entries.settlement_entry_type
FROM (
  SELECT
    $1 as settlement_id,
    oi.event_id,
    oi.ticket_type_id,
    CASE oi.item_type WHEN 'EventFees' THEN 0 ELSE CAST(oi.unit_price_in_cents + COALESCE(oi_promo_code.unit_price_in_cents, 0) AS BIGINT) END as face_value_in_cents,
    -- Event fees record list the fee as part of the revenue share for that item with 0 face value
    CASE oi.item_type WHEN 'EventFees' THEN CAST(oi.client_fee_in_cents AS BIGINT) ELSE CAST(COALESCE(oi_t_fees.client_fee_in_cents, 0) AS BIGINT) END as revenue_share_value_in_cents,
    -- Event fees list their quantity in the fee_sold_quantity field
    CASE oi.item_type
      WHEN 'EventFees' THEN 0
      ELSE
        CASE WHEN oi_r.quantity IS NOT NULL THEN
          CAST(-SUM(oi_r.quantity) AS BIGINT)
        ELSE
          CAST(SUM(oi.quantity) AS BIGINT)
        END
    END as online_sold_quantity,
    CASE oi.item_type
      WHEN 'EventFees' THEN
        CASE WHEN oi_r.quantity IS NOT NULL THEN
          CAST(-SUM(oi_r.quantity) AS BIGINT)
        ELSE
          CAST(SUM(oi.quantity) AS BIGINT)
        END
      ELSE
        CASE WHEN oi_t_fees_r.quantity IS NOT NULL THEN
          CAST(-SUM(oi_t_fees_r.quantity) AS BIGINT)
        ELSE
          CAST(SUM(COALESCE(oi_t_fees.quantity, 0)) AS BIGINT)
        END
    END as fee_sold_quantity,
    CASE oi.item_type WHEN 'EventFees' THEN 'EventFees' ELSE 'TicketType' END as settlement_entry_type
  FROM order_items oi
  INNER JOIN order_item_ids oi_ids ON oi.id = oi_ids.id
  INNER JOIN orders o ON oi.order_id = o.id
  LEFT JOIN refund_items oi_r ON oi_r.order_item_id = oi.id AND oi_r.refund_id = oi_ids.refund_id
  LEFT JOIN order_items oi_promo_code ON (oi_promo_code.item_type = 'Discount' AND oi.id = oi_promo_code.parent_id)
  LEFT JOIN order_items oi_t_fees ON oi_t_fees.parent_id = oi.id AND oi_t_fees.item_type = 'PerUnitFees'
  LEFT JOIN refund_items oi_t_fees_r ON oi_t_fees_r.order_item_id = oi_t_fees.id AND oi_t_fees_r.refund_id = oi_ids.refund_id
  GROUP BY
    oi.item_type,
    oi.event_id,
    oi.ticket_type_id,
    oi.unit_price_in_cents,
    oi.client_fee_in_cents,
    oi_t_fees.client_fee_in_cents,
    oi_promo_code.unit_price_in_cents,
    oi_t_fees_r.quantity,
    oi_r.quantity
) entries
  GROUP BY
    entries.settlement_id,
    entries.event_id,
    entries.ticket_type_id,
    entries.face_value_in_cents,
    entries.revenue_share_value_in_cents,
    entries.settlement_entry_type
  -- Filter out any records where the sum of their quantities is 0
  -- Negative indicates a refund settlement adjustment, positive purchases
  HAVING
    SUM(online_sold_quantity) <> 0
  OR
    SUM(fee_sold_quantity) <> 0
;

-- Update associated orders as part of this settlement
UPDATE orders SET settlement_id = $1
FROM order_item_ids oi_ids
JOIN order_items oi ON oi.id = oi_ids.id
JOIN orders o ON oi.order_id = o.id
WHERE orders.id = o.id
AND orders.settlement_id IS NULL
AND oi_ids.refund_id IS NULL;

-- Update refunds that occurred during this settlement for orders in this settlement
UPDATE refunds SET settlement_id = $1
FROM order_item_ids oi_ids
WHERE refunds.id = oi_ids.refund_id
AND refunds.settlement_id IS NULL
AND oi_ids.refund_id IS NOT NULL;

DROP TABLE order_item_ids;

END $$ LANGUAGE 'plpgsql';
