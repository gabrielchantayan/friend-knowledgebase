## Logging Best Practices

**Core Principle:** Log *what happened to a request*, not what your code is doing.

### Use Wide Events / Canonical Log Lines
- Emit **one comprehensive log event per request per service**
- Include all debugging context in that single event: request info, user context, business data, timing, errors, feature flags
- Stop scattering context across dozens of log lines

### What to Include in Wide Events
```
request_id, trace_id, timestamp
service, version, region
method, path, status_code, duration_ms
user: {id, subscription, account_age, ltv}
business_context: {relevant entities, amounts, states}
error: {type, code, message, retriable}
feature_flags: {enabled flags}
```

### Sampling Strategy (Tail Sampling)
- **100%**: Errors, exceptions, slow requests (>p99)
- **100%**: VIP users, flagged sessions, feature flag rollouts
- **1-5%**: Normal successful requests

### Key Concepts
- **High cardinality** = unique values (user_id) → enables useful queries
- **High dimensionality** = many fields → more questions answerable
- **Structured logging** = JSON format (necessary but not sufficient)

### Anti-Patterns to Avoid
- Multiple log lines per request
- Logging without user/business context
- String-based log searching
- Thinking OpenTelemetry alone solves observability

