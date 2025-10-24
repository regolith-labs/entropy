# Entropy

**Entropy** is a provably-fair random number generation protocol for Solana. It uses an N-party commit-reveal scheme to generate random numbers onchain in a secure and cost-effective way. 

## How it works

### Open
Anyone can permissionlessly open a `Var` account to launch a new random variable. Upon opening, the creator should specify the following properties:
- `id`: a unique identifying ID for the variable
- `deposit`: the amount of SOL committers must deposit, to be returned only upon revealing their seed
- `fee_collector`: the account which should collect deposits from unrevealed commits
- `last_commit_at`: the slot after which no new commits are accepted
- `last_reveal_at`: the slot after which no new reveals are accepted
- `close_at`: the slot after which the var can be closed

The variable's digest will automatically be seeded by a keccak hash of the creator's account address, the variable id, and current clock slot.

### Commit
Any user can permissionlessly commit to an open variable to participate in random number generation. To commit, users should generate a random number and store is securely. This value is known as the "seed" and should be kept secret until it is deemed safe to reveal. Safety will depend on the context in which the variable is being used. Users should then hash the seed and submit the hash via a commit instruction. This will open a `Commitment` account to record the committed hash and make a deposit, in SOL, of the value specified by the variable. Since the entropy protocol supports permissionless N-party participation, this deposit acts as a deterence to users who might try to influence the outcome by committing many values and selectively revealing only after other participants have revealed. For all any commitement that is not revealed by the variable's deadline, the deposit will be lost and sent to the variable's fee collector.

### Reveal
To reveal, users must provide their seed. Reveal is a permissionless instruction allowing anyone to call it for any other user, simplifying UI flows for users participating in web applications. The host of the application can program their app to collect seeds at the appropriate time and batch submit them on behalf of users. For trustless setups, users can chose to reveal their seed independently. Upon receiving a seed, the protocol will hash it and verify the result matches the hash provided in the commit step. If so, the reveal will be recorded by XOR'ing the seed value into the existing digest. The deposit will be automatically returned to the committer, and the commmitment account will be closed. 

### Finalization
After all commitments have been revealed or the clock slot has passed the deadline specified by the variable creator, the value can be finalized and used. To finalize, the variable account provides a `finalize()` function which creates a keccak hash of the digest buffer. Note that if no commitments were revealed, the initial digest that the variable was seeded with will be used. Protocols that use Entropy for random number generation should implement thier own independent security checks using the properties on the variable account if, for example, they require a minimum number of commitments or reveal threshold. 

### Close
After a variable has been used, its account and all unrevealed commitments can be cleaned up via the close instruction. All rent will automatically be returned to the accounts which initially openned the accounts and paid for the rent.

        
## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/event.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions.

## Instructions
- [`Open`](program/src/open.rs) – Opens a new var.
- [`Close`](program/src/close.rs) – Closes a var and/or commitment account.
- [`Commit`](program/src/commit.rs) – Commits a hash for secure random number generation.
- [`Reveal`](program/src/reveal.rs) – Reveals a seed and updates the digest.

## State
- [`Commitment`](api/src/state/commitment.rs) – Commitment holds onto a commit hash for a variable.
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
