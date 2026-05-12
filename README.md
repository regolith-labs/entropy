# Entropy

**Entropy** is a secret-free random number generator for Solana. It reads 32 Pyth price feeds, discretizes each into a single bit using a volatility-adaptive threshold (EWMA), and hashes the resulting 32-bit state to produce a 256-bit random value. No off-chain secrets, no commit-reveal, no trusted infrastructure.

## How It Works

1. Each of 32 Pyth price feeds is tracked with an exponentially weighted moving average (EWMA) of its per-slot variance.
2. On each `sample` call, the program compares each feed's price change against a threshold derived from its historical volatility and the time elapsed.
3. If the price moved beyond the threshold, the feed's bit flips (1 = price went up, 0 = price went down). If not, the bit is unchanged ("sticky").
4. The 32-bit state is hashed with keccak256 to produce the random value.
5. A minimum number of bits must flip per sample (MIN_FLIPS), ensuring sufficient entropy. If too few feeds moved, the sample is rejected.

### EWMA Threshold

The threshold for each feed is:

```
threshold = max(sqrt(variance_per_slot) * sqrt(dt), price * MIN_BPS / 10_000)
```

Where:
- `variance_per_slot` is the EWMA-smoothed variance, updated each sample: `var = lerp(old_var, dp²/dt, alpha)`
- `alpha = min(dt / HALFLIFE, 1.0)` — controls how fast old data decays (~1 min half-life)
- `sqrt(dt)` scales the threshold by elapsed time (prices follow √T random walk scaling)
- `MIN_BPS` floor prevents stale feeds (variance → 0) from becoming trivially flippable

This means the bit only flips when the price moved **more than expected** for the time elapsed. A publisher trying to flip a bit must move the aggregate price beyond what the market naturally does — which requires overpowering 20+ other publishers reporting through Pyth's weighted median.

### Constants

| Parameter | Value | Meaning |
|-----------|-------|---------|
| NUM_FEEDS | 32 | Pyth feeds (crypto, equities, forex, commodities) |
| HALFLIFE | 150 slots | ~1 minute EWMA decay |
| MIN_BPS | 1 | 0.01% minimum threshold floor |
| MIN_FLIPS | 8 | Minimum bits that must change per sample |
| K | 2 | Binary discretization (up/down) |

## Security Analysis

### Threat Model

The adversary is a Pyth publisher (or coalition) who can influence the reported price of one or more feeds. Pyth aggregates prices from 20+ publishers using a stake-weighted median. A single publisher can shift the aggregate by a small bounded amount — enough to move the price by a few dollars, but not enough to overpower the median of honest publishers.

### Why Discretization (K=2) Matters

With raw continuous prices, shifting a price by $0.01 would produce a completely different hash — giving the attacker millions of candidate hashes for free. With K=2 binary discretization plus EWMA thresholds, the attacker must move the price beyond 1 standard deviation of natural volatility to flip a single bit. This is the difference between **free manipulation** and **costly manipulation**.

With K=2 and m controlled feeds, the attacker's search space is exactly **2^m** candidate hashes.

| Feeds controlled (m) | Candidate hashes (C) | P(picking 1-of-25 outcome) |
|----------------------|----------------------|----------------------------|
| 1 | 2 | 7.8% |
| 2 | 4 | 15.1% |
| 3 | 8 | 27.5% |
| 4 | 16 | 47.5% |
| 5 | 32 | 72.4% |
| 6 | 64 | 92.5% |

An attacker needs to control **6 feeds across different asset classes** to reliably pick a 1-in-25 outcome. With the current feed set spanning crypto, equities, forex, and commodities, this requires compromising 6 independent Pyth publishers — an extremely high bar.

### Effective P: Ticket Share Matters, Not Ticket Count

For applications where users hold varying shares of tickets (e.g., mining), the raw ticket count is irrelevant. What matters is the attacker's **fractional share** `s`:

```
P(attacker wins) = 1 - (1 - s)^C
Advantage = [1 - (1-s)^C] / s
```

