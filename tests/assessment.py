"""Executable mathematical assessment for lessons/ipa; Python standard library only.

Exit 0: all eight assessment categories passed. Exit 1: incomplete/incorrect work.
`--self-test` additionally demonstrates that representative wrong folds fail.
This checks algebra in a formal vector space; it is not a cryptographic verifier.
"""

import argparse
import copy
import importlib.util
import json
from pathlib import Path
import random
import sys
import types

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]


def load_candidate(path):
    spec = importlib.util.spec_from_file_location("learner_candidate", path)
    if spec is None or spec.loader is None:
        raise ValueError("cannot load candidate")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    for name in ("fold_witness", "fold_bases", "fold_commitment"):
        if not callable(getattr(module, name, None)):
            raise ValueError(f"missing function: {name}")
    return module


def add(*points, q):
    return tuple(sum(values) % q for values in zip(*points))


def scale(scalar, point, q):
    return tuple(scalar * value % q for value in point)


def dot(a, b, q):
    return sum(ai * bi for ai, bi in zip(a, b)) % q


def multi(scalars, points, q):
    return tuple(sum(s * point[j] for s, point in zip(scalars, points)) % q
                 for j in range(len(points[0])))


def commitment(a, b, g, h, u, q):
    return add(multi(a, g, q), multi(b, h, q), scale(dot(a, b, q), u, q), q=q)


def formal_instance(n, rng, q):
    # Every generator has its own coordinate. Equality checks every coefficient,
    # so a relation cannot pass just because small toy group elements collide.
    width = 2 * n + 1
    units = [tuple(int(i == j) for j in range(width)) for i in range(width)]
    a = [rng.randrange(q) for _ in range(n)]
    b = [rng.randrange(q) for _ in range(n)]
    return a, b, units[:n], units[n:2*n], units[-1]


def cross_commitments(a, b, g, h, u, q):
    half = len(a) // 2
    left = add(multi(a[:half], g[half:], q), multi(b[half:], h[:half], q),
               scale(dot(a[:half], b[half:], q), u, q), q=q)
    right = add(multi(a[half:], g[:half], q), multi(b[:half], h[half:], q),
                scale(dot(a[half:], b[:half], q), u, q), q=q)
    return left, right


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def check_known_witness(candidate):
    a, b = candidate.fold_witness([1, 2, 3, 4], [5, 6, 7, 8], 2, 101)
    require(list(a) == [54, 6] and list(b) == [67, 19],
            "known fold differs; inverse of 2 modulo 101 is 51")
    a, b = candidate.fold_witness([1, 2], [3, 4], 1, 101)
    require(list(a) == [3] and list(b) == [7], "x=1 fold failed")


