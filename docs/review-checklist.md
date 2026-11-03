# Technical review checklist

Technical review and learner-walkthrough verification completed on 2026-11-03
against commit `846c968` (initial code commit `e22eb2a`). This is a technical
code-and-reproduction verification of the educational lab, not a production
cryptographic audit.

## Statement and recursive computation

- [x] Confirm the pinned official Ragu revision and toolchain against upstream.
- [x] Reproduce the upstream example using the documented commands.
- [x] Trace public initial value, final value, and step count into constraints.
- [x] Confirm field reduction and encoding use the documented Ragu field.
- [x] Inspect the constrained base case, continuity rule, step-count bound, and
      recursive predecessor validity; identify each constraint by source location.
- [x] Try an invalid intermediate transition through the circuit/proof API.
- [x] Try a reset to a fresh base case inside a non-base chain.
- [x] Try a predecessor proof invalid under the expected recursive relation.
- [x] Reproduce 1, 2, 4, and 8 steps and check final public values independently.
- [x] Verify each final artifact in a new process with an independently supplied
      expected statement and without an intermediate-state history.
- [x] Follow the pinned Ragu verifier API to every required final accumulator
      check. A successful partial verifier call is not sufficient evidence.
- [x] Confirm the implementation recursively verifies a predecessor; a list of
      independent verification results does not satisfy this requirement.

## Original IPA component

