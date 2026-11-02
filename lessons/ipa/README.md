# Lesson: shrink an inner-product relation without losing its meaning

This lesson teaches one round of the inner-product argument used in the lab's
original Rust `edu-ipa` crate. You will implement the algebra, test it against
independent coefficient equations, and explain its relation to recursive proofs.
Allow 60–90 minutes. The lesson and assessment require Python 3.11 or newer and
only the standard library; the actual elliptic-curve proof is implemented in Rust.

## Prerequisites and goals

You should be able to compute a dot product, split a vector in half, work modulo a
prime, and explain scalar multiplication in an additive group. Python lists and
functions are enough for the exercise; Rust familiarity helps with the extension.
If modular inverses are new, find `2^-1 mod 101`: the answer is 51 because
`2 * 51 = 1 mod 101`. An inverse exists for every nonzero scalar in a prime field.

After completing the lesson, you should be able to:

- Implement witness, generator, and commitment folds with the correct powers.
- Verify a fold coefficient by coefficient and reduce length eight to length one.
- Distinguish a short argument from a hiding argument and a recursive computation.
- Explain why the verifier must bind the public statement before taking challenges.

## The construction we selected

The selected published construction is Bünz, Bootle, Boneh, Poelstra, Wuille, and
Maxwell, *Bulletproofs: Short Proofs for Confidential Transactions and More*,
[ePrint 2017/1066](https://eprint.iacr.org/2017/1066), Section 3, Protocols 1 and 2.
The [author-hosted PDF](https://web.stanford.edu/~buenz/pubs/bulletproofs.pdf) was read
for this implementation: Protocol 1 is on printed page 15 and Protocol 2 on printed
page 16 in that copy. Page numbering can differ in other revisions. The paper uses
multiplicative group notation; we write group operations additively. Protocol 1
binds the asserted dot product using a fresh challenge, then Protocol 2 repeatedly
halves the vectors and ends with two scalars. Fiat–Shamir replaces verifier
randomness with transcript challenges in this implementation.

The Rust code, transcript framing, encoding, and this exercise are original lab
work. The published protocol is attributed research, and curve/field/hash
operations come from dependencies. Read
[construction notes](../../docs/ipa-construction.md) for the exact Rust conventions.

## Write down the statement before proving it

Let `a,b` be scalar vectors of equal power-of-two length `n`. Let `G,H` be vectors
of group generators, and let `U` be another generator. We use this shorthand:

```text
<a,b> = sum_i a_i b_i                scalar dot product
<a,G> = sum_i a_i G_i                group multiscalar multiplication
C = <a,G> + <b,H>                    public vector commitment
c = <a,b>                           public claimed dot product
```

The independent expected statement includes the parameter set, `n`, `C`, and `c`.
The prover does not get to choose a replacement statement at verification time.

After transcript binding, derive a nonzero scalar `y` and define `Q=yU` and
`P=C+cQ`. The folding argument is about the combined relation

```text
P = <a,G> + <b,H> + <a,b> Q.
```

For the exercise, call the dot-product generator `u`; it represents `Q` above.
The fresh `y` prevents treating an incorrect asserted dot product as a freely
adjustable generator coefficient. Transcript binding is part of the protocol,
not metadata added after the proof is created.

## Derive one round

Split every vector into equal left and right halves. Before the round challenge
`x` is known, the prover supplies these cross-term commitments:

```text
cL = <aL,bR>                         cR = <aR,bL>
L = <aL,GR> + <bR,HL> + cL u
R = <aR,GL> + <bL,HR> + cR u
```

Bind `L,R` to the transcript, then derive nonzero `x`. Every expression below is
over the scalar field; negative exponents mean multiplicative inverses.

```text
a' = x aL + x^-1 aR                 b' = x^-1 bL + x bR
G' = x^-1 GL + x GR                 H' = x HL + x^-1 HR
P' = P + x^2 L + x^-2 R
```

Here is the algebra you should check yourself. Expanding the new dot product gives

```text
<a',b'> = <aL,bL> + <aR,bR> + x^2 <aL,bR> + x^-2 <aR,bL>.
```

The coefficient of each original `GL_i` in `<a',G'>` is
`aL_i + x^-2 aR_i`. The coefficient of `GR_i` is
`x^2 aL_i + aR_i`. The `H` terms have the opposite powers. Combining these
coefficients with the dot-product expansion gives

```text
P' = <a',G'> + <b',H'> + <a',b'> u.
```

The relation has the same form at half the length. At length one the verifier
checks `P_final = a_final G_final + b_final H_final + a_final b_final u`.
The proof therefore transmits two group elements per halving round and two final
scalars. The verifier still processes generators associated with the original
vector; short communication does not make all verification work logarithmic.

## Starter exercise

From the workspace root, make a learner copy so the starter remains reproducible:

```powershell
Copy-Item lessons/ipa/starter.py lessons/ipa/my_solution.py
python tests/assessment.py --candidate lessons/ipa/my_solution.py
```

On another shell use its ordinary file-copy command; the assessment invocation is
the same. The fresh starter intentionally fails with `NotImplementedError`.
Implement all three functions in your copy:

1. `fold_witness(a,b,x,q)` returns the two shortened scalar lists.
2. `fold_bases(g,h,x,q)` returns the two shortened generator lists.
3. `fold_commitment(p,left,right,x,q)` returns the updated group element.

For this exercise a group element is a tuple of coefficients modulo `q`. Addition
and scalar multiplication operate coordinatewise. For example,
`3*(1,0,2) + 5*(0,1,4) = (3,5,26) mod 101`.
The assessment assigns each original generator its own coordinate. Consequently
an incorrect fold cannot pass by exploiting collisions between small toy-group
elements. This formal vector space is an algebraic test instrument; it has no
discrete-log security, serialization protocol, or cryptographic proofs.

Require equal power-of-two lengths at least two for vector folds, a nonzero
challenge modulo `q`, and equally sized nonempty coefficient tuples for group
operations. Invalid shapes and zero challenges must raise `ValueError`.
Reduce integer inputs modulo `q`, return canonical residues, and leave inputs
unchanged. The exercise assumes a prime modulus at least three; it does not ask
you to implement a primality test. Do not read or import the worked solution until
you have attempted the three functions.

## Assessment and pass criterion

Run the same command after each change. All eight categories must pass:

| Category | What is checked |
|---|---|
| Known witness | A hand-computable example and the `x=1` boundary |
| Random witness | Coefficient equations over three primes and four lengths |
| Bases | Correct, opposite challenge powers for `G` and `H` |
| Update | The verifier's commitment uses squared challenge powers |
| Identity | Exact coefficient equality for a complete round |
| Reduction | Lengths 2,4,8,16,32 reach a valid final scalar relation |
| Rejections | Zero challenges and invalid dimensions raise `ValueError` |
| Immutability | Inputs remain intact and challenge representatives agree |

The assessment uses deterministic random seeds for reproducibility. It independently
builds `L,R` and commitments rather than calling a learner's commitment helper.
It checks many equations after multiplying through by `x` or `x^2`, so its main
oracle does not merely repeat your inverse implementation. It imports and executes
the submitted Python file locally; use your own exercise file as the candidate.

```powershell
python tests/assessment.py --candidate lessons/ipa/solution.py --self-test
```

This checks the reference and demonstrates rejection of five representative
mutations: reversed witness challenges, reversed generator challenges, omission
of `R`, unsquared powers, and all-zero witnesses. It tests correctness of this
exercise, not knowledge of a particular written phrase. It is not an exhaustive
proof of software correctness or a cryptographic security evaluation.

For the short written portion, answer these prompts in your own words and discuss
them with a reviewer using [the pilot rubric](../../docs/pilot.md):

1. Expand the coefficient of `GL_i` and explain why the inverse is on that half.
2. Why must `L,R` be committed before deriving `x`?
3. What does the terminal length-one case disclose? Does a short proof imply hiding?
4. Does this folding loop itself prove `x_next=x_current^2+3`? Explain what else the
   Ragu demonstration must enforce.

## Worked solution

The complete runnable reference is [solution.py](solution.py). Each new witness
coordinate combines matching positions in opposite halves. Generator coordinates
use the reciprocal pattern so expansion reconstructs the original diagonal terms.
`fold_commitment` adds the two weighted cross-term commitments to `P`.

For `q=101`, `a=(1,2,3,4)`, `b=(5,6,7,8)`, `x=2`, we have `x^-1=51`.
The first fold gives `a'=(54,6)` and `b'=(67,19)`.
The original dot product is `70`; the cross products are `cL=23` and `cR=39`.
The new dot product is `96`, agreeing with `70+4*23+76*39 mod 101`, because
`x^-2=76`. Notice that the inner product changes. The invariant includes the
updated commitment and folded generators; preserving the original scalar dot
product by itself would be the wrong exercise.

Zero is excluded because its inverse does not exist. Silently assigning an
inverse to zero changes the argument. Lengths not equal to powers of two require
an explicit padding protocol; this lab rejects them instead.

## From this lesson to Halo recursion

There are two different repetitions in this project. The IPA halves a relation
inside one argument. The `recursive-demo` chain instead carries a claim about
previous computation into another proof. Checking a list of independent proofs
does not establish that recursive relationship.

The original Halo paper by Bowe, Grigg, and Hopwood describes recursive proof
composition without a trusted setup and nested amortization of verification work:
[ePrint 2019/1021](https://eprint.iacr.org/2019/1021). The lesson's folding invariant
helps explain how algebraic verification work can be compressed. A full Halo
system also needs a constraint system, polynomial commitments, transcript rules,
recursive verification, curve/field compatibility, and completion of deferred
accumulator checks. Ragu supplies the recursive proof machinery used by the lab;
the Rust `edu-ipa` component is a separately implemented educational protocol and
is not substituted into Ragu's internals.

The unblinded lab IPA provides no hiding guarantee and is not claimed to be
zero knowledge. At length one it explicitly reveals both witness scalars.
Hashed generators should have unknown discrete-log relationships, and security
reasoning also relies on the discrete-log assumption and Fiat–Shamir's
random-oracle model. The formal exercise intentionally provides none of those
cryptographic assumptions.

The original IPA plus the Ragu-backed demonstration does not constitute an
independently implemented complete Halo system. See the project acceptance record
for which recursion runs and final checks were actually executed.
