# Attribution and scope of original work

## Original code in this lab

* `crates/edu-ipa`: a new implementation of commitments, a domain-separated
  transcript, inner-product prover/verifier and binary encoding, following the
  cited published construction. No existing IPA prover or verifier is called.
* `crates/recursive-demo`: the application relation and public statement, boundary
  constraints, command-facing proof handling and adversarial tests.
* `crates/lab-cli`: command interface and independent expected-statement handling.
* `lessons`, `tests`: exercise, worked solution, assessment and acceptance harness.
* The codec additions listed in `docs/ragu-api.md` expose terminal compressed
  proofs through a canonical bounded encoding. They are a local fork patch.

## Reused implementations

* [Ragu](https://github.com/tachyon-zcash/ragu), pinned at
  `4e3d08ddea2542c7ebb2debec2ec94bc6a4f59e5`, supplies the actual recursive PCD
  engine, recursion circuits, split accumulation, polynomial commitments,
  Pasta arithmetic, Poseidon transcript and terminal compression/verification.
  Its source is included under `vendor/ragu`, with its MIT/Apache-2.0 licenses,
  COPYRIGHT and source documentation. The lab's MIT license does not replace
  those licenses.
* `curve25519-dalek` supplies Ristretto group and scalar arithmetic for the
  independent IPA. `sha2` supplies SHA-512, used for generator derivation and
  transcript challenges. The IPA crate's construction notes specify encodings
  and domains.
* Rust and transitive crates are reused under their respective licenses. Direct
  versions and `Cargo.lock` fix the dependency graph.

## Construction sources

* Bünz, Bootle, Boneh, Poelstra, Wuille and Maxwell, *Bulletproofs: Short Proofs
  for Confidential Transactions and More*, [ePrint 2017/1066](https://eprint.iacr.org/2017/1066),
  §3, Protocols 1–2; [author PDF](https://web.stanford.edu/~buenz/pubs/bulletproofs.pdf).
  This lab implements the non-hiding inner-product argument, not Bulletproofs
  range proofs.
* Bowe, Grigg and Hopwood, *Halo: Recursive Proof Composition without a Trusted
  Setup*, [ePrint 2019/1021](https://eprint.iacr.org/2019/1021).
* Ragu's [book](https://tachyon.z.cash/ragu/) is also included at the pinned
  revision in `vendor/ragu/book/src`. Source APIs at the pinned commit take
  precedence over stale examples in the book.

The original IPA and the Ragu-backed application are separate demonstrations;
the original IPA is not wired into Ragu and does not implement its accumulator
scheme. Together they do not constitute an independently implemented complete
Halo system.