- [x] Compare Protocol 1 and Protocol 2 of Section 3 of the attributed
      [Bulletproofs paper](https://web.stanford.edu/~buenz/pubs/bulletproofs.pdf)
      against the implementation and identify deliberate adaptations.
- [x] Confirm established curve/field arithmetic is reused and protocol logic,
      transcript, commitment construction, and encoding are original lab code.
- [x] Inspect generator derivation: role/index/parameter context separation,
      appropriate hash-to-group operation, and no known scalar-multiple shortcut.
- [x] Check transcript binding of parameter identity, vector length, commitment,
      claimed inner product, initial challenge, every `L,R`, and round order.
- [x] Confirm transcript framing is unambiguous and all challenges are nonzero.
- [x] Check opposite witness/generator powers and squared commitment-update powers.
- [x] Confirm the expected statement is an independent verifier argument.
- [x] Verify that false statements are rejected by the verifier even when an
      adversarial test bypasses honest-prover validation.
- [x] Inspect canonical point/scalar parsing, exact lengths, bounds, and trailing
      byte rejection; mutate serialized proofs and parameter contexts.
- [x] Check length-one and maximum supported vector length behavior.
- [x] Confirm the absence of blinding is documented and no zero-knowledge or
      production-security claim is made.

## Reproducibility and teaching

- [x] Rebuild using the locked dependency graph and run documented CI commands.
- [x] Check raw logs distinguish executed tests from pending checks and blockers.
- [x] Inspect timing scope, proof-byte definition, profile, run count, toolchain,
      operating system, CPU, memory, and measurement limitations in benchmark data.
- [x] Run the learner starter: it must fail with the incomplete functions.
- [x] Run the worked solution: all eight assessment categories must pass.
- [x] Run assessment self-tests: every documented mutation must be rejected.
- [x] Attempt the exercise without consulting the solution, then evaluate the
      written explanations using the [pilot rubric](pilot.md).
- [x] Confirm source attribution, licensing boundaries, and the explicit statement
      that this project is not an independently implemented complete Halo system.

## Review record template

```text
Reviewer (optional public attribution): intelliDean (technical review & L1 walkthrough)
Date and reviewed commit: 2026-11-03; commit 846c968 (code tree e22eb2a33e0c750aa974e29775de9b5f96fca80b)
Environment and reproduction commands:
  - Local Linux: Linux-6.12.112-1-MANJARO-x86_64 (Intel Core i5-6300U, 16.6 GB RAM), Rust 1.97.1, Python 3.14.7
  - Hosted CI: GitHub Actions ubuntu-24.04 (run 41509961599: https://github.com/intelliDean/Build-Halo-Lab/actions/runs/41509961599)
  - Commands executed and passed:
    * cargo fmt --all -- --check --config-path /dev/null (local) & cargo fmt --all --check (CI)
    * cargo build --locked --release -p lab-cli
    * cargo test --workspace --locked --release (17/17 passed: 9 edu-ipa, 3 lab-cli, 5 recursive-demo)
    * cargo clippy --workspace --all-targets --locked --release -- -D warnings
    * cargo test --manifest-path vendor/ragu/Cargo.toml -p ragu_pcd --test nontrivial --locked --release --jobs 1 --features baked,ragu_circuits/insecure-test-registry-tag (1/1 passed in 28.08s)
    * python3 -m unittest discover -s tests -p test_acceptance.py -v (13/13 passed)
    * python3 tests/acceptance.py --output evidence/generated (20/20 subprocess checks passed)
    * python3 tests/assessment.py --candidate lessons/ipa/starter.py (0/8 failed with NotImplementedError as expected)
    * python3 tests/assessment.py --candidate lessons/ipa/solution.py --self-test (8/8 passed, 5/5 mutants rejected)
    * python3 tests/assessment.py --candidate lessons/ipa/my_solution.py (L1 walkthrough: 7/8 first pass -> 8/8 final pass)
Checks completed:
  - Statement & Recursive Computation (12/12):
    * Pinned Ragu commit 4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5 verified against vendor/ragu; only transport codec added in vendor/ragu/crates/ragu_pcd/src/compress/codec.rs.
    * ChainState [initial, current, count] encoded in crates/recursive-demo/src/lib.rs:33-49 over ragu_core::pasta::Fp.
    * Base case (Seed, lib.rs:50-56) enforces initial == current and count == 0.
    * Transition (Advance, lib.rs:57-76) enforces out.initial == left.initial, out.current == left.current^2 + 3, out.count == left.count + 1, and prod_{i=1..8}(out.count - i) == 0.
    * Recursive predecessor folding via app.fuse (lib.rs:155-164) and adversarial rejection of relabeled child PCD, invalid transitions, and base resets verified in recursive_soundness_tests (lib.rs:281-392).
    * Terminal accumulator checks traced from ChainLab::verify (lib.rs:193-195) -> Application::verify_compressed (vendor/ragu/crates/ragu_pcd/src/compress/verifier.rs:31-169) -> batch::verify_openings (compress/batch/verifier.rs:19-36) -> ipa::verify_proof(...).use_challenges().eval::<B>() (ipa/verifier.rs:38-43, 67-148) on both native and nested curves.
  - Original IPA Component (11/11):
    * Bulletproofs §3 Protocols 1 & 2 verified in crates/edu-ipa/src/lib.rs:231-306.
    * Generator derivation (lib.rs:61-98) uses domain-separated SHA-512 frames mapped via RistrettoPoint::from_uniform_bytes with identity/duplicate rejection.
    * Transcript framing (lib.rs:375-416) uses length-prefixed u64_le frames and rejects Scalar::ZERO challenges.
    * Reciprocal witness/generator powers and squared L/R commitment powers verified (lib.rs:268-270, 297, 342-357).
    * Malicious prover bypass tests (prove_inner, lib.rs:459-498) and strict canonical decoding tests (lib.rs:550-584) verified.
  - Reproducibility & Teaching (8/8):
    * Starter fails 0/8, reference solution passes 8/8 + 5/5 mutants, L1 walkthrough completed 8/8 executable + 8/8 rubric.
Findings (source location, impact, reproduction, proposed correction):
  - No soundness or implementation defects found within the documented educational scope.
  - Minor usability observation from L1 walkthrough (lessons/ipa/README.md:135-138): formatting the required ValueError input preconditions as a bulleted list would make tuple-width checks easier to spot on the first pass.
Checks not completed and reason:
  - All 31 checklist items completed. Multi-participant external cohort (beyond L1 walkthrough) remains optional future work.
Disposition: educational scope accepted
```
