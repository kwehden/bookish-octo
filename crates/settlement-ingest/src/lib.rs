use chrono::NaiveDate;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripeSettlementLine {
    pub payout_id: String,
    pub balance_transaction_id: String,
    pub gateway_transaction_id: String,
    pub available_on: NaiveDate,
    pub currency: String,
    pub gross_minor: i64,
    pub fee_minor: i64,
    pub net_minor: i64,
    pub transaction_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BankStatementLine {
    pub statement_id: String,
    pub value_date: NaiveDate,
    pub bank_reference: String,
    pub description: String,
    pub currency: String,
    pub amount_minor: i64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum IngestError {
    #[error("csv parse error at line {line}: {message}")]
    Csv { line: usize, message: String },
    #[error("missing required field '{field}' at line {line}")]
    MissingField { line: usize, field: &'static str },
    #[error("invalid date '{value}' for field '{field}' at line {line}")]
    InvalidDate {
        line: usize,
        field: &'static str,
        value: String,
    },
    #[error("invalid amount '{value}' for field '{field}' at line {line}")]
    InvalidAmount {
        line: usize,
        field: &'static str,
        value: String,
    },
}

#[derive(Debug, Deserialize)]
struct StripeCsvRow {
    payout_id: Option<String>,
    balance_transaction_id: Option<String>,
    source_id: Option<String>,
    available_on: Option<String>,
    currency: Option<String>,
    gross: Option<String>,
    fee: Option<String>,
    net: Option<String>,
    #[serde(rename = "type")]
    transaction_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BankCsvRow {
    statement_id: Option<String>,
    value_date: Option<String>,
    bank_reference: Option<String>,
    description: Option<String>,
    currency: Option<String>,
    amount: Option<String>,
}

pub fn parse_stripe_settlement_csv(input: &str) -> Result<Vec<StripeSettlementLine>, IngestError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(input.as_bytes());

    let mut lines = Vec::new();
    for (idx, result) in reader.deserialize::<StripeCsvRow>().enumerate() {
        let line = idx + 2;
        let row = result.map_err(|err| map_csv_error(err, line))?;

        let payout_id = required(row.payout_id, "payout_id", line)?;
        let balance_transaction_id =
            required(row.balance_transaction_id, "balance_transaction_id", line)?;
        let gateway_transaction_id = required(row.source_id, "source_id", line)?;
        let available_on = parse_date(
            &required(row.available_on, "available_on", line)?,
            "available_on",
            line,
        )?;
        let currency = required(row.currency, "currency", line)?.to_ascii_uppercase();
        let gross_minor = parse_minor_units(&required(row.gross, "gross", line)?, "gross", line)?;
        let fee_minor = parse_minor_units(&required(row.fee, "fee", line)?, "fee", line)?;
        let net_minor = parse_minor_units(&required(row.net, "net", line)?, "net", line)?;
        let transaction_type = required(row.transaction_type, "type", line)?;

        lines.push(StripeSettlementLine {
            payout_id,
            balance_transaction_id,
            gateway_transaction_id,
            available_on,
            currency,
            gross_minor,
            fee_minor,
            net_minor,
            transaction_type,
        });
    }

    Ok(lines)
}

pub fn parse_bank_statement_csv(input: &str) -> Result<Vec<BankStatementLine>, IngestError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(input.as_bytes());

    let mut lines = Vec::new();
    for (idx, result) in reader.deserialize::<BankCsvRow>().enumerate() {
        let line = idx + 2;
        let row = result.map_err(|err| map_csv_error(err, line))?;

        let statement_id = required(row.statement_id, "statement_id", line)?;
        let value_date = parse_date(
            &required(row.value_date, "value_date", line)?,
            "value_date",
            line,
        )?;
        let bank_reference = required(row.bank_reference, "bank_reference", line)?;
        let description = required(row.description, "description", line)?;
        let currency = required(row.currency, "currency", line)?.to_ascii_uppercase();
        let amount_minor =
            parse_minor_units(&required(row.amount, "amount", line)?, "amount", line)?;

        lines.push(BankStatementLine {
            statement_id,
            value_date,
            bank_reference,
            description,
            currency,
            amount_minor,
        });
    }

    Ok(lines)
}

fn map_csv_error(err: csv::Error, fallback_line: usize) -> IngestError {
    let line = err
        .position()
        .and_then(|p| usize::try_from(p.line()).ok())
        .unwrap_or(fallback_line);

    IngestError::Csv {
        line,
        message: err.to_string(),
    }
}

fn required(
    value: Option<String>,
    field: &'static str,
    line: usize,
) -> Result<String, IngestError> {
    let value = value.ok_or(IngestError::MissingField { line, field })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(IngestError::MissingField { line, field });
    }
    Ok(trimmed.to_owned())
}

fn parse_date(input: &str, field: &'static str, line: usize) -> Result<NaiveDate, IngestError> {
    for fmt in ["%Y-%m-%d", "%m/%d/%Y"] {
        if let Ok(parsed) = NaiveDate::parse_from_str(input, fmt) {
            return Ok(parsed);
        }
    }

    Err(IngestError::InvalidDate {
        line,
        field,
        value: input.to_owned(),
    })
}

fn parse_minor_units(input: &str, field: &'static str, line: usize) -> Result<i64, IngestError> {
    let value = input.trim();
    if value.is_empty() {
        return Err(IngestError::InvalidAmount {
            line,
            field,
            value: input.to_owned(),
        });
    }

    let (negative, magnitude) = if let Some(rest) = value.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = value.strip_prefix('+') {
        (false, rest)
    } else {
        (false, value)
    };
    if magnitude.is_empty() {
        return Err(IngestError::InvalidAmount {
            line,
            field,
            value: input.to_owned(),
        });
    }

    let mut parts = magnitude.split('.');
    let whole = parts.next().unwrap_or_default();
    let fractional = parts.next();

    if parts.next().is_some() {
        return Err(IngestError::InvalidAmount {
            line,
            field,
            value: input.to_owned(),
        });
    }

    let whole_part = if whole.is_empty() {
        0
    } else {
        whole
            .parse::<i64>()
            .map_err(|_| IngestError::InvalidAmount {
                line,
                field,
                value: input.to_owned(),
            })?
    };

    let fractional_minor = match fractional {
        None => 0,
        Some("") => 0,
        Some(f) if f.len() == 1 => {
            let digit = f.parse::<i64>().map_err(|_| IngestError::InvalidAmount {
                line,
                field,
                value: input.to_owned(),
            })?;
            digit * 10
        }
        Some(f) if f.len() == 2 => f.parse::<i64>().map_err(|_| IngestError::InvalidAmount {
            line,
            field,
            value: input.to_owned(),
        })?,
        Some(_) => {
            return Err(IngestError::InvalidAmount {
                line,
                field,
                value: input.to_owned(),
            });
        }
    };

    let total = whole_part
        .checked_mul(100)
        .and_then(|v| v.checked_add(fractional_minor))
        .ok_or_else(|| IngestError::InvalidAmount {
            line,
            field,
            value: input.to_owned(),
        })?;

    Ok(if negative { -total } else { total })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_stripe_settlement_csv_rows() {
        let csv = "payout_id,balance_transaction_id,source_id,available_on,currency,gross,fee,net,type\npo_1,txn_1,ch_1,2026-02-20,usd,100.50,-3.25,97.25,charge\n";

        let rows = parse_stripe_settlement_csv(csv).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].payout_id, "po_1");
        assert_eq!(rows[0].balance_transaction_id, "txn_1");
        assert_eq!(rows[0].gateway_transaction_id, "ch_1");
        assert_eq!(
            rows[0].available_on,
            NaiveDate::from_ymd_opt(2026, 2, 20).unwrap()
        );
        assert_eq!(rows[0].currency, "USD");
        assert_eq!(rows[0].gross_minor, 10050);
        assert_eq!(rows[0].fee_minor, -325);
        assert_eq!(rows[0].net_minor, 9725);
        assert_eq!(rows[0].transaction_type, "charge");
    }

    #[test]
    fn stripe_parser_rejects_invalid_amount() {
        let csv = "payout_id,balance_transaction_id,source_id,available_on,currency,gross,fee,net,type\npo_1,txn_1,ch_1,2026-02-20,USD,10.001,-3.25,6.75,charge\n";

        let err = parse_stripe_settlement_csv(csv).unwrap_err();
        assert_eq!(
            err,
            IngestError::InvalidAmount {
                line: 2,
                field: "gross",
                value: "10.001".to_string(),
            }
        );
    }

    #[test]
    fn parses_bank_statement_csv_rows() {
        let csv = "statement_id,value_date,bank_reference,description,currency,amount\nst_1,2026-02-21,bank_ref_1,Payout transfer,usd,97.25\n";

        let rows = parse_bank_statement_csv(csv).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].statement_id, "st_1");
        assert_eq!(
            rows[0].value_date,
            NaiveDate::from_ymd_opt(2026, 2, 21).unwrap()
        );
        assert_eq!(rows[0].bank_reference, "bank_ref_1");
        assert_eq!(rows[0].description, "Payout transfer");
        assert_eq!(rows[0].currency, "USD");
        assert_eq!(rows[0].amount_minor, 9725);
    }

    #[test]
    fn bank_parser_rejects_missing_required_field() {
        let csv = "statement_id,value_date,bank_reference,description,currency,amount\nst_1,2026-02-21,,Payout transfer,USD,97.25\n";

        let err = parse_bank_statement_csv(csv).unwrap_err();
        assert_eq!(
            err,
            IngestError::MissingField {
                line: 2,
                field: "bank_reference",
            }
        );
    }
}
