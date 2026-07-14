[← Summary](../core/SUMMARY.md)

# Billing & Cost Explorer

## Important: API Costs

The Cost Explorer API charges **$0.01 per request**. Easy Cloud minimizes this with:
- **2-hour cache per account** — repeated views don't make new API calls
- **Per-account caching** — switching between accounts uses cached data
- **Manual refresh only** — data doesn't auto-refresh

Typical cost: $0.15-0.30/month with normal usage.

## Dashboard Summary

The billing section shows:

| Card | Description |
|------|-------------|
| Current Month | Total spend this month so far |
| Last Month | Final total from previous month |
| Forecast (EOM) | Projected end-of-month cost |

The delta indicator shows % change vs last month:
- 🟢 Green = spending decreased
- 🔴 Red = spending increased

## Cost Over Time Chart

A bar chart showing costs over your selected period:
- **≤ 62 days** → Daily bars
- **> 62 days** → Monthly bars

## Period Filter

Select a time range:
- 7 days
- 1 month (default)
- 3 months
- 6 months
- 1 year
- Custom (month picker)

## Top Services

The sidebar shows your most expensive services with proportional bars.

## Resource Usage

The usage section shows actual consumption per service:

| Column | Description |
|--------|-------------|
| Usage Type | What was consumed (e.g., `TimedStorage-ByteHrs`, `Requests-Tier1`) |
| Usage | Quantity consumed |
| Rate | Effective unit price (cost ÷ usage) |
| Cost | Actual cost |

- **FREE** badge — resource is within free tier (usage > 0, cost = $0)
- Services are sorted by cost (highest first)
- Click a service to expand/collapse its usage details

## Cache Indicator

When cached data is displayed, a clock icon with remaining time appears next to the Refresh button:
```
🕐 1h 45m  [Refresh]
```

Click **Refresh** to force a new API call (bypasses cache).

## All Services Table

A scrollable table listing every service with its cost for the selected period.
