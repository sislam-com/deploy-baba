# Zero-Cost Philosophy

Last updated: 2026-05-19

deploy-baba is built on a single principle: **abstractions should cost nothing —
not at runtime, and not on your AWS bill.**

## What Zero-Cost Means in Practice

When you write generic code using deploy-baba's traits, the Rust compiler
generates specialized machine code for each concrete type through
monomorphization. There is no vtable lookup, no heap allocation for trait
objects, no runtime dispatch overhead.

```rust
// This generic function becomes specialized at compile time
// for TomlParser, YamlParser, JsonParser — no runtime cost.
fn load_config<P: ConfigParser<MyConfig>>(source: &str) -> Result<MyConfig, ConfigError> {
    P::parse(source)
}
```

The compiler produces three distinct, optimized functions — one per parser type.
The trait boundary exists only at compile time.

## Why Not `dyn Trait`?

Dynamic dispatch (`Box<dyn ConfigParser>`) adds:
- A heap allocation per trait object
- An indirect function call (vtable) on every method invocation
- Missed optimization opportunities (the compiler can't inline across vtables)

For a deployment automation tool that processes config files and generates specs,
these costs are small in absolute terms. But the zero-cost approach also gives
better error messages (concrete types in stack traces) and eliminates an entire
class of lifetime complexity (`dyn Trait + 'a`).

## Where This Shows Up

Every trait in deploy-baba is designed for static dispatch:

- `ConfigParser<T>` — generic over the config type
- `ApiSpecGenerator` — associated types for Schema and Output
- `SpecFormatConverter<T>` — generic over the target format

The merger (`api-merger`) is the one place where we use enum dispatch
(`UnifiedApiSpec`) instead of generics, because the set of formats is closed
and known at compile time. This is still zero-cost — it compiles to a match
statement, not a vtable.

## Trade-Off

The cost of monomorphization is compile time. Each generic instantiation
produces a new copy of the function in the binary. For a library with 3-5
format implementations, this is negligible. For a library with hundreds of
instantiations, you'd want to measure binary size.

deploy-baba stays well within the negligible range.

## Zero-Cost AWS Infrastructure

The same philosophy extends to infrastructure. Every architectural choice optimizes for the AWS free tier or the cheapest possible alternative ([ADR-002](../plans/adr/ADR-002-sqlite-over-postgresql.md)).

| Component | deploy-baba | Typical alternative | Monthly cost delta |
|-----------|-------------|--------------------|--------------------|
| Database | SQLite on EFS | RDS PostgreSQL (db.t3.micro) | $0 vs ~$15 |
| Compute | Lambda (free tier: 1M req, 400K GB-s) | ECS Fargate / EC2 | $0 vs ~$10+ |
| CDN + hosting | CloudFront + S3 | Vercel / Amplify | Pennies vs $0–20 |
| Scheduler | EventBridge (free tier covers daily backup) | Cron on EC2 | $0 vs ~$5+ |
| Email | SES ($0.10/1K emails) | SendGrid / Mailgun | Pennies vs $15+ |
| Observability | SQLite metrics tables ([ADR-025](../plans/adr/ADR-025-sqlite-metrics-collection.md)) | CloudWatch Metrics | $0 vs $0.30/metric/mo |

At low traffic (a portfolio site), the total monthly AWS bill is under $1 — dominated by EFS storage and Secrets Manager.

## Where We Pay

Not everything is free. These are the non-zero line items:

| Service | Cost | Why we pay |
|---------|------|-----------|
| EFS | ~$0.30/GB/month | SQLite needs a persistent filesystem. Lambda's ephemeral `/tmp` doesn't survive across invocations. |
| Secrets Manager | $0.40/secret/month | Two secrets (PoW key, Anthropic API key). Cheaper than the compliance risk of env vars. |
| CloudFront | Variable (pennies at low traffic) | CDN is the entry point. Data transfer to the internet is the main cost. |
| SES | $0.10/1,000 emails | Contact form acknowledgements. Volume is negligible for a portfolio. |
| S3 | $0.023/GB/month | SPA assets (~5 MB), backups (~3 MB), state file. Under a cent. |

The architecture is designed so that a portfolio site with modest traffic runs within the AWS free tier. Costs only appear if traffic scales — and by then, the revenue justifies the spend.
