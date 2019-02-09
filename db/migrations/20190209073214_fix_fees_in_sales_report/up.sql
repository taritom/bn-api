DROP FUNCTION ticket_sales_per_ticket_pricing(start TIMESTAMP, "end" TIMESTAMP, group_by_ticket_type BOOLEAN, group_by_event_id BOOLEAN);
CREATE OR REPLACE FUNCTION ticket_sales_per_ticket_pricing(start TIMESTAMP, "end" TIMESTAMP, group_by_ticket_type BOOLEAN, group_by_event_id BOOLEAN)
    RETURNS TABLE
            (
                organization_id                  UUID,
                event_id                         UUID,
                ticket_type_id                   UUID,
                ticket_pricing_id                UUID,
                ticket_name                      TEXT,
                ticket_status                    TEXT,
                event_name                       TEXT,
                ticket_pricing_name              TEXT,
                ticket_pricing_price_in_cents    BIGINT,
                box_office_sales_in_cents        BIGINT,
                online_sales_in_cents            BIGINT,
                box_office_refunded_count        BIGINT,
                online_refunded_count            BIGINT,
                box_office_sale_count            BIGINT,
                online_sale_count                BIGINT,
                comp_sale_count                  BIGINT,
                total_box_office_fees_in_cents   BIGINT,
                total_online_fees_in_cents       BIGINT,
                company_box_office_fees_in_cents BIGINT,
                client_box_office_fees_in_cents  BIGINT,
                company_online_fees_in_cents     BIGINT,
                client_online_fees_in_cents      BIGINT

            )
AS
$body$
SELECT e.organization_id                                                                                                                                                                                                                                       AS organization_id,
       e.id                                                                                                                                                                                                                                                    AS event_id,
       tt.id                                                                                                                                                                                                                                                   AS ticket_type_id,
       tp.id                                                                                                                                                                                                                                                   AS ticket_pricing_id,
       tt.name                                                                                                                                                                                                                                                 AS ticket_name,
       tt.status                                                                                                                                                                                                                                               AS ticket_status,
       e.name                                                                                                                                                                                                                                                  AS event_name,
       tp.name                                                                                                                                                                                                                                                 AS ticket_pricing_name,
       tp.price_in_cents                                                                                                                                                                                                                                       AS ticket_pricing_price_in_cents,
       -- Actual Values
       CAST(COALESCE(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) FILTER (WHERE p.is_box_office IS TRUE AND o.status = 'Paid'), 0) AS BIGINT)                                                                                            AS box_office_sales_in_cents,
       CAST(COALESCE(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) FILTER (WHERE p.is_box_office IS FALSE AND o.status = 'Paid'), 0) AS BIGINT)                                                                                           AS online_sales_in_cents,

       -- Refunded count
       CAST(COALESCE(SUM(oi.refunded_quantity) FILTER (WHERE p.is_box_office IS TRUE AND o.status = 'Paid'), 0) AS BIGINT)                                                                                                                                     AS box_office_refunded_count,
       CAST(COALESCE(SUM(oi.refunded_quantity) FILTER (WHERE p.is_box_office IS FALSE AND o.status = 'Paid'), 0) AS BIGINT)                                                                                                                                    AS online_refunded_count,

       --Total Sold Count
       CAST(COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE p.is_box_office IS TRUE AND o.status = 'Paid' AND (h.hold_type IS NULL OR h.hold_type != 'Comp')), 0) AS BIGINT)                                                                    AS box_office_sale_count,
       CAST(COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE p.is_box_office IS FALSE AND o.status = 'Paid' AND (h.hold_type IS NULL OR h.hold_type != 'Comp')), 0) AS BIGINT)                                                                   AS online_sale_count,
       CAST(COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE h.hold_type = 'Comp' AND o.status = 'Paid'), 0) AS BIGINT)                                                                                                                          AS comp_sale_count,


       -- Total box office fees
       CAST(COALESCE(SUM((COALESCE(oi_t_fees.unit_price_in_cents, 0) * (COALESCE(oi_t_fees.quantity, 0) - COALESCE(oi_t_fees.refunded_quantity, 0))) + (COALESCE(oi_e_fees.unit_price_in_cents, 0) * (COALESCE(oi_e_fees.quantity, 0) - COALESCE(oi_e_fees.refunded_quantity, 0))))
                         FILTER (WHERE p.is_box_office IS TRUE), 0) AS BIGINT)                                                                                                               AS total_box_office_fees_in_cents,
       -- Total online fees
       CAST(COALESCE(SUM((COALESCE(oi_t_fees.unit_price_in_cents, 0) * (COALESCE(oi_t_fees.quantity, 0) - COALESCE(oi_t_fees.refunded_quantity, 0))) + (COALESCE(oi_e_fees.unit_price_in_cents, 0) * (COALESCE(oi_e_fees.quantity, 0) - COALESCE(oi_e_fees.refunded_quantity, 0))))
                         FILTER (WHERE p.is_box_office IS FALSE), 0) AS BIGINT)                                                                                                              AS total_online_fees_in_cents,
       -- Company Box Office Fees
       CAST(COALESCE(SUM((COALESCE(oi_t_fees.company_fee_in_cents, 0) * (COALESCE(oi_t_fees.quantity, 0) - COALESCE(oi_t_fees.refunded_quantity, 0))) + (COALESCE(oi_e_fees.company_fee_in_cents, 0) * (COALESCE(oi_e_fees.quantity, 0) - COALESCE(oi_e_fees.refunded_quantity, 0))))
                         FILTER (WHERE p.is_box_office IS TRUE),
                     0) AS BIGINT)                                                                                                                                                           AS company_box_office_fees_in_cents,
       -- Client Box Office Fees
       CAST(COALESCE(SUM((COALESCE(oi_t_fees.client_fee_in_cents, 0) * (COALESCE(oi_t_fees.quantity, 0) - COALESCE(oi_t_fees.refunded_quantity, 0))) + (COALESCE(oi_e_fees.client_fee_in_cents, 0) * (COALESCE(oi_e_fees.quantity, 0) - COALESCE(oi_e_fees.refunded_quantity, 0))))
                         FILTER (WHERE p.is_box_office IS TRUE), 0) AS BIGINT)                                                                                                               AS client_box_office_fees_in_cents,
       -- Company Online Fees
       CAST(COALESCE(SUM((COALESCE(oi_t_fees.company_fee_in_cents, 0) * (COALESCE(oi_t_fees.quantity, 0) - COALESCE(oi_t_fees.refunded_quantity, 0))) + (COALESCE(oi_e_fees.company_fee_in_cents, 0) * (COALESCE(oi_e_fees.quantity, 0) - COALESCE(oi_e_fees.refunded_quantity, 0))))
                         FILTER (WHERE p.is_box_office IS FALSE), 0) AS BIGINT)                                                                                                              AS company_online_fees_in_cents,
       -- Client Online Fees
       CAST(COALESCE(SUM((COALESCE(oi_t_fees.client_fee_in_cents, 0) * (COALESCE(oi_t_fees.quantity, 0) - COALESCE(oi_t_fees.refunded_quantity, 0))) + (COALESCE(oi_e_fees.client_fee_in_cents, 0) * (COALESCE(oi_e_fees.quantity, 0) - COALESCE(oi_e_fees.refunded_quantity, 0))))
                         FILTER (WHERE p.is_box_office IS FALSE), 0) AS BIGINT)                                                                                                              AS client_online_fees_in_cents
