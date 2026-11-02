# Build Halo Lab

A Rust educational MVP with a genuine Ragu recursive computation, an original
implementation of a published inner-product argument, and one assessed lesson.
This is educational software, **not production cryptography**. Neither proof
implementation provides hiding or zero knowledge in this lab.

**The original IPA plus the Ragu-backed demonstration does not constitute an
independently implemented complete Halo system.**

## Reproduce

Run these commands from this directory with Rustup and Python 3.11+ installed:

```text
rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy
cargo build --locked --release -p lab-cli
cargo test --workspace --locked --release
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked --release -- -D warnings
cargo test --manifest-path vendor/ragu/Cargo.toml -p ragu_pcd --test nontrivial --locked --release --jobs 1 --features baked,ragu_circuits/insecure-test-registry-tag
python tests/assessment.py --candidate lessons/ipa/solution.py --self-test
python -m unittest discover -s tests -p test_acceptance.py -v
python tests/acceptance.py --output evidence/generated
```

On Unix, use `python3` if `python` is unavailable. The acceptance script chooses
`target/release/lab-cli.exe` on Windows and `target/release/lab-cli` elsewhere;
use `--binary PATH` to override. It generates and verifies real 1, 2, 4 and 8 step
chains, with each verifier in a new process. Measurements include hardware,
proof bytes, setup time, proving time including terminal compression, and terminal
verification time. See [recorded results](evidence/RESULTS.md) for actual runs and
limitations; CI configuration alone is not evidence of a completed CI run.

```text
cargo run --locked --release -p lab-cli -- ipa-demo --length 8
cargo run --locked --release -p lab-cli -- evaluate --initial 2 --steps 2
cargo run --locked --release -p lab-cli -- prove-chain --initial 2 --steps 2 --out chain.bin
cargo run --locked --release -p lab-cli -- verify-chain --initial 2 --final 52 --steps 2 --proof chain.bin
```

The last command supplies the expected statement independently: starting at 2,
the first two states are 7 and 52. Wrong initial/final/count must fail. Hex fields
are `0x` followed by exactly 32 canonical **little-endian bytes**; decimal accepts
u64. Larger field values are printed in that hex format. Arithmetic is modulo
Pasta Fp, not integer arithmetic over an unbounded domain.

The verifier receives only the final compressed proof and the caller's expected
initial value, final value and count. The artifact contains no history list or
predecessor proof list. Its fixed-size Ragu instance includes the immediate
child headers needed by the terminal protocol; this is not a hiding proof.
Ragu's recursive `fuse` accumulates predecessor validity;
its compressed terminal verifier completes the checks on both curves. The
independent IPA is not plugged into Ragu's engine.

The acceptance gate requires explicit JSON `verified: true` on successful
verification and matches expected rejections by exit code **and exact diagnostic**.
Crashes and unrelated errors fail the gate. The CLI reads at most 128 KiB plus
one byte before decoding, rejecting oversized proof files before setup. See the
[acceptance contract](docs/acceptance-contract.md) and its regression tests.

## Learn and inspect

* [Complete IPA lesson](lessons/ipa/README.md): prerequisites, explanation,
  starter, automated assessment, worked solution, Halo connection.
* [IPA construction](docs/ipa-construction.md): published protocol, assumptions,
  exact generator derivation, transcript and canonical encoding.
* [Pinned Ragu APIs and fork](docs/ragu-api.md): upstream reproduction, application
  constraints, codec changes and final accumulator checks.
* [Attribution](docs/attribution.md): original application/proof code versus reused
  arithmetic and recursive cryptography; upstream licenses are retained.
* [Technical review checklist](docs/review-checklist.md) and
  [learner pilot kit](docs/pilot.md): prepared materials, no claimed outside review.
* [Acceptance checklist](docs/acceptance.md) and [evidence](evidence/RESULTS.md).

The workspace contains `edu-ipa`, `recursive-demo`, and `lab-cli`; supplementary
tests, lessons, documentation and evidence live in their named directories.
Direct dependencies are version-pinned and transitive versions/checksums are in
`Cargo.lock`. Ragu is vendored at commit
`4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5` with an attributed transport patch;
no mock proof feature is enabled. The lab's code is MIT licensed. Ragu retains
its own MIT OR Apache-2.0 licensing and copyrights.
