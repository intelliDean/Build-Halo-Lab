# Sources inspected for the lesson

Access date: 2026-10-20. These notes record primary-source inspection and deliberate
adaptations; they do not claim an external technical review.

1. Bünz, Bootle, Boneh, Poelstra, Wuille, and Maxwell,
   *Bulletproofs: Short Proofs for Confidential Transactions and More*, ePrint
   2017/1066, Section 3, relation (2), relation (3), and Protocols 1 and 2.
   Canonical record: <https://eprint.iacr.org/2017/1066>.
   Read from the author-hosted 45-page PDF:
   <https://web.stanford.edu/~buenz/pubs/bulletproofs.pdf>.
   The direct ePrint PDF fetch returned HTTP 403 in the browsing tool; the author
   copy was available. Protocol 1 was on printed page 15, Protocol 2 on printed
   page 16. Selection and fold conventions were communicated to the Rust IPA
   implementer before this lesson was finalized.

2. Bowe, Grigg, and Hopwood, *Recursive Proof Composition without a Trusted Setup*,
   ePrint 2019/1021. Read the primary ePrint abstract and bibliographic record:
   <https://eprint.iacr.org/2019/1021>. This supports the lesson's attribution of
   Halo and the broad concept of recursive proof composition. It is not a claim
   that the entire Halo paper was audited as part of this lesson task.

The lesson uses original explanatory examples, exercise code, and assessment.
The known-answer example uses modulus 101 and a length-four witness selected for
hand calculation. The formal-basis assessment is a mathematical instrument, not a
replacement cryptographic construction from either paper. The full project
construction notes specify the actual Rust curve, transcript, and canonical
encoding adaptations, which are distinct from the published abstract protocols.
