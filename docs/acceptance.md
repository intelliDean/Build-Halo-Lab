# Acceptance checklist

Execution status and measurements are recorded in `evidence/RESULTS.md` and its
linked raw evidence. The checklist describes the required acceptance surface;
an implemented check is not a claim that an unrecorded CI run occurred.

| Requirement | Implementation / evidence |
| --- | --- |
| Official dependency inspected and pinned | `docs/ragu-api.md`, vendored commit, `Cargo.lock`, `rust-toolchain.toml` |
| Existing Ragu example reproduced | Unchanged upstream `nontrivial::various_merging_operations`, raw baseline log |
| Three Rust workspace crates | `edu-ipa`, `recursive-demo`, `lab-cli` |
| Genuine recursion | Ragu `seed` then `fuse` with accumulated predecessor validity, no proof list |
| Public initial/final/count | ChainState output header, caller-supplied `Statement` in verifier |
| Base case, continuity, count bound | Seed equality/count-zero and Advance recurrence/count/product constraints |
| 1, 2, 4, 8 steps | `tests/acceptance.py`; independently calculated Fp expected results |
| Separate final verifier process | Each acceptance verification invokes a fresh CLI process with only compressed bytes and expected statement |
| Final accumulator checks | Ragu terminal verifier, both `use_challenges().eval` calls, documented source links |
| Original educational IPA | Published Bulletproofs §3 Protocols 1–2, `edu-ipa`; no IPA library reused |
| Arithmetic reused and attributed | Dalek Ristretto/scalars, SHA-512; Ragu's Pasta/proof stack; license files retained |
| Generators, transcript, encoding | Independently hash-derived generators, framed domains, canonical bounded codecs |
| Non-hiding status explicit | Crate docs, construction notes, README, lesson; no ZK claim |
| Altered public statements | IPA malicious-prover bypass tests; Ragu tests; separate-process acceptance mismatches |
| Invalid transitions | Simulator executes the actual constraint helpers and real malicious transition proof test |
| Invalid predecessor | Relabel real PCD, generate locally consistent parent, reject uncompressed and compressed; reject next generation |
| Base-case resets | Simulator plus real Seed attempts with changed current or nonzero count |
| Wrong parameters | IPA wrong context/dimension; Ragu different native/nested registry tags |
| Malformed encodings | Strict scalar/point/length/version/truncation/trailing/rank tests |
| Modified proof | IPA point/scalar/order tests; Ragu compressed-byte modifications and terminal rejection |
| Bounded file read | CLI reads at most 128 KiB plus one byte before decoding; Rust boundary/cursor/I/O tests and oversized-file CLI case |
| Process-result gate | Exact expected rejection status and diagnostic; strict `verified: true`, terminal-check flag and independent statement; 13 regression tests in `tests/test_acceptance.py` |
| Reproducible CI | `.github/workflows/ci.yml`; executed on hosted `ubuntu-24.04` ([run 41509961599](https://github.com/intelliDean/Build-Halo-Lab/actions/runs/41509961599), [metadata](../evidence/github-actions-run-41509961599.json)) |
| Proof/time/hardware measurements | Single real release run per case in acceptance JSON; timing scope disclosed |
| Complete assessed lesson | `lessons/ipa`, `tests/assessment.py`, reference 8/8 and five mutants rejected |
| Reviewer / pilot materials | `docs/review-checklist.md`, `docs/pilot.md`; no external participation invented |
| Honest scope boundary | Original IPA plus Ragu demonstration is not an independently implemented complete Halo system |

Release review must read the raw evidence and remaining limitations, rather than
treating this table as a security certification.

## Completed local acceptance

- [x] Inspected/pinned official Ragu and reproduced the unchanged example.
- [x] Implemented all three Rust crates and original published-construction IPA.
- [x] All 17 workspace tests passed, including real recursive adversarial proofs.
- [x] Generated and independently verified 1, 2, 4 and 8 steps in separate processes.
- [x] Completed both curves' final accumulator checks with no history list.
- [x] All 20 CLI acceptance subprocess checks returned their expected outcomes.
- [x] All 13 acceptance-validator regression tests passed, including crash and false-flag responses.
- [x] Release build, formatting and Clippy with warnings denied passed.
- [x] Measured proof sizes, proving/verification/setup times and hardware.
- [x] Lesson solution 8/8; five mutants rejected; fresh starter fails as expected.
- [x] Review checklist and pilot kit prepared, source attribution/licenses retained.
- [x] Hosted Linux CI executed on `ubuntu-24.04` ([run 41509961599](https://github.com/intelliDean/Build-Halo-Lab/actions/runs/41509961599) on commit `e22eb2a33e0c750aa974e29775de9b5f96fca80b`; artifact `lab-evidence` ID `13173361100`).
- [ ] External cryptographic review and learner pilot performed (materials prepared).
