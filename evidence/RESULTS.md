# Current validated results — 2026-10-20

The acceptance harness defects are fixed: expected rejections require exit 1,
empty stdout and the exact named diagnostic. Successful verification requires
the JSON boolean `verified: true`, matching command and independently expected
statement; chain verification also requires `terminal_accumulator_checks: true`.
Crashes, unrelated errors, missing/false flags and ambiguous JSON fail the gate.
[Exact contract and regression coverage](../docs/acceptance-contract.md).

The CLI now bounds the proof read to 128 KiB plus one detection byte, before
application setup or decoding. The transport decoder retains its own size limit.
The recursive circuit, original IPA and reused Ragu cryptography are unchanged.

## Checks actually run after the fixes

| Check | Result | Evidence |
| --- | --- | --- |
| Rust workspace, release | 17 passed, 0 failed, 0 ignored: 9 IPA, 5 recursive, 3 CLI reader | [log](workspace-tests-followup.log) |
| Gate regression suite | 13 passed, including synthetic crash/false-flag cases | [log](acceptance-validator-tests.log) |
| Strict real-process gate | 20 passed: 12 successful commands and 8 exact expected rejections | [schema-v2 report](hardened/acceptance.json), [log](acceptance-hardened.log) |
| Reader boundaries | Empty/small/exact-limit accepted; oversized cursor stops at limit+1; I/O errors propagated | [log](cli-reader-tests.log) |
| Release executable | Build passed | [log](cli-build-followup.log) |
| Clippy, all workspace targets | Passed with -D warnings | [log](clippy-followup.log) |
| Formatting | cargo fmt --all --check passed | [empty clean log](format-followup.log) |

The new real-process suite generates fresh 1-, 2-, 4- and 8-step proofs and
verifies each in a separate process without a history list. It repeats statement
and proof mutations and adds an oversized-file rejection. Every saved response
has `contract_validated: true`; the suite report sets `passed: true` only after
all cases finish. Unit fixtures test the gate, rather than substitute for proofs.

Previously executed, unchanged checks remain recorded: the official unmodified
Ragu example passed (1 test, 31.38 seconds), and the lesson solution passed 8/8
with five incorrect mutants rejected and the fresh starter failing as expected.
[Official log](ragu-official-example.log), [lesson evidence](lesson-assessment.json).
The [initial results and benchmarks](results-initial.md) are retained as
historical observations; their older harness is not used as the current gate.

## Fresh measurements from the hardened gate

Rust 1.97.1, standard release profile; one observed sample per case, setup
excluded from proving/verification columns. Initial state 2; expected final
states independently calculated in Python modulo Pasta Fp. All terminal opening
checks execute on both curves. Proof bytes exclude the separately supplied
expected public statement and verifier parameters.

| Steps | Proof bytes | Proving s | Verification s | Prover setup s | Verifier setup s |
| --- | --- | --- | --- | --- | --- |
| 1 | 7,516 | 10.104 | 0.793 | 4.777 | 4.711 |
| 2 | 7,516 | 17.050 | 1.031 | 5.809 | 7.338 |
| 4 | 7,516 | 23.296 | 0.839 | 6.142 | 6.656 |
| 8 | 7,516 | 33.377 | 0.790 | 6.628 | 6.198 |

Artifacts: [1 step](hardened/chain-1.bin), [2 steps](hardened/chain-2.bin),
[4 steps](hardened/chain-4.bin), [8 steps](hardened/chain-8.bin).

| IPA vector length | Proof bytes | Proving ms | Verification ms | Setup ms |
| --- | --- | --- | --- | --- |
| 1 | 76 | 0.144 | 0.192 | 0.237 |
| 2 | 140 | 0.595 | 0.489 | 0.327 |
| 4 | 204 | 1.437 | 0.871 | 0.544 |
| 8 | 268 | 2.964 | 1.480 | 1.431 |

Chain setup builds the Ragu application/registries. Chain proving includes seed,
all fuses, terminal compression and canonical serialization, excluding file
writing. Chain verification includes expected-field parsing, decoding/re-encoding
and the complete terminal verifier, excluding bounded file reading. The raw JSON
also contains process wall times including setup and I/O. IPA setup includes
generators, witness creation and the first commitment; proving includes the
second witness-opening commitment check. IPA verification is timed after decode.
No confidence intervals or cross-machine performance claim is made.

Hardware: 11th Gen Intel(R) Core(TM) i5-1145G7 @ 2.60GHz, 8 logical CPUs,
16,844,550,144 RAM bytes; `Windows-10-10.0.26200-SP0`, `AMD64`. Samples ran sequentially
after builds/linting completed. [Compiler/LLVM details](toolchain.txt).

## Reproduce

From the workspace root, with the pinned toolchain installed:

```text
cargo build --locked --release -p lab-cli
cargo test --workspace --locked --release
cargo clippy --workspace --all-targets --locked --release -- -D warnings
cargo fmt --all --check
python -m unittest discover -s tests -p test_acceptance.py -v
python tests/acceptance.py --output evidence/generated
python tests/assessment.py --candidate lessons/ipa/solution.py --self-test
```

On Unix use python3 if required. The local run reused
`CARGO_TARGET_DIR=../../work/ragu/target` and invoked acceptance with
`--binary ../../work/ragu/target/release/lab-cli.exe --output evidence/hardened`.
Normal reproduction needs no shared cache. The report preserves every actual
subprocess command/stdout/stderr and exact rejection expectation.

## Remaining limitations

No local implementation blocker remains. Hosted Linux CI has not been executed;
it now runs gate regressions before real acceptance. External cryptographic review
and learner participation have not occurred; the review checklist/pilot kit are
prepared. The toolchain, dependencies and Ragu revision remain pinned; Ragu uses
fixed insecure educational registry tags and a documented transport adapter.
[Source revision](upstream-revision.txt), [vendor delta](ragu-vendor-delta.json),
[transport patch](ragu-transport.patch), [attribution](../docs/attribution.md).

Neither proof implementation provides hiding or zero knowledge here. This is
educational software, not production cryptography. **The original IPA plus the
Ragu-backed demonstration does not constitute an independently implemented
complete Halo system.**
