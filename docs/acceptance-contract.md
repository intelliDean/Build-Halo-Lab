# Acceptance process contract and bounded proof input

The acceptance gate validates subprocess responses separately from proof
verification. A generic nonzero process exit is not evidence of proof rejection.

## Successful commands

`tests/acceptance.py::validate_completed` requires exit 0, empty stderr, and
exactly one JSON object covering stdout. The reported command must match the
requested command. Both `ipa-demo` and `verify-chain` require `verified` to be the
JSON boolean `true`; missing, false, numeric or string alternatives fail.
`verify-chain` also requires `terminal_accumulator_checks: true`.

For chain proving and verification, the reported statement must match the
independently calculated expected statement, with exact keys, values and types.
Duplicate JSON keys, nonfinite JSON constants and additional stdout are rejected.

## Expected rejections

A negative case requires exit **1**, empty stdout, and the exact caller-specified
stderr diagnostic, including its newline. In particular, proof rejection must
report `error: proof rejected for expected public statement`. The invalid IPA
dimension and oversized-file cases have their own expected diagnostics.

Panics (including Rust exit 101), Unix signal exits, Windows crash statuses,
generic exit-1 errors, and rejection diagnostics with extra panic output fail
the gate. Reports start with `passed: false`; each completed response records
`contract_validated`, and only the completed suite can set `passed: true`.

The original gate classified any nonzero exit as rejection and did not inspect
`verified`; those were defects in the test harness. The historical real proof
artifacts are retained separately, while the revised gate is run again against
fresh real proofs. The gate regression fixtures do not replace cryptographic
proof runs.

## File read boundary

`crates/lab-cli/src/main.rs::read_proof` wraps the opened reader with
`Read::take(128 * 1024 + 1)`. It accepts at most 128 KiB; the extra byte detects
oversize. This bounds the read even when file metadata is unavailable or the
file grows after opening. Oversized inputs are rejected before Ragu application
setup and before decoding. The decoder retains its independent 128 KiB cap.

Rust tests cover empty/small/exact-limit inputs, I/O error propagation, and an
oversized source whose cursor must stop after exactly limit-plus-one bytes.
The real CLI acceptance sequence includes an oversized proof file and expects
the specific limit diagnostic.

## Run the regressions and real gate

```text
cargo test --locked --release -p lab-cli
python -m unittest discover -s tests -p test_acceptance.py -v
python tests/acceptance.py --output evidence/generated
```

CI runs the Python gate regressions before the real acceptance suite. Current
execution results and historical evidence are distinguished in
[the results report](../evidence/RESULTS.md).
