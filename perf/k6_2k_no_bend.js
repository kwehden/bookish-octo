import exec from 'k6/execution';
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';

const latency500 = new Trend('latency_phase_500');
const latency2k = new Trend('latency_phase_2k');
const apiFailures = new Rate('api_failures');

export const options = {
  scenarios: {
    no_bend_2k: {
      executor: 'ramping-vus',
      startVUs: 500,
      stages: [
        { duration: '5m', target: 500 },
        { duration: '10m', target: 2000 },
        { duration: '15m', target: 2000 },
        { duration: '5m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    checks: ['rate>=0.995'],
    http_req_failed: ['rate<=0.005'],
    api_failures: ['rate<=0.005'],
    latency_phase_500: ['p(95)<=300'],
    latency_phase_2k: ['p(95)<=360'],
  },
};

const payload = JSON.stringify({
  event_type: 'payment.settled.v1',
  tenant_id: 'tenant_1',
  legal_entity_id: 'US_CO_01',
  ledger_book: 'US_GAAP',
  accounting_date: '2026-02-21',
  source_event_id: 'evt-k6-2k',
  posting_run_id: 'run-k6-2k',
  lines: [
    {
      account_id: '1105',
      entry_side: 'debit',
      amount_minor: 10000,
      currency: 'USD',
      base_amount_minor: 10000,
      base_currency: 'USD',
    },
    {
      account_id: '4000',
      entry_side: 'credit',
      amount_minor: 10000,
      currency: 'USD',
      base_amount_minor: 10000,
      base_currency: 'USD',
    },
  ],
  provenance: {
    book_policy_id: 'policy_dual_book',
    policy_version: '1.0.0',
    fx_rate_set_id: 'fx_2026_02_21',
    ruleset_version: 'v1',
    workflow_id: 'wf-k6-2k',
  },
});

export default function () {
  const response = http.post('http://localhost:3000/v1/posting/events', payload, {
    headers: {
      'Content-Type': 'application/json',
      'Idempotency-Key': `k6-2k-${__VU}-${__ITER}`,
    },
  });

  const ok = check(response, {
    'status is 200': (r) => r.status === 200,
  });

  apiFailures.add(!ok);

  const elapsedMs = exec.instance.currentTestRunDuration;
  if (elapsedMs <= 5 * 60 * 1000) {
    latency500.add(response.timings.duration);
  }
  if (elapsedMs >= 15 * 60 * 1000 && elapsedMs <= 30 * 60 * 1000) {
    latency2k.add(response.timings.duration);
  }

  sleep(1);
}