| Attacker share (s) | Effective P (1/s) | Honest win rate | Win rate (C=2) | Advantage |
|--------------------|-------------------|----------------|----------------|-----------|
| 50% | 2 | 50.0% | 75.0% | 1.5x |
| 20% | 5 | 20.0% | 36.0% | 1.8x |
| 10% | 10 | 10.0% | 19.0% | 1.9x |
| 5% | 20 | 5.0% | 9.8% | 1.95x |
| 1% | 100 | 1.0% | 2.0% | 2.0x |
| 0.1% | 1,000 | 0.1% | 0.2% | 2.0x |

**Key insight:** With C=2 (one controlled feed), the worst-case advantage asymptotes to 2x. An attacker with 10% of tickets goes from 10% to 19% — meaningful but bounded. The advantage is independent of how many total tickets exist.

### Security Guidelines

Use this table to determine if Entropy is suitable for your application:

| Use case | Effective P | C=2 attacker win rate | Acceptable? |
|----------|------------|----------------------|-------------|
| 1-of-25 selection (uniform) | 25 | 7.8% (vs 4% honest) | Yes — 2x edge is economically irrational for Pyth publishers |
| 1-of-625 rare event | 625 | 0.32% (vs 0.16%) | Yes — negligible advantage |
| Ticket lottery (10% holder) | 10 | 19% (vs 10%) | Depends on stakes — 2x advantage compounds over time |
| Ticket lottery (1% holder) | 100 | 2.0% (vs 1%) | Yes — advantage is small in absolute terms |
| Binary coin flip | 2 | 75% (vs 50%) | No — too few outcomes for C=2 to be safe |
| High-value single-winner | 1/s | ≤ 2x honest rate | Only if stakes are low enough that 2x edge isn't worth a publisher's reputation |

**Not suitable for:** Binary outcomes (coin flips), or any application where a 2x advantage is catastrophic. **Suitable for:** Multi-outcome selections (≥ 25 outcomes) and ticket-weighted lotteries where the 2x worst-case advantage is an acceptable tradeoff for eliminating centralized secrets.

### Timing Attack Mitigation

Entropy's `sample` instruction is permissionless — anyone can call it. This creates a potential timing attack where a caller monitors prices and waits for a favorable slot. This is mitigated by the consuming protocol:

- The consuming contract validates `var.sample_at == current_slot`, requiring the sample and finalization to land in the same slot (Jito bundle).
- Multiple miners race to finalize, so the first valid bundle wins — the attacker can't delay without risking someone else finalizing first.
- The slot is **not** included in the hash input, so the Solana leader cannot gain advantage by choosing which slot to process.

### Pyth Publisher Economics

Pyth publishers include Coinbase, Wintermute, Virtu Financial, Cboe, and ~120 other institutions. Oracle Integrity Staking puts publisher capital at risk for inaccurate data. The economic question for any attack:

```
Expected gain: (manipulated_rate - honest_rate) × reward_value
Expected cost: reputation_damage + staking_slash_risk + legal_exposure
```

For established publishers, the cost vastly exceeds any plausible gain from manipulating a mining lottery. The residual risk is from small/marginal publishers or bribery — bounded by the 2x structural cap.

## Account Layout

```rust
struct Var {
    value: [u8; 32],             // keccak hash — the random value
    sample_at: u64,              // slot of last sample
    bits: u32,                   // current bit state (1 per feed)
    prices: [i64; 32],           // prices from last sample
    variances: [Numeric; 32],    // EWMA variance-per-slot per feed
}
```

Total account size: 820 bytes (including discriminator).

## API
- [`Consts`](api/src/consts.rs) – Feed addresses, tickers, and algorithm parameters.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Instruction`](api/src/instruction.rs) – Declared instructions.
- [`SDK`](api/src/sdk.rs) – Instruction builders.

## Instructions
- [`Init`](program/src/init.rs) – Initializes the var account.
- [`Sample`](program/src/sample.rs) – Reads 32 Pyth feeds, applies EWMA thresholds, updates bits and hash.

## State
- [`Var`](api/src/state/var.rs) – Stores the random value, bit state, prices, and variance estimates.

## Usage

```sh
# Build
cargo build-sbf

# CLI
KEYPAIR=~/.config/solana/id.json RPC=<rpc_url> cargo run -p entropy-cli <init|sample|var|lut>
```
