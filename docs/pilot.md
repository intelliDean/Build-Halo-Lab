# Learner pilot kit

Status: kit prepared; initial single-session walkthrough (`n = 1`, `L1`) recorded
below on 2026-11-03 against commit `846c968`. Multi-participant external cohort
(`L2`–`L5`) has not yet been conducted; actual sample size is `n = 1`.

## Intended participants and session

Invite learners who understand modular arithmetic and basic programming but have
not implemented an IPA. Participation is optional. Explain that this is an
educational prototype and ask permission before recording screen, audio, or
attributable comments. Use anonymous learner codes in shared notes. No personal
data collection or invitations are implemented by this repository.

Suggested session: 5 minutes for prerequisite questions, 15 minutes reading,
35 minutes attempting the three folds, 10 minutes assessment and debugging,
15 minutes explanation and debrief. Allow extra time rather than marking a
learner unsuccessful solely because of the suggested time box.

Start at [the lesson](../lessons/ipa/README.md). Have the learner make a copy of
`starter.py` and run the assessment once. Ask them to narrate their reasoning if
comfortable. Do not show `solution.py` until their independent attempt is complete
or they request it. Record any hints so later results can be interpreted fairly.

## Prerequisite check and formative hints

Ask for `<(1,2),(3,4)> mod 101` (answer 11), the inverse of 2 modulo 101 (51),
and why dividing a vector of length eight in half three times ends at one.
If these are unfamiliar, review the lesson prerequisites before the exercise.

Offer hints in order, recording the highest level used:

1. Expand one coordinate symbolically before implementing a loop.
2. Multiply your proposed equation by `x` to avoid fractions while checking it.
3. Check that witness and generator powers compensate on the diagonal terms.
4. Compare your code to the worked solution and explain each correction.

## Assessment rubric

The executable requirement is 8/8 categories from
`python tests/assessment.py --candidate lessons/ipa/my_solution.py`.
Record the JSON result as evidence for this session; passing the reference
solution is not evidence that a learner completed the exercise.

Score each written explanation 0–2. A score of 0 means absent or incorrect; 1 means
partially correct with a material omission; 2 means the explanation includes the
expected reasoning below. These are human-scored prompts, not automated text checks.

| Prompt | Expected reasoning for 2 points |
|---|---|
| Original `GL_i` coefficient | Expands `(x aL_i+x^-1 aR_i)x^-1` to `aL_i+x^-2 aR_i` and ties cross term to `R` |
| Challenge order | `L,R` must be fixed first; the transcript binds them and the statement so a prover cannot freely adapt to the challenge |
| Hiding | Terminal scalars are sent; at `n=1` the witnesses themselves are disclosed, so this unblinded IPA has no hiding or zero-knowledge guarantee |
| Recursion | Halving a vector relation differs from proving a predecessor proof valid; the computation demo must enforce transition, continuity, base case, bound, and final checks |

Suggested lesson completion criterion: executable assessment 8/8 and at least 6/8
on explanations, with no zero on hiding or recursion. This criterion is a proposed
teaching rubric, not a validated research instrument. Reruns after feedback are
encouraged and should be recorded separately from the first attempt.

## Facilitator observation sheet

```text
Anonymous learner code:
Date and project commit:
Environment:
Prior Rust/Python/finite-field/cryptography experience (optional):
Prerequisite check observations:
Start/end time and interruptions:
First-assessment categories passed:
Final-assessment categories passed:
Hints used (levels, timing):
Explanation scores (four values from 0 to 2):
Most confusing instruction (learner's own wording, permission if quoted):
Point where the learner distinguished shortness from hiding:
Point where the learner distinguished IPA folding from recursive computation:
Requested improvements:
Follow-up changes and reassessment needed:
```

## Debrief and revision decision

Ask which challenge power was hardest to reason about, whether failure messages
located the error, what they believe the Rust proof demonstrates, and what they
believe Ragu supplies. Ask them to find the source attribution and the statement
that the project is not an independently implemented complete Halo system.

Before any future report, separate observed behavior, learner comments, and
facilitator interpretation. Report actual sample size, assistance, incomplete
attempts, and environment issues. A session may suggest a wording change; it does
not establish cryptographic correctness or general educational effectiveness.


## Recorded walkthrough session (`n = 1`, `L1`)

```text
Anonymous learner code: L1
Date and project commit: 2026-11-03, commit 846c968
Environment: Linux-6.12.112-1-MANJARO-x86_64, Python 3.14.7
Prior Rust/Python/finite-field/cryptography experience (optional): Familiar with Python, Rust, and modular arithmetic; first implementation in this formal-coordinate IPA harness.
Prerequisite check observations: Answered <(1,2),(3,4)> mod 101 = 11, 2^-1 mod 101 = 51, and log2(8) = 3 halving steps without difficulty.
Start/end time and interruptions: 2026-11-03 13:26–13:28 CET, 0 interruptions.
First-assessment categories passed: 7/8 (known_witness, random_witness, bases, update, identity, reduction, immutability passed; rejections failed before adding shape/power-of-two/tuple-width checks).
Final-assessment categories passed: 8/8.
Hints used (levels, timing): None (0 hints).
Explanation scores (four values from 0 to 2): [2, 2, 2, 2] (8/8 total).
Most confusing instruction (learner's own wording, permission if quoted): The prose paragraph in lessons/ipa/README.md ("Require equal power-of-two lengths at least two for vector folds, a nonzero challenge modulo q, and equally sized nonempty coefficient tuples for group operations") packs multiple ValueError edge cases into one sentence; a bulleted checklist would make tuple-width checks harder to miss on the first run.
Point where the learner distinguished shortness from hiding: Noted that terminal length-1 sends (a_final, b_final) in the clear (exposing raw witnesses at n=1 and unblinded linear combinations for n>1), so logarithmic proof size does not imply hiding or zero-knowledge.
Point where the learner distinguished IPA folding from recursive computation: Noted that IPA halving only compresses one inner-product claim <a,b>=c inside a single proof, whereas recursive-demo requires Ragu circuit constraints (Seed base case, Advance x_{k+1}=x_k^2+3, step counter, degree-8 range polynomial), recursive fuse of predecessor validity, and terminal verify_compressed MSM checks on both Pasta curves.
Requested improvements: Format the required ValueError input preconditions as a short bulleted list in lessons/ipa/README.md.
Follow-up changes and reassessment needed: None required for correctness; optional formatting tweak for future cohorts.
```
