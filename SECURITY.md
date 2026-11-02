# Educational cryptography

This is educational software, not production cryptography. Neither the original
IPA implementation nor the Ragu fork has received an independent security review
for this lab. Ragu itself states that it is under development and unaudited.

The original IPA is a non-hiding argument of knowledge under its documented
assumptions. The recursive demonstration reuses Ragu's prover, accumulators,
curve cycle, transcript, polynomial commitments and terminal verifier. Neither
component is presented as a zero-knowledge system.

The original IPA plus the Ragu-backed demonstration **does not constitute an
independently implemented complete Halo system**.

Use the technical review checklist before drawing conclusions from this lab.
Report educational correctness problems with the smallest reproducible command,
expected statement, actual result and dependency revision. Do not include secrets.
