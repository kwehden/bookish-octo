import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 500,
  duration: '30m',
};

const payload = JSON.stringify({
  event_type: 'order.captured.v1',
  tenant_id: 'tenant_1',
  legal_entity_id: 'US_CO_01',
  ledger_book: 'US_GAAP',
  accounting_date: '2026-02-21',
  source_event_id: 'evt-k6',
  posting_run_id: 'run-k6',
  lines: [
    { account_id: '1105', entry_side: 'debit', amount_minor: 10000, currency: 'USD', base_amount_minor: 10000, base_currency: 'USD' },
    { account_id: '4000', entry_side: 'credit', amount_minor: 10000, currency: 'USD', base_amount_minor: 10000, base_currency: 'USD' }
  ],
  provenance: {
    book_policy_id: 'policy_dual_book',
    policy_version: '1.0.0',
    fx_rate_set_id: 'fx_2026_02_21',
    ruleset_version: 'v1',
    workflow_id: 'wf-k6'
  }
});

export default function () {
  const res = http.post('http://localhost:3000/v1/posting/events', payload, {
    headers: {
      'Content-Type': 'application/json',
      'Idempotency-Key': `k6-${__VU}-${__ITER}`,
    },
  });

  check(res, {
    'status is 200': (r) => r.status === 200,
  });

  sleep(1);
}
