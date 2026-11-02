"""Learner exercise: implement one round of the Bulletproofs IPA fold.

Read README.md before editing. All arithmetic is modulo the supplied prime q.
Group elements here are tuples of formal coefficients, not elliptic-curve points.
Do not import solution.py. The assessment imports this module by its file path.
"""


def fold_witness(a, b, x, q):
    """Return (a_next, b_next), each a list of length len(a)//2.

    Require equal power-of-two lengths >= 2 and a nonzero challenge modulo q.
    Accept noncanonical integer inputs by reducing modulo q; do not mutate inputs.
    Invalid inputs must raise ValueError. Assume q is prime and q >= 3.
    """
    raise NotImplementedError("Exercise 1: fold both witness vectors")


def fold_bases(g, h, x, q):
    """Return (g_next, h_next), lists of formal group-element tuples.

    Require equal power-of-two lengths >= 2, nonempty equally sized tuples, and
    nonzero x modulo q. Apply scalar multiplication and addition coefficientwise.
    Invalid inputs must raise ValueError; do not mutate inputs.
    """
    raise NotImplementedError("Exercise 2: fold the two generator vectors")


def fold_commitment(p, left, right, x, q):
    """Return the folded commitment as a tuple of coefficients modulo q.

    Require nonempty equal-size tuples and nonzero x modulo q.
    Invalid inputs must raise ValueError; do not mutate inputs.
    """
    raise NotImplementedError("Exercise 3: update the verifier's commitment")
