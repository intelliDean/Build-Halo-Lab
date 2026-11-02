# Educational inner-product argument

Construction selected before coding: Bünz, Bootle, Boneh, Poelstra, Wuille and
Maxwell, *Bulletproofs: Short Proofs for Confidential Transactions and More*,
[ePrint 2017/1066](https://eprint.iacr.org/2017/1066), §3, relation (2), Protocols
1 and 2. The [author-hosted PDF](https://web.stanford.edu/~buenz/pubs/bulletproofs.pdf)
was read on 2026-10-20; the direct ePrint PDF returned HTTP 403 in this environment.
Protocol 1 is on printed page 15, Protocol 2 on printed page 16. This code is an
original implementation of that published algorithm, not a new cryptographic
construction. No Bulletproofs proof library is used.

## Relation and implementation choices

For scalar vectors `a,b` of power-of-two dimension `n`, the expected statement is
the Ristretto commitment `C = sum(a_i G_i + b_i H_i)` and scalar `c = sum(a_i b_i)`.
Protocol 1 derives a nonzero challenge `y` after binding that statement and the
parameters, sets `Q=yU`, and starts from `P=C+cQ`. Protocol 2 folds vectors and
generators until a single final group equation can be checked. Omitting `y`
would omit a step of the selected explicit-inner-product construction.

The implementation uses additive notation. For each split into left/right halves:

```text
L = <aL,GR> + <bR,HL> + <aL,bR> Q
R = <aR,GL> + <bL,HR> + <aR,bL> Q
x = transcript challenge after L,R; z = inverse(x)
a' = x aL + z aR             b' = z bL + x bR
G' = z GL + x GR             H' = x HL + z HR
P' = x² L + P + z² R
final check: P = a G + b H + (a b) Q
```

The verifier deliberately performs the same explicit generator folds as the
prover. It does not implement the paper's faster multi-exponentiation verifier.
Verifier work is linear in `n`; proof payload grows logarithmically. Accepted
dimensions are `1,2,4,...,4096`; `n=1` is a real base case with no round messages.

## Original work and reused arithmetic

`edu-ipa` owns its commitment logic, generator derivation, framed SHA-512
transcript, prover, verifier and byte codec. Exact dependency
`curve25519-dalek = 4.1.3` supplies established Ristretto255 point/scalar
arithmetic, canonical compression and uniform-byte-to-point mapping;
`sha2 = 0.10.9` supplies SHA-512. These arithmetic and hash primitives are reused,
not independently implemented or audited by this project.

The proof crate is separate from `recursive-demo`. Ragu does not call this IPA
implementation. Its role is an independently implemented educational proof
component beside a Ragu-backed recursive demonstration. Together they do not
constitute an independently implemented complete Halo system.

## Generator and transcript specification

Every hash message is framed as `u64_le(label_length) || label ||
u64_le(value_length) || value`. Integers within values are `u64_le` unless
otherwise specified below. Byte strings are case sensitive and contain no
implicit terminators. SHA-512 output is used in its standard byte order.

For each generator, initialize SHA-512 and append frames in this order:

| Label | Value |
| --- | --- |
| `domain` | `build-halo-lab/ipa/generators/v1` |
| `context` | caller's context bytes |
| `n` | vector dimension |
| `role` | `G`, `H`, or `U` |
| `index` | zero-based index; zero for U |
| `counter` | starts at zero |

Map its 64-byte digest with `RistrettoPoint::from_uniform_bytes`. Reject identity
and any prior point, incrementing the counter on rejection. Derive all G points,
then all H points, then U. This is a hash-to-group construction; computing a
scalar hash and multiplying a fixed base would disclose generator relations
and is intentionally not used. Random-oracle modeling is assumed for this
derivation. This is not a claim of compliance with an RFC 9380 ciphersuite.

The parameter ID is the first 32 bytes of SHA-512 over frames: `domain` =
`build-halo-lab/ipa/parameter-id/v1`, `n`, `context`, followed by a `point` frame
for every compressed generator in derivation order. The default context is
`Build Halo Lab educational IPA v1`. The verifier derives its expected parameters
independently; the proof cannot select an attacker-chosen parameter set.

The proof transcript starts with frames: `domain` =
`build-halo-lab/ipa/transcript/v1`, `parameters` = parameter ID, `n`,
`commitment` = canonical C bytes, and `inner-product` = canonical c bytes.
Obtain y using challenge label `inner-product-binding`. Each round appends
`round` = zero-based round number, `L`, and `R`, then obtains x using label
`fold`. Point values are compressed canonical Ristretto bytes.

A challenge hashes a clone of the transcript after appending frames
`challenge-label` and `challenge-counter` (starting at zero). Reduce the full
64-byte digest with `Scalar::from_bytes_mod_order_wide`; retry if zero. On
acceptance append the same label/counter and a `challenge-value` frame containing
the canonical scalar to the live transcript. The reduction has negligible bias;
this document does not claim exact uniform sampling. No challenge arrives from
the proof file. Modifying C, c, parameters, round order, L or R changes challenges.

## Canonical encoding

The expected statement is exactly 64 bytes: compressed Ristretto C (32 bytes),
then canonical little-endian scalar c (32 bytes). The versioned proof is:

```text
ASCII "BHIPA001" (8 bytes)
n (u32 little endian, 4 bytes)
L_0,R_0,...,L_(log2(n)-1),R_(log2(n)-1) (32 bytes per point)
final a, final b (32 bytes per canonical little-endian scalar)
```

Proof size is exactly `76 + 64*log2(n)` bytes. Round count is determined by n;
there is no attacker-controlled allocation count. Decoding rejects out-of-bound
dimensions, invalid Ristretto encodings, noncanonical scalars, short/trailing
data and unknown headers. Statement bytes have no embedded parameters: the
caller must supply the independently expected dimension, context and statement.

## Assumptions and limits

Binding relies on the hardness of finding discrete-log relations among the
derived generators. The noninteractive argument additionally relies on the
Fiat–Shamir random-oracle heuristic, unambiguous transcript binding, and correct
group/hash implementations. Hash-based parameter IDs additionally rely on
collision resistance. This educational adaptation has not received external
cryptographic review and no security theorem is claimed for its exact encoding.

**There is no hiding and no zero knowledge.** Commitments are deterministic,
there is no blinding scalar, and final folded witness values are public. At n=1
the witness is revealed in full. Never enter secret data. Memory zeroization,
side-channel hardening, optimized verification, streaming codecs, a formal
security proof and deployment-grade denial-of-service analysis are outside this
MVP. Input bounds make local tests predictable; they do not establish production
safety.

## API and reproduction

```rust
use edu_ipa::{Parameters, Scalar, prove, verify, Proof};
fn main() -> Result<(), edu_ipa::Error> {
    let pp = Parameters::new(2)?;
    let a = [Scalar::from(1u64), Scalar::from(2u64)];
    let b = [Scalar::from(3u64), Scalar::from(4u64)];
    let expected = pp.commit(&a, &b)?;
    let proof = prove(&pp, &expected, &a, &b)?;
    let received = Proof::from_bytes(&proof.to_bytes())?;
    verify(&pp, &expected, &received)?;
    Ok(())
}
```

From the workspace root run `cargo test --locked -p edu-ipa` and
`cargo run --locked --release -p lab-cli -- ipa-demo --length 8`. See the top-level
evidence for commands actually executed and measured timings. Deriving parameters
and committing are separate operations from proof generation; benchmark scopes
must disclose whether they include them.

The test-only `prove_inner` bypass exercises malicious witnesses against false
public commitments/products. These tests reach the proof verifier's algebraic
equation; honest-prover prechecks are not their only defense. Other unit tests
cover n=1/2/4/8/64, zero witnesses, wrong dimensions/contexts, modified round
messages/final scalars and strict malformed encoding rejection.