FROM order_items oi
         LEFT JOIN orders o on oi.order_id = o.id
         LEFT JOIN events e on oi.event_id = e.id
         LEFT JOIN holds h ON oi.hold_id = h.id
         LEFT JOIN order_items oi_t_fees ON oi_t_fees.parent_id = oi.id AND oi_t_fees.item_type = 'PerUnitFees'
         LEFT JOIN order_items oi_e_fees ON oi_e_fees.order_id = oi.order_id AND oi_e_fees.item_type = 'EventFees'
         LEFT JOIN (SELECT order_id, CAST(ARRAY_TO_STRING(ARRAY_AGG(DISTINCT p.payment_method), ', ') LIKE '%External' AS BOOLEAN) AS is_box_office FROM payments p GROUP BY p.payment_method, p.order_id) AS p on o.id = p.order_id
         LEFT JOIN (SELECT tt.id, tt.name, tt.status FROM ticket_types tt WHERE $4 IS NOT TRUE) AS tt ON tt.id = oi.ticket_type_id
         LEFT JOIN (SELECT tp.id, tp.name, tp.price_in_cents FROM ticket_pricing tp WHERE $3 IS NOT TRUE) AS tp ON oi.ticket_pricing_id = tp.id
WHERE oi.ticket_type_id IS NOT NULL
  AND ($1 IS NULL OR o.paid_at >= $1)
  AND ($2 IS NULL OR o.paid_at <= $2)
GROUP BY e.id, tt.id, tt.name, tt.status, tp.id, tp.name, tp.price_in_cents;
$body$
    LANGUAGE SQL;