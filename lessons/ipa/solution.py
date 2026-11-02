"""Worked solution for the formal-coefficient exercise; not cryptography."""


def _inverse(x, q):
    if not isinstance(q, int) or q < 3 or not isinstance(x, int):
        raise ValueError("q >= 3 and integer challenge required")
    try:
        return pow(x % q, -1, q)
    except ValueError as error:
        raise ValueError("challenge must be nonzero modulo q") from error


def _length(left, right):
    n = len(left)
    if n != len(right) or n < 2 or n & (n - 1):
        raise ValueError("equal power-of-two lengths >= 2 required")
    return n // 2


def _points(points):
    if not points or not points[0]:
        raise ValueError("nonempty formal group elements required")
    width = len(points[0])
    if any(len(point) != width for point in points):
        raise ValueError("formal group elements must have equal dimensions")


def _combine(a, b, sa, sb, q):
    return tuple((sa * ai + sb * bi) % q for ai, bi in zip(a, b))


def fold_witness(a, b, x, q):
    inverse = _inverse(x, q)
    half = _length(a, b)
    next_a = [(x * a[i] + inverse * a[half + i]) % q for i in range(half)]
    next_b = [(inverse * b[i] + x * b[half + i]) % q for i in range(half)]
    return next_a, next_b


def fold_bases(g, h, x, q):
    inverse = _inverse(x, q)
    half = _length(g, h)
    _points(list(g) + list(h))
    next_g = [_combine(g[i], g[half + i], inverse, x, q) for i in range(half)]
    next_h = [_combine(h[i], h[half + i], x, inverse, q) for i in range(half)]
    return next_g, next_h


def fold_commitment(p, left, right, x, q):
    inverse = _inverse(x, q)
    _points([p, left, right])
    return tuple((pi + x * x * li + inverse * inverse * ri) % q
                 for pi, li, ri in zip(p, left, right))
