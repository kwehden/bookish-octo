use ledger_posting::EntrySide;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedPostingLine {
    pub account_id: String,
    pub entry_side: EntrySide,
    pub amount_minor: i64,
    pub currency: String,
    pub base_amount_minor: i64,
    pub base_currency: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RuleEngineError {
    #[error("unsupported event type: {0}")]
    UnsupportedEventType(String),
    #[error("missing required field `{0}`")]
    MissingField(&'static str),
    #[error("invalid numeric field `{0}`")]
    InvalidNumber(&'static str),
    #[error("invalid settlement math: gross != net + fee")]
    InvalidSettlementMath,
    #[error("invalid entry side `{0}`")]
    InvalidEntrySide(String),
}

pub fn derive_lines_v1(
    event_type: &str,
    payload: &Value,
) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    match event_type {
        "order.captured.v1" => order_captured(payload),
        "payment.settled.v1" => payment_settled(payload),
        "refund.v1" => refund(payload),
        "inntopia.reservation.captured.v1" => inntopia_reservation_captured(payload),
        unsupported => Err(RuleEngineError::UnsupportedEventType(
            unsupported.to_string(),
        )),
    }
}

fn order_captured(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &[
            "/amount_minor",
            "/totals/grand_total_minor",
            "/totals/grand_total/amount_minor",
        ],
        "amount_minor",
    )?;
    let currency = first_string(
        payload,
        &[
            "/currency",
            "/totals/currency",
            "/totals/grand_total/currency",
        ],
    )
    .unwrap_or_else(|| "USD".to_string());
    let base_amount_minor = optional_i64(payload, &["/base_amount_minor"]).unwrap_or(amount);
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    Ok(vec![
        line(
            "1105-CASH-CLEARING",
            EntrySide::Debit,
            amount,
            &currency,
            base_amount_minor,
            &base_currency,
        ),
        line(
            "4000-REVENUE",
            EntrySide::Credit,
            amount,
            &currency,
            base_amount_minor,
            &base_currency,
        ),
    ])
}

fn payment_settled(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let gross = require_positive_i64(
        payload,
        &["/gross_amount_minor", "/amount_minor"],
        "gross_amount_minor",
    )?;
    let fee = optional_i64(payload, &["/fee_amount_minor"]).unwrap_or(0);
    if fee < 0 {
        return Err(RuleEngineError::InvalidNumber("fee_amount_minor"));
    }
    let net = optional_i64(payload, &["/net_amount_minor"]).unwrap_or(gross - fee);
    if net < 0 {
        return Err(RuleEngineError::InvalidNumber("net_amount_minor"));
    }
    if gross != net + fee {
        return Err(RuleEngineError::InvalidSettlementMath);
    }

    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    let mut lines = vec![line(
        "1000-CASH",
        EntrySide::Debit,
        net,
        &currency,
        net,
        &base_currency,
    )];
    if fee > 0 {
        lines.push(line(
            "6100-PAYMENT-FEES",
            EntrySide::Debit,
            fee,
            &currency,
            fee,
            &base_currency,
        ));
    }
    lines.push(line(
        "1105-CASH-CLEARING",
        EntrySide::Credit,
        gross,
        &currency,
        gross,
        &base_currency,
    ));
    Ok(lines)
}

fn refund(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/amount_minor", "/refund_amount_minor"],
        "amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    Ok(vec![
        line(
            "4050-REFUNDS",
            EntrySide::Debit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
        line(
            "1105-CASH-CLEARING",
            EntrySide::Credit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
    ])
}

fn inntopia_reservation_captured(
    payload: &Value,
) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/total_amount_minor", "/amount_minor"],
        "total_amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    Ok(vec![
        line(
            "1105-CASH-CLEARING",
            EntrySide::Debit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
        line(
            "2200-DEFERRED-REVENUE-RESERVATIONS",
            EntrySide::Credit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
    ])
}

fn line(
    account_id: &str,
    entry_side: EntrySide,
    amount_minor: i64,
    currency: &str,
    base_amount_minor: i64,
    base_currency: &str,
) -> DerivedPostingLine {
    DerivedPostingLine {
        account_id: account_id.to_string(),
        entry_side,
        amount_minor,
        currency: currency.to_string(),
        base_amount_minor,
        base_currency: base_currency.to_string(),
    }
}

