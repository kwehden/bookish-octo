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
        "fee.assessed.v1" => fee_assessed(payload),
        "chargeback.created.v1" => chargeback_created(payload),
        "payout.cleared.v1" => payout_cleared(payload),
        "dispute.opened.v1" => dispute_opened(payload),
        "dispute.won.v1" => dispute_won(payload),
        "dispute.lost.v1" => dispute_lost(payload),
        "inntopia.reservation.captured.v1" => inntopia_reservation_captured(payload),
        "intercompany.due_to_due_from.v1" => intercompany_due_to_due_from(payload),
        "consolidation.elimination.v1" => consolidation_elimination(payload),
        "fx.translation.v1" => fx_translation(payload),
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

fn fee_assessed(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/fee_amount_minor", "/amount_minor"],
        "fee_amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    Ok(vec![
        line(
            "6100-PAYMENT-FEES",
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

fn chargeback_created(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/amount_minor", "/chargeback_amount_minor"],
        "amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    Ok(vec![
        line(
            "6150-CHARGEBACK-LOSSES",
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

fn payout_cleared(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/amount_minor", "/net_amount_minor"],
        "amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    Ok(vec![
        line(
            "1010-BANK-OPERATING",
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

fn dispute_opened(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/amount_minor", "/dispute_amount_minor"],
        "amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    Ok(vec![
        line(
            "1205-DISPUTE-RECEIVABLE",
            EntrySide::Debit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
        line(
            "6150-CHARGEBACK-LOSSES",
            EntrySide::Credit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
    ])
}

fn dispute_won(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/amount_minor", "/dispute_amount_minor"],
        "amount_minor",
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
            "1205-DISPUTE-RECEIVABLE",
            EntrySide::Credit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
    ])
}

fn dispute_lost(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/amount_minor", "/dispute_amount_minor"],
        "amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    Ok(vec![
        line(
            "6150-CHARGEBACK-LOSSES",
            EntrySide::Debit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
        line(
            "1205-DISPUTE-RECEIVABLE",
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

fn intercompany_due_to_due_from(
    payload: &Value,
) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/amount_minor", "/due_to_due_from_amount_minor"],
        "amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    let due_from_account = first_string(payload, &["/due_from_account_id"])
        .unwrap_or_else(|| "1305-DUE-FROM-AFFILIATES".to_string());
    let due_to_account = first_string(payload, &["/due_to_account_id"])
        .unwrap_or_else(|| "2305-DUE-TO-AFFILIATES".to_string());

    Ok(vec![
        line(
            &due_from_account,
            EntrySide::Debit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
        line(
            &due_to_account,
            EntrySide::Credit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
    ])
}

fn consolidation_elimination(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let amount = require_positive_i64(
        payload,
        &["/amount_minor", "/elimination_amount_minor"],
        "amount_minor",
    )?;
    let currency = first_string(payload, &["/currency"]).unwrap_or_else(|| "USD".to_string());
    let base_currency = first_string(payload, &["/base_currency"]).unwrap_or(currency.clone());

    let debit_account = first_string(payload, &["/elimination_debit_account_id"])
        .unwrap_or_else(|| "4999-INTERCOMPANY-ELIMINATION".to_string());
    let credit_account = first_string(payload, &["/elimination_credit_account_id"])
        .unwrap_or_else(|| "5999-INTERCOMPANY-ELIMINATION".to_string());

    Ok(vec![
        line(
            &debit_account,
            EntrySide::Debit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
        line(
            &credit_account,
            EntrySide::Credit,
            amount,
            &currency,
            amount,
            &base_currency,
        ),
    ])
}

fn fx_translation(payload: &Value) -> Result<Vec<DerivedPostingLine>, RuleEngineError> {
    let translation_amount = optional_i64(
        payload,
        &[
            "/translation_amount_minor",
            "/fx_translation_amount_minor",
            "/amount_minor",
        ],
    )
    .ok_or(RuleEngineError::MissingField("translation_amount_minor"))?;
    if translation_amount == 0 {
        return Err(RuleEngineError::InvalidNumber("translation_amount_minor"));
    }

    let amount = translation_amount.abs();
    let currency = first_string(payload, &["/base_currency", "/currency"])
        .unwrap_or_else(|| "USD".to_string());

    if translation_amount > 0 {
        Ok(vec![
            line(
                "3100-CUMULATIVE-TRANSLATION-ADJUSTMENT",
                EntrySide::Debit,
                amount,
                &currency,
                amount,
                &currency,
            ),
            line(
                "7300-FX-TRANSLATION-GAIN-LOSS",
                EntrySide::Credit,
                amount,
                &currency,
                amount,
                &currency,
            ),
        ])
    } else {
        Ok(vec![
            line(
                "7300-FX-TRANSLATION-GAIN-LOSS",
                EntrySide::Debit,
                amount,
                &currency,
                amount,
                &currency,
            ),
            line(
                "3100-CUMULATIVE-TRANSLATION-ADJUSTMENT",
                EntrySide::Credit,
                amount,
                &currency,
                amount,
                &currency,
            ),
        ])
    }
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
    fn fee_assessed_maps_to_fee_expense_and_cash_clearing() {
        let lines = derive_lines_v1(
            "fee.assessed.v1",
            &json!({"fee_amount_minor": 325, "currency": "USD"}),
        )
        .unwrap();

        assert_eq!(lines.len(), 2);
        assert_balanced(&lines);
        assert_eq!(lines[0].account_id, "6100-PAYMENT-FEES");
        assert_eq!(lines[1].account_id, "1105-CASH-CLEARING");
    }

    #[test]
    fn chargeback_created_maps_to_losses_and_clearing() {
        let lines = derive_lines_v1(
            "chargeback.created.v1",
            &json!({"chargeback_amount_minor": 10000, "currency": "USD"}),
        )
        .unwrap();

        assert_eq!(lines.len(), 2);
        assert_balanced(&lines);
        assert_eq!(lines[0].account_id, "6150-CHARGEBACK-LOSSES");
        assert_eq!(lines[1].account_id, "1105-CASH-CLEARING");
    }

    #[test]
    fn payout_cleared_maps_to_bank_and_clearing() {
        let lines = derive_lines_v1(
            "payout.cleared.v1",
            &json!({"amount_minor": 9750, "currency": "USD"}),
        )
        .unwrap();

        assert_eq!(lines.len(), 2);
        assert_balanced(&lines);
        assert_eq!(lines[0].account_id, "1010-BANK-OPERATING");
        assert_eq!(lines[1].account_id, "1105-CASH-CLEARING");
    }

    #[test]
    fn dispute_lifecycle_states_map_with_balanced_lines() {
        let opened = derive_lines_v1(
            "dispute.opened.v1",
            &json!({"amount_minor": 5000, "currency": "USD"}),
        )
        .unwrap();
        assert_balanced(&opened);
        assert_eq!(opened[0].account_id, "1205-DISPUTE-RECEIVABLE");
        assert_eq!(opened[1].account_id, "6150-CHARGEBACK-LOSSES");

        let won = derive_lines_v1(
            "dispute.won.v1",
            &json!({"amount_minor": 5000, "currency": "USD"}),
        )
        .unwrap();
        assert_balanced(&won);
        assert_eq!(won[0].account_id, "1105-CASH-CLEARING");
        assert_eq!(won[1].account_id, "1205-DISPUTE-RECEIVABLE");

        let lost = derive_lines_v1(
            "dispute.lost.v1",
            &json!({"amount_minor": 5000, "currency": "USD"}),
        )
        .unwrap();
        assert_balanced(&lost);
        assert_eq!(lost[0].account_id, "6150-CHARGEBACK-LOSSES");
        assert_eq!(lost[1].account_id, "1205-DISPUTE-RECEIVABLE");
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

    #[test]
    fn dispute_opened_requires_positive_amount() {
        let error = derive_lines_v1("dispute.opened.v1", &json!({"amount_minor": 0})).unwrap_err();
        assert_eq!(error, RuleEngineError::InvalidNumber("amount_minor"));
    }

    #[test]
    fn intercompany_due_to_due_from_maps_to_intercompany_accounts() {
        let lines = derive_lines_v1(
            "intercompany.due_to_due_from.v1",
            &json!({"amount_minor": 15000, "currency": "USD"}),
        )
        .unwrap();

        assert_eq!(lines.len(), 2);
        assert_balanced(&lines);
        assert_eq!(lines[0].account_id, "1305-DUE-FROM-AFFILIATES");
        assert_eq!(lines[1].account_id, "2305-DUE-TO-AFFILIATES");
    }

    #[test]
    fn consolidation_elimination_maps_to_elimination_accounts() {
        let lines = derive_lines_v1(
            "consolidation.elimination.v1",
            &json!({"elimination_amount_minor": 7750, "currency": "USD"}),
        )
        .unwrap();

        assert_eq!(lines.len(), 2);
        assert_balanced(&lines);
        assert_eq!(lines[0].account_id, "4999-INTERCOMPANY-ELIMINATION");
        assert_eq!(lines[1].account_id, "5999-INTERCOMPANY-ELIMINATION");
    }

    #[test]
    fn fx_translation_positive_and_negative_both_balance() {
        let positive = derive_lines_v1(
            "fx.translation.v1",
            &json!({"translation_amount_minor": 1234, "base_currency": "USD"}),
        )
        .unwrap();
        assert_balanced(&positive);
        assert_eq!(
            positive[0].account_id,
            "3100-CUMULATIVE-TRANSLATION-ADJUSTMENT"
        );
        assert_eq!(positive[1].account_id, "7300-FX-TRANSLATION-GAIN-LOSS");

        let negative = derive_lines_v1(
            "fx.translation.v1",
            &json!({"translation_amount_minor": -1234, "base_currency": "USD"}),
        )
        .unwrap();
        assert_balanced(&negative);
        assert_eq!(negative[0].account_id, "7300-FX-TRANSLATION-GAIN-LOSS");
        assert_eq!(
            negative[1].account_id,
            "3100-CUMULATIVE-TRANSLATION-ADJUSTMENT"
        );
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
