# Performance Harness

## Sprint 1 baseline
Baseline scenario:
- 500 active users
- 30 minutes duration
- exercises posting API minimal scope

Run:
```bash
k6 run perf/k6_baseline.js
```

## Sprint 2 no-bend comfort gate
No-bend scenario:
- ramp from 500 to 2,000 VUs
- hold 2,000 VUs for sustained window
- enforce error and latency thresholds for comfort gate

Run:
```bash
k6 run perf/k6_2k_no_bend.js
```

Threshold intent:
- `http_req_failed` and `api_failures` stay `<=0.5%`.
- `latency_phase_500` p95 stays `<=300ms`.
- `latency_phase_2k` p95 stays `<=360ms` (20% growth cap vs 500 phase target).
