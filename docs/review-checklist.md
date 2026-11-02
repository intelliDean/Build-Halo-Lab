# Technical review checklist

Prepared for a future reviewer. No external review has been claimed or completed.
Record the reviewed commit, environment, commands, findings, and unresolved issues.
An unchecked item is a request for inspection, not evidence of a passing result.

## Statement and recursive computation

- [ ] Confirm the pinned official Ragu revision and toolchain against upstream.
- [ ] Reproduce the upstream example using the documented commands.
- [ ] Trace public initial value, final value, and step count into constraints.
- [ ] Confirm field reduction and encoding use the documented Ragu field.
- [ ] Inspect the constrained base case, continuity rule, step-count bound, and
      recursive predecessor validity; identify each constraint by source location.
- [ ] Try an invalid intermediate transition through the circuit/proof API.
- [ ] Try a reset to a fresh base case inside a non-base chain.
- [ ] Try a predecessor proof invalid under the expected recursive relation.
- [ ] Reproduce 1, 2, 4, and 8 steps and check final public values independently.
- [ ] Verify each final artifact in a new process with an independently supplied
      expected statement and without an intermediate-state history.
- [ ] Follow the pinned Ragu verifier API to every required final accumulator
      check. A successful partial verifier call is not sufficient evidence.
- [ ] Confirm the implementation recursively verifies a predecessor; a list of
      independent verification results does not satisfy this requirement.

## Original IPA component

- [ ] Compare Protocol 1 and Protocol 2 of Section 3 of the attributed
      [Bulletproofs paper](https://web.stanford.edu/~buenz/pubs/bulletproofs.pdf)
      against the implementation and identify deliberate adaptations.
- [ ] Confirm established curve/field arithmetic is reused and protocol logic,
      transcript, commitment construction, and encoding are original lab code.
- [ ] Inspect generator derivation: role/index/parameter context separation,
      appropriate hash-to-group operation, and no known scalar-multiple shortcut.
- [ ] Check transcript binding of parameter identity, vector length, commitment,
      claimed inner product, initial challenge, every `L,R`, and round order.
- [ ] Confirm transcript framing is unambiguous and all challenges are nonzero.
- [ ] Check opposite witness/generator powers and squared commitment-update powers.
- [ ] Confirm the expected statement is an independent verifier argument.
- [ ] Verify that false statements are rejected by the verifier even when an
      adversarial test bypasses honest-prover validation.
- [ ] Inspect canonical point/scalar parsing, exact lengths, bounds, and trailing
      byte rejection; mutate serialized proofs and parameter contexts.
- [ ] Check length-one and maximum supported vector length behavior.
- [ ] Confirm the absence of blinding is documented and no zero-knowledge or
      production-security claim is made.

## Reproducibility and teaching

- [ ] Rebuild using the locked dependency graph and run documented CI commands.
- [ ] Check raw logs distinguish executed tests from pending checks and blockers.
- [ ] Inspect timing scope, proof-byte definition, profile, run count, toolchain,
      operating system, CPU, memory, and measurement limitations in benchmark data.
- [ ] Run the learner starter: it must fail with the incomplete functions.
- [ ] Run the worked solution: all eight assessment categories must pass.
- [ ] Run assessment self-tests: every documented mutation must be rejected.
- [ ] Attempt the exercise without consulting the solution, then evaluate the
      written explanations using the [pilot rubric](pilot.md).
- [ ] Confirm source attribution, licensing boundaries, and the explicit statement
      that this project is not an independently implemented complete Halo system.

## Review record template

```text
Reviewer (optional public attribution):
Date and reviewed commit:
Environment and reproduction commands:
Checks completed:
Findings (source location, impact, reproduction, proposed correction):
Checks not completed and reason:
Disposition: pending / changes requested / educational scope accepted
```