def check_random_witness(candidate):
    rng = random.Random(0xB017)
    for q in (17, 101, 2**61 - 1):
        for n in (2, 4, 8, 16):
            for _ in range(12):
                a = [rng.randrange(-2*q, 3*q) for _ in range(n)]
                b = [rng.randrange(-2*q, 3*q) for _ in range(n)]
                x = rng.randrange(1, q)
                actual_a, actual_b = candidate.fold_witness(a, b, x, q)
                require(len(actual_a) == n//2 and len(actual_b) == n//2,
                        "wrong folded length")
                for i in range(n//2):
                    # Multiplying by x avoids using an inverse in this oracle.
                    require((x * actual_a[i] - x*x*a[i] - a[i+n//2]) % q == 0,
                            "a coefficient equation failed")
                    require((x * actual_b[i] - b[i] - x*x*b[i+n//2]) % q == 0,
                            "b coefficient equation failed")
                require(all(0 <= v < q for v in list(actual_a) + list(actual_b)),
                        "folded scalars must be canonical residues")


def check_bases(candidate):
    rng = random.Random(0xBA5E)
    for n in (2, 4, 8):
        q, x = 101, 3
        _, _, g, h, _ = formal_instance(n, rng, q)
        ng, nh = candidate.fold_bases(g, h, x, q)
        require(len(ng) == n//2 and len(nh) == n//2, "wrong folded bases length")
        for i in range(n//2):
            require(scale(x, ng[i], q) == add(g[i], scale(x*x, g[i+n//2], q), q=q),
                    "G coefficients failed")
            require(scale(x, nh[i], q) == add(scale(x*x, h[i], q), h[i+n//2], q=q),
                    "H coefficients failed")


def check_update(candidate):
    q, x = 101, 7
    p, left, right = (4, 5, 6), (7, 8, 9), (10, 11, 12)
    actual = candidate.fold_commitment(p, left, right, x, q)
    require(len(actual) == len(p), "wrong commitment dimension")
    require(scale(x*x, actual, q) == add(scale(x*x, p, q),
                scale(x**4, left, q), right, q=q), "commitment update equation failed")
    require(all(0 <= v < q for v in actual), "commitment residues not canonical")


def check_identity(candidate):
    rng = random.Random(0xF01D)
    for q in (17, 101, 2**61 - 1):
        for n in (2, 4, 8, 16):
            for _ in range(8):
                a, b, g, h, u = formal_instance(n, rng, q)
                p = commitment(a, b, g, h, u, q)
                left, right = cross_commitments(a, b, g, h, u, q)
                x = rng.randrange(1, q)
                na, nb = candidate.fold_witness(a, b, x, q)
                ng, nh = candidate.fold_bases(g, h, x, q)
                np = candidate.fold_commitment(p, left, right, x, q)
                require(tuple(np) == commitment(na, nb, ng, nh, u, q),
                        "formal group identity failed")


def check_reduction(candidate):
    rng = random.Random(0xC1C1E)
    for n in (2, 4, 8, 16, 32):
        q = 2**61 - 1
        a, b, g, h, u = formal_instance(n, rng, q)
        p = commitment(a, b, g, h, u, q)
        rounds = 0
        while len(a) > 1:
            left, right = cross_commitments(a, b, g, h, u, q)
            x = rng.randrange(1, q)
            a, b = candidate.fold_witness(a, b, x, q)
            g, h = candidate.fold_bases(g, h, x, q)
            p = candidate.fold_commitment(p, left, right, x, q)
            rounds += 1
        require(rounds == n.bit_length()-1, "incorrect logarithmic round count")
        require(tuple(p) == commitment(a, b, g, h, u, q), "final scalar check failed")
        changed = add(tuple(p), tuple([1] + [0] * (len(p)-1)), q=q)
        require(changed != commitment(a, b, g, h, u, q), "tamper oracle broken")


def check_rejections(candidate):
    def rejected(function, *args):
        try:
            function(*args)
        except ValueError:
            return
        raise AssertionError("invalid shape or zero challenge must raise ValueError")

    for a, b in (([], []), ([1], [2]), ([1, 2], [3]), ([1, 2, 3], [4, 5, 6])):
        rejected(candidate.fold_witness, a, b, 2, 101)
    for bad_x in (0, 101, -101):
        rejected(candidate.fold_witness, [1, 2], [3, 4], bad_x, 101)
        rejected(candidate.fold_bases, [(1,), (2,)], [(3,), (4,)], bad_x, 101)
        rejected(candidate.fold_commitment, (1,), (2,), (3,), bad_x, 101)
    rejected(candidate.fold_bases, [(1,), (2,)], [(3,)], 2, 101)
    rejected(candidate.fold_bases, [(1,), (2, 3)], [(4,), (5,)], 2, 101)
    rejected(candidate.fold_bases, [(), ()], [(), ()], 2, 101)
    rejected(candidate.fold_commitment, (), (), (), 2, 101)
    rejected(candidate.fold_commitment, (1,), (2, 3), (4,), 2, 101)


def check_immutability(candidate):
    rng = random.Random(0x5AFE)
    a, b, g, h, u = formal_instance(4, rng, 101)
    before = copy.deepcopy((a, b, g, h, u))
    candidate.fold_witness(a, b, 3, 101)
    candidate.fold_bases(g, h, 3, 101)
    candidate.fold_commitment(u, g[0], h[0], 3, 101)
    require(before == (a, b, g, h, u), "candidate mutated an input")
    # Noncanonical challenges represent the same field element.
    require(candidate.fold_witness(a, b, -98, 101) ==
            candidate.fold_witness(a, b, 104, 101), "challenge reduction failed")


CHECKS = [check_known_witness, check_random_witness, check_bases, check_update,
          check_identity, check_reduction, check_rejections, check_immutability]


def assess(candidate):
    results = []
    for check in CHECKS:
        try:
            check(candidate)
            results.append({"test": check.__name__.removeprefix("check_"), "passed": True})
        except Exception as error:
            results.append({"test": check.__name__.removeprefix("check_"), "passed": False,
                            "error": f"{type(error).__name__}: {error}"})
    return results


def mutation_self_test(reference):
    mutants = {}
    def clone():
        return types.SimpleNamespace(**{name: getattr(reference, name) for name in
            ("fold_witness", "fold_bases", "fold_commitment")})
    wrong = clone()
    wrong.fold_witness = lambda a, b, x, q: reference.fold_witness(a, b, pow(x, -1, q), q)
    mutants["swapped_witness_challenges"] = wrong
    wrong = clone()
    wrong.fold_bases = lambda g, h, x, q: reference.fold_bases(g, h, pow(x, -1, q), q)
    mutants["swapped_generator_challenges"] = wrong
    wrong = clone()
    wrong.fold_commitment = lambda p, left, right, x, q: add(p, scale(x*x, left, q), q=q)
    mutants["omitted_R_term"] = wrong
    wrong = clone()
    wrong.fold_commitment = lambda p, left, right, x, q: add(p, scale(x, left, q),
                                                               scale(pow(x, -1, q), right, q), q=q)
    mutants["unsquared_challenges"] = wrong
    wrong = clone()
    wrong.fold_witness = lambda a, b, x, q: ([0] * (len(a)//2), [0] * (len(a)//2))
    mutants["all_zero_witness"] = wrong
    return [{"mutant": name, "rejected": any(not result["passed"] for result in assess(mutant))}
            for name, mutant in mutants.items()]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidate", type=Path, default=ROOT / "lessons/ipa/starter.py")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        candidate = load_candidate(args.candidate.resolve())
        checks = assess(candidate)
        report = {"candidate": str(args.candidate), "passed": sum(r["passed"] for r in checks),
                  "total": len(checks), "checks": checks}
        success = all(r["passed"] for r in checks)
        if args.self_test:
            reference = load_candidate(ROOT / "lessons/ipa/solution.py")
            reference_checks = assess(reference)
            report["reference_passed"] = sum(r["passed"] for r in reference_checks)
            report["mutations"] = mutation_self_test(reference)
            success = success and all(r["passed"] for r in reference_checks) and all(
                r["rejected"] for r in report["mutations"])
        print(json.dumps(report, indent=2))
        return 0 if success else 1
    except Exception as error:
        print(json.dumps({"error": f"{type(error).__name__}: {error}"}, indent=2))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