fn require_positive_i64(
    payload: &Value,
    pointers: &[&str],
    field_name: &'static str,
) -> Result<i64, RuleEngineError> {
    let value = optional_i64(payload, pointers).ok_or(RuleEngineError::MissingField(field_name))?;
    if value <= 0 {
        return Err(RuleEngineError::InvalidNumber(field_name));
    }
    Ok(value)
}

fn optional_i64(payload: &Value, pointers: &[&str]) -> Option<i64> {
    pointers.iter().find_map(|pointer| {
        payload.pointer(pointer).and_then(|value| match value {
            Value::Number(number) => number.as_i64(),
            Value::String(string) => string.parse().ok(),
            _ => None,
        })
    })
}

fn first_string(payload: &Value, pointers: &[&str]) -> Option<String> {
    pointers
        .iter()
        .find_map(|pointer| payload.pointer(pointer).and_then(Value::as_str))
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use ledger_posting::EntrySide;
    use serde_json::json;

    use super::{derive_lines_v1, DerivedPostingLine, RuleEngineError};

    #[test]
    fn order_captured_maps_to_revenue_and_cash_clearing() {
        let lines = derive_lines_v1(
            "order.captured.v1",
            &json!({"amount_minor": 10000, "currency": "USD"}),
        )
        .unwrap();

        assert_eq!(lines.len(), 2);
        assert_balanced(&lines);
        assert_eq!(lines[0].entry_side, EntrySide::Debit);
        assert_eq!(lines[1].entry_side, EntrySide::Credit);
    }

    #[test]
    fn payment_settled_maps_to_cash_fee_and_clearing() {
        let lines = derive_lines_v1(
            "payment.settled.v1",
            &json!({
                "gross_amount_minor": 10000,
                "fee_amount_minor": 250,
                "net_amount_minor": 9750,
                "currency": "USD"
            }),
        )
        .unwrap();

        assert_eq!(lines.len(), 3);
        assert_balanced(&lines);
        assert_eq!(lines[2].account_id, "1105-CASH-CLEARING");
    }

    #[test]
    fn refund_maps_to_contra_revenue() {
        let lines = derive_lines_v1(
            "refund.v1",
            &json!({"refund_amount_minor": 1500, "currency": "USD"}),
        )
        .unwrap();

        assert_eq!(lines.len(), 2);
        assert_balanced(&lines);
        assert_eq!(lines[0].account_id, "4050-REFUNDS");
    }

    #[test]
    fn inntopia_reservation_maps_to_deferred_revenue() {
        let lines = derive_lines_v1(
            "inntopia.reservation.captured.v1",
            &json!({"total_amount_minor": 41250, "currency": "USD"}),
        )
        .unwrap();

        assert_eq!(lines.len(), 2);
        assert_balanced(&lines);
        assert_eq!(lines[1].account_id, "2200-DEFERRED-REVENUE-RESERVATIONS");
    }

    #[test]
    fn unsupported_type_is_rejected() {
        let error = derive_lines_v1("unknown.event.v1", &json!({})).unwrap_err();
        assert_eq!(
            error,
            RuleEngineError::UnsupportedEventType("unknown.event.v1".to_string())
        );
    }

    #[test]
    fn invalid_settlement_math_is_rejected() {
        let error = derive_lines_v1(
            "payment.settled.v1",
            &json!({
                "gross_amount_minor": 10000,
                "fee_amount_minor": 300,
                "net_amount_minor": 9800
            }),
        )
        .unwrap_err();

        assert_eq!(error, RuleEngineError::InvalidSettlementMath);
    }

    fn assert_balanced(lines: &[DerivedPostingLine]) {
        let debit_total = lines
            .iter()
            .filter(|line| line.entry_side == EntrySide::Debit)
            .map(|line| line.amount_minor)
            .sum::<i64>();
        let credit_total = lines
            .iter()
            .filter(|line| line.entry_side == EntrySide::Credit)
            .map(|line| line.amount_minor)
            .sum::<i64>();
        assert_eq!(debit_total, credit_total);
    }
}
