> Historical initial results, before the acceptance gate was hardened.
> Its broad nonzero-exit and missing verified-flag checks were defective.
> These are retained raw observations, not the current release gate.
> See [current validated results](RESULTS.md) for the fresh strict run.

# Executed results — 2026-10-20

All required implementation and local acceptance checks passed on the recorded
Windows host. No implementation blocker remains on this tested environment.
Hosted Linux CI, external cryptographic review and learner participation have
not occurred; configuration and review/pilot materials are prepared.

## Actual completed checks

| Check | Actual result | Raw evidence |
| --- | --- | --- |
| Unmodified official Ragu example | 1 passed, 0 failed; 31.38 seconds test execution | [log](ragu-official-example.log) |
| Pinned Rust workspace release tests | 14 passed, 0 failed, 0 ignored: IPA 9, recursive application 5 | [log](workspace-tests.log) |
| Original IPA release tests | 0.23 seconds | [same log](workspace-tests.log) |
| Recursive application release tests | 70.06 seconds, including actual invalid predecessor/base reset proofs and wrong registry parameters | [same log](workspace-tests.log) |
| Separate-process CLI acceptance | 19 completed subprocess checks: 12 successful commands, 7 expected rejections | [full commands, stdout, stderr, timings](measured/acceptance.json) |
| Build / Clippy / formatting | release CLI build passed; Clippy all workspace targets with -D warnings passed; cargo fmt check passed | [build](cli-build.log), [Clippy](clippy.log), [format](format.log) |
| Lesson reference assessment | 8/8 categories passed; all 5 incorrect mutants rejected | [lesson evidence](lesson-assessment.json) |
| Fresh lesson starter | 0/8, exit 1 as expected until learner implements it | [lesson evidence](lesson-assessment.json) |

The official example ran from the unmodified checkout at
`4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5`; `git status --porcelain` was empty
afterward. Earlier parallel compilation attempts were interrupted before any
successful example was claimed. The completed run used one Cargo build job.
Its development dependency proc-macro-error2 emits a future-compatibility warning
on the pinned compiler; it did not fail that example. The workspace Clippy run
has no warnings. [Toolchain details](toolchain.txt), [revision](upstream-revision.txt),
[vendor comparison](ragu-vendor-delta.json), [transport patch](ragu-transport.patch).

## Measured recursive proofs

Release profile, Rust 1.97.1, one sample per case. Initial state is 2.
Expected final states were computed independently in Python modulo Pasta Fp,
not copied from the prover. Each verification below ran in a fresh OS process
with only one compressed proof and three independent expected public inputs.
The terminal IPA evaluation on each of the two curves executes through Ragu's
complete compressed verifier. No history list or predecessor proof list is supplied.

| Steps | Proof bytes | Proving s | Verification s | Prover setup s | Verifier setup s |
| --- | --- | --- | --- | --- | --- |
| 1 | 7,516 | 10.121 | 0.846 | 5.207 | 3.884 |
| 2 | 7,516 | 12.992 | 0.926 | 4.761 | 4.345 |
| 4 | 7,516 | 21.783 | 0.697 | 4.233 | 4.323 |
| 8 | 7,516 | 39.614 | 1.079 | 4.495 | 5.157 |

Artifacts: [1 step](measured/chain-1.bin), [2 steps](measured/chain-2.bin),
[4 steps](measured/chain-4.bin), [8 steps](measured/chain-8.bin).
Proof size includes the complete versioned wire artifact; it excludes the three
expected public inputs and independently rebuilt verifier parameters. Ragu's
fixed-size instance includes immediate child headers; it is not hiding.

Chain setup is `ChainLab::new()` (baked parameters and circuit registries).
Proving includes RNG creation, seed proof, all recursive fuses, terminal
compression and canonical serialization. It excludes file writing and setup.
Verification includes expected-field parsing, strict decoding/re-encoding and
the full terminal verifier. It excludes file reading and setup. The raw JSON
also records whole-process wall times, including startup and setup.

## Measured original IPA

| Vector length | Proof bytes | Proving ms | Verification ms | Setup ms |
| --- | --- | --- | --- | --- |
| 1 | 76 | 0.155 | 0.217 | 0.228 |
| 2 | 140 | 0.768 | 0.605 | 0.314 |
| 4 | 204 | 1.358 | 0.944 | 0.563 |
| 8 | 268 | 3.917 | 1.983 | 1.175 |

IPA setup includes generator derivation, witness creation and the first vector
commitment. The public prover rechecks the witness opening with a second
commitment; that check is included in proving time. IPA verification is timed
after proof decoding and excludes setup. Proof sizes exclude the 64-byte
expected statement and the generator set. These are single observed samples,
with no warm-up, confidence intervals or cross-machine performance claim.

## Hardware and software

* CPU: 11th Gen Intel(R) Core(TM) i5-1145G7 @ 2.60GHz; 8 logical CPUs.
* RAM: 16,844,550,144 bytes (16.84 GB).
* OS/architecture as reported by Python: `Windows-10-10.0.26200-SP0`, `AMD64`.
* Rust: 1.97.1, x86_64-pc-windows-msvc; LLVM 22.1.6; standard release profile.
* Ragu: pinned vendored revision above, ProductionRank R<13>, fixed educational
  registry tags, baked Pasta parameters, multicore enabled. Original IPA:
  curve25519-dalek 4.1.3 and SHA-512 from sha2 0.10.9. Full versions/checksums
  are in Cargo.lock; most Ragu versions are preserved from its tested lockfile.
* Samples ran sequentially after compilation/linting; no other lab build ran
  concurrently with the measured acceptance sequence.

## Reproduce from the workspace root

```text
rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy
cargo build --locked --release -p lab-cli
cargo test --workspace --locked --release
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --release -- -D warnings
cargo test --manifest-path vendor/ragu/Cargo.toml -p ragu_pcd --test nontrivial --locked --release --jobs 1 --features baked,ragu_circuits/insecure-test-registry-tag
python tests/assessment.py --candidate lessons/ipa/solution.py --self-test
python tests/acceptance.py --output evidence/generated
```

The actual local run reused `CARGO_TARGET_DIR` at the sibling scratch checkout's
`work/ragu/target`; acceptance used `--binary ../../work/ragu/target/release/lab-cli.exe
--output evidence/measured` from this workspace. This cache location is not
needed to reproduce: the commands above use the normal target directory.
On Unix use python3 if required. All CLI arguments for every measured subprocess
are retained in the raw JSON. [CLI examples](../README.md).

## Remaining limitations and scope

* Educational, unaudited cryptography. Fixed insecure test registry tags are
  deliberate teaching parameters, not a production setup procedure.
* Neither IPA implementation nor the compressed Ragu demonstration provides
  hiding or zero knowledge here. No deployment or formal-security claim is made.
* Hosted CI is configured with pinned action commits and retains generated
  artifacts, but no remote CI run is claimed. Only Windows execution is observed.
* Technical review and learner-pilot materials are complete; external reviewers
  and learners have not participated. No invented sign-off or participation.
* The original IPA is a new implementation of the selected published
  construction, while curve arithmetic is reused. Ragu supplies the recursive
  cryptography; the local transport adapter does not change its equations.

**The original IPA plus the Ragu-backed demonstration does not constitute an
independently implemented complete Halo system.**
