# Entropy [WIP]

## ⚠️ CAUTION This code is a work-in-progress and not ready for production. Please do not use or rely on it in any way.

**Entropy** is a provably-fair random number generation protocol for Solana. It uses an commit-reveal scheme paired with slothash sampling strategy to generate random numbers onchain in a secure and cost-effective way.

## How it works

Users create a new variables by pinging the Entropy API offchain and receiving back a signed transaction from the provider to open a `Var` account. Users must sign this transaction and submit it to the chain to open the account. On the backend, the Entropy API will generate a set of N random numbers by recursively hashing a psuedorandom number in a loop and returning the last value in the set. The number N will represent the number of values the variable will take on over its lifetime.

The variable will be initialized with the commit provided by the Entropy API and the ending slot provided by the user. When that slot comes due, users should call `Sample` to sample the slothash from the chain and record it on the variable. Only after the slothash has been sampled, the Entropy API will make the seed value available via the API. Users can fetch this value and then submit it via the `Reveal` instruction to create the finalized value. If the variable is initialzied with `is_auto = true`, then the Entropy API will automatically sample the slothash and reveal the seed. The variable can then be read by any onchain program via `value` property on the variable account.

The variable will then wait for the user to call `Next` to reset the variable for its next value. The slothash and finalized value will be reset to zero, and the recorded seed from the last value will become the commit for the next value. In this way, the Entropy API is "locked in" to all future values and cannot selectively manipulate specific outcomes. Likewise, the slothosh sampled at the ending slot is unknown to the Entropy API at the time of openning the variable, thus the Entropy service provided cannot know the results of future outcomes. Since the Entropy provider keeps future seed values secret until reveal, the validators who provide slothashes cannot favorably manipulate the outcome of the result either.


## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/event.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions.

## Instructions
- [`Open`](program/src/open.rs) – Opens a new variable.
- [`Close`](program/src/close.rs) – Closes a variable account.
- [`Next`](program/src/next.rs) - Moves a variable to the next value.
- [`Reveal`](program/src/reveal.rs) – Reveals a seed.
- [`Sample`](program/src/sample.rs) - Samples the slothash.

## State
- [`Variable`](api/src/state/variable.rs) – Variable tracks a unique random variable.

## Get started

Compile your program:
```sh
steel build
```

Run unit and integration tests:
```sh
steel test
```
