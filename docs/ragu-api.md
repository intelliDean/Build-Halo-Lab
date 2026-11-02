# Ragu dependency and actual API notes

Inspected official source: <https://github.com/tachyon-zcash/ragu>, commit
`4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5`. The vendored source is pinned to
that commit, with the transport-only modification described below. The real
`ragu_pcd` crate is used. The repository root `ragu` crate's opt-in `mock`
feature is **not** used.

Upstream pins Rust **1.97.1**, edition 2024, MSRV 1.97. Its workspace crate
versions are 0.0.0. Its Getting Started book currently shows version 0.1
and an older `.carry()` call after `seed`; those snippets are not the exact
current API. This lab uses the inspected source signatures:

```rust
ApplicationBuilder::<Pasta, ProductionRank, 4>::new()
    .register(Seed)?
    .register(Advance)?
    .finalize(ragu_pcd::pasta::baked())?;
app.seed(&mut rng, Seed, witness)?; // -> (Pcd<...>, Aux)
app.fuse(&mut rng, Advance, witness, left_pcd, right_pcd)?;
app.compress(&pcd, &mut rng)?;
app.verify_compressed(&compressed_pcd)?; // terminal decider
```

Sources: [PCD API](https://github.com/tachyon-zcash/ragu/blob/4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5/crates/ragu_pcd/src/lib.rs),
[fusion](https://github.com/tachyon-zcash/ragu/blob/4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5/crates/ragu_pcd/src/fuse/mod.rs),
[compression](https://github.com/tachyon-zcash/ragu/blob/4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5/crates/ragu_pcd/src/compress/prover.rs),
[book](https://github.com/tachyon-zcash/ragu/blob/4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5/book/src/guide/getting_started.md).

## Reproduce the existing official example

From this workspace root, the following runs the unchanged official
`various_merging_operations` test. It constructs two Poseidon leaf proofs,
recursively fuses them, and verifies the result. This is an existing upstream
example/test, not lab-authored evidence.

```sh
cargo +1.97.1 test --manifest-path vendor/ragu/Cargo.toml \
  -p ragu_pcd --test nontrivial --release --jobs 1 \
  --features baked,ragu_circuits/insecure-test-registry-tag
```

For independent comparison against unmodified source:

```sh
git clone https://github.com/tachyon-zcash/ragu.git ragu-official
git -C ragu-official checkout 4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5
cargo +1.97.1 test --manifest-path ragu-official/Cargo.toml \
  -p ragu_pcd --test nontrivial --release --jobs 1 \
  --features baked,ragu_circuits/insecure-test-registry-tag
```

See the evidence directory and reproduction report for actual completed runs.
An interrupted compilation is not a reproduced example.

## What the recursive circuit enforces

Field: `ragu_core::pasta::Fp`, Pallas base field, modulus
`0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001`.
`ProductionRank = R<13>` gives 8192 polynomial coefficients and up to 2048
gates. The header has three data fields `(initial,current,count)` plus a
Ragu type suffix, for `HEADER_SIZE = 4`.

`Seed`, application index 0, consumes unit children via `app.seed` and
constrains `current = initial` and `count = 0`. `Advance`, index 1, consumes
`ChainState` children and constrains the output to
`(left.initial, left.current² + 3, left.count + 1)`. The product constraint
`product(count-i, i=1..8) = 0` enforces the bound in the field. These are
arithmetic proof constraints. The CLI's additional step checks improve error
messages; they do not carry the soundness argument.

Ragu has binary PCD fusion. The honest chain prover supplies the same
predecessor in both child slots. It uses the left header for the recurrence,
while Ragu folds **both** child validity claims. This is a shared predecessor
DAG and genuine recursive computation; the verifier receives one terminal
compressed proof, never a history or a list of independently checked proofs.
The right header is authenticated by Ragu, although the application makes no
additional transition claim about it. The induction argument follows the left
child down to `Seed`; each left count decreases by one, and a valid seed
starts at zero and preserves its initial state.

Ragu's own private bootstrap is the only cryptographic base case and outputs
unit data. Application `Seed` is an ordinary recursive fuse against it. The
application's `ChainState` suffix cannot be substituted for the bootstrap's
unit suffix. No application code exposes Ragu's private Dummy header.

## Final accumulator checks

`ChainLab::verify` calls the complete `Application::verify_compressed`, using
the independently supplied initial, final, and count as the output header.
It does not accept a purported statement from inside the artifact.
Upstream checks the native and nested instance, replays fuse challenges,
checks stage commitments, reduces all revdot/wire-binding claims to openings,
and checks each batch. On **each** curve,
`compress/batch/verifier.rs::verify_openings` calls
`ipa::verify_proof(...).use_challenges().eval::<B>()`. The MSM evaluation
executes the final generator/accumulator check. The lab never stops at an
IPA guard or unchecked accumulator.

Sources: [terminal verifier](https://github.com/tachyon-zcash/ragu/blob/4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5/crates/ragu_pcd/src/compress/verifier.rs),
[batch verifier](https://github.com/tachyon-zcash/ragu/blob/4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5/crates/ragu_pcd/src/compress/batch/verifier.rs),
[IPA guard](https://github.com/tachyon-zcash/ragu/blob/4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5/crates/ragu_pcd/src/ipa/verifier.rs).

## Exact local fork

Upstream has no public serialization API for either `Proof` or
`CompressedProof`, and their fields are private. Reconstructing one externally
would require invented APIs or unsafe memory layout. The lab therefore adds
`crates/ragu_pcd/src/compress/codec.rs` and one `mod codec;` declaration to
`compress/mod.rs`. This adapter exposes `to_canonical_bytes()` and
`from_canonical_bytes::<R,H>()`. It implements transport only and changes no
proving, transcript, recursive-circuit, or verification equations. It is lab
work and is not an upstream-reviewed API. The original Ragu licenses and
copyright files are retained. Windows-incompatible documentation symlinks
`crates/*/katex-header.html` are materialized with their upstream target's
contents.

Format `BHLRAG01`: 8-byte magic; little-endian u32 rank and header width;
circuit index; explicit length-prefixed header fields and component
commitments; all remaining instance fields in declaration order; native and
nested reduction/batch/IPA messages. Integers and lengths are u32 LE; fields
use Udon's canonical 32-byte LE integers; curve points use Udon's canonical
32-byte compressed encodings. The decoder fixes every vector length from
`R`, `H`, and the pinned component enumerations before allocation; rejects
noncanonical fields/points, dimension mismatches, trailing data, truncation,
and inputs larger than 128 KiB; and requires byte-for-byte re-encoding.
This format is deliberately tied to this commit, not a promise of upstream
or cross-version compatibility. The artifact contains no separately attached output statement. Ragu's terminal instance does include its immediate child headers, as required by its protocol; these reveal the preceding state. It contains no full intermediate history and no list of earlier proofs. This is not a claim of hiding.

## Educational limitations

The application enables upstream's `insecure-test-registry-tag`: fixed test
setup tags, not a production setup ceremony. The baked curve generators and
Poseidon parameters, recursion, compressor, cryptographic IPA and final
decider are reused Ragu code. The lab's original `edu-ipa` is separate and is
not substituted into Ragu. Ragu's current compressed proof is not hiding,
and its arithmetic is variable-time. This is educational software, not
production cryptography. The original IPA plus this Ragu-backed demonstration
**does not constitute an independently implemented complete Halo system**.

