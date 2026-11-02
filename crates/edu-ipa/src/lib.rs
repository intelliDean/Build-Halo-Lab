//! An original educational implementation of Bulletproofs §3, Protocols 1 and 2.
//!
//! This crate implements commitments, the Fiat–Shamir transcript, the prover,
//! verifier, and canonical wire format. `curve25519-dalek` supplies Ristretto255
//! group/scalar arithmetic. It does **not** supply the proof implementation.
//! There is no blinding: this argument and its commitments are NOT hiding or
//! zero knowledge. In particular, length one exposes both witness scalars.
//! This crate is unaudited educational software, not production cryptography.

pub use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::{
    ristretto::{CompressedRistretto, RistrettoPoint},
    traits::Identity,
};
use sha2::{Digest, Sha512};
use std::{error::Error as StdError, fmt};

/// Resource bound for this educational wire format, inclusive.
pub const MAX_VECTOR_LEN: usize = 4096;
pub const DEFAULT_CONTEXT: &[u8] = b"Build Halo Lab educational IPA v1";
const PROOF_MAGIC: &[u8; 8] = b"BHIPA001";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidLength,
    DimensionMismatch,
    InvalidEncoding,
    WitnessMismatch,
    VerificationFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLength => "length must be a power of two in 1..=4096",
            Self::DimensionMismatch => "witness, parameters, and proof dimensions differ",
            Self::InvalidEncoding => "noncanonical, malformed, or unsupported IPA encoding",
            Self::WitnessMismatch => "witness does not open the expected public statement",
            Self::VerificationFailed => "inner-product verification equation failed",
        })
    }
}
impl StdError for Error {}

/// Deterministic transparent parameters. No generator is a known-scalar multiple
/// of a fixed basepoint: each uses the Ristretto map of a separate SHA-512 output.
#[derive(Clone, Debug)]
pub struct Parameters {
    g: Vec<RistrettoPoint>,
    h: Vec<RistrettoPoint>,
    u: RistrettoPoint,
    id: [u8; 32],
}

impl Parameters {
    pub fn new(n: usize) -> Result<Self, Error> {
        Self::derive(n, DEFAULT_CONTEXT)
    }

    /// Bind dimensions and an application context into all generator derivations.
    pub fn derive(n: usize, context: &[u8]) -> Result<Self, Error> {
        check_len(n)?;
        let mut used = Vec::with_capacity(2 * n + 1);
        let mut derive = |role: &[u8], index: usize| {
            // Identity/collision rejection makes even the negligible corner case
            // explicit. The counter is part of the generator derivation.
            for counter in 0u64.. {
                let mut hash = Sha512::new();
                frame(&mut hash, b"domain", b"build-halo-lab/ipa/generators/v1");
                frame(&mut hash, b"context", context);
                frame(&mut hash, b"n", &(n as u64).to_le_bytes());
                frame(&mut hash, b"role", role);
                frame(&mut hash, b"index", &(index as u64).to_le_bytes());
                frame(&mut hash, b"counter", &counter.to_le_bytes());
                let uniform: [u8; 64] = hash.finalize().into();
                let point = RistrettoPoint::from_uniform_bytes(&uniform);
                if point != RistrettoPoint::identity() && !used.contains(&point) {
                    used.push(point);
                    return point;
                }
            }
            unreachable!("generator derivation counter exhausted")
        };
        let g = (0..n).map(|i| derive(b"G", i)).collect::<Vec<_>>();
        let h = (0..n).map(|i| derive(b"H", i)).collect::<Vec<_>>();
        let u = derive(b"U", 0);
        let mut hash = Sha512::new();
        frame(&mut hash, b"domain", b"build-halo-lab/ipa/parameter-id/v1");
        frame(&mut hash, b"n", &(n as u64).to_le_bytes());
        frame(&mut hash, b"context", context);
        for point in g.iter().chain(h.iter()).chain(std::iter::once(&u)) {
            frame(&mut hash, b"point", point.compress().as_bytes());
        }
        let digest = hash.finalize();
        let mut id = [0; 32];
        id.copy_from_slice(&digest[..32]);
        Ok(Self { g, h, u, id })
    }

    pub fn len(&self) -> usize {
        self.g.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn id(&self) -> [u8; 32] {
        self.id
    }

    /// Commit to both vectors and state their scalar inner product. There is no
    /// random blinding term; never use this commitment to conceal small secrets.
    pub fn commit(&self, a: &[Scalar], b: &[Scalar]) -> Result<Statement, Error> {
        self.check_dimensions(a, b)?;
        Ok(Statement {
            commitment: vector_commitment(a, &self.g, b, &self.h)
                .compress()
                .to_bytes(),
            inner_product: dot(a, b).to_bytes(),
        })
    }

    fn check_dimensions(&self, a: &[Scalar], b: &[Scalar]) -> Result<(), Error> {
        if a.len() != self.len() || b.len() != self.len() {
            Err(Error::DimensionMismatch)
        } else {
            Ok(())
        }
    }
}

/// The verifier supplies this expected statement independently of the proof.
/// Fields are byte arrays so callers can store them without an additional codec;
/// `verify` validates their canonical encodings even when constructed directly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub commitment: [u8; 32],
    pub inner_product: [u8; 32],
}

impl Statement {
    pub fn to_bytes(&self) -> [u8; 64] {
        let mut out = [0; 64];
        out[..32].copy_from_slice(&self.commitment);
        out[32..].copy_from_slice(&self.inner_product);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 64 {
            return Err(Error::InvalidEncoding);
        }
        let mut commitment = [0; 32];
        commitment.copy_from_slice(&bytes[..32]);
        let mut inner_product = [0; 32];
        inner_product.copy_from_slice(&bytes[32..]);
        let statement = Self {
            commitment,
            inner_product,
        };
        statement.decode()?;
        Ok(statement)
    }

    fn decode(&self) -> Result<(RistrettoPoint, Scalar), Error> {
        Ok((
            decode_point(self.commitment)?,
            decode_scalar(self.inner_product)?,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct Proof {
    n: usize,
    rounds: Vec<(RistrettoPoint, RistrettoPoint)>,
    a: Scalar,
    b: Scalar,
}

impl Proof {
    pub fn vector_len(&self) -> usize {
        self.n
    }

    /// BHIPA001 || n:u32-LE || (L:32 || R:32)*log2(n) || a:32 || b:32.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(76 + 64 * self.rounds.len());
        out.extend_from_slice(PROOF_MAGIC);
        out.extend_from_slice(&(self.n as u32).to_le_bytes());
        for (l, r) in &self.rounds {
            out.extend_from_slice(l.compress().as_bytes());
            out.extend_from_slice(r.compress().as_bytes());
        }
        out.extend_from_slice(self.a.as_bytes());
        out.extend_from_slice(self.b.as_bytes());
        out
    }

    /// Strict decoder: reject aliases, trailing bytes, bad points/scalars,
    /// unsupported versions, and dimensions before allocating round storage.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 12 || &bytes[..8] != PROOF_MAGIC {
            return Err(Error::InvalidEncoding);
        }
        let n = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        check_len(n).map_err(|_| Error::InvalidEncoding)?;
        let round_count = n.ilog2() as usize;
        if bytes.len() != 76 + 64 * round_count {
            return Err(Error::InvalidEncoding);
        }
        let mut offset = 12;
        let mut read = || {
            let mut array = [0; 32];
            array.copy_from_slice(&bytes[offset..offset + 32]);
            offset += 32;
            array
        };
        let mut rounds = Vec::with_capacity(round_count);
        for _ in 0..round_count {
            rounds.push((decode_point(read())?, decode_point(read())?));
        }
        let a = decode_scalar(read())?;
        let b = decode_scalar(read())?;
        Ok(Self { n, rounds, a, b })
    }
}

/// Check that the supplied witness opens the expected statement, then prove it.
pub fn prove(
    params: &Parameters,
    statement: &Statement,
    a: &[Scalar],
    b: &[Scalar],
) -> Result<Proof, Error> {
    if params.commit(a, b)? != *statement {
        return Err(Error::WitnessMismatch);
    }
    prove_inner(params, statement, a, b)
}

fn prove_inner(
    params: &Parameters,
    statement: &Statement,
    a: &[Scalar],
    b: &[Scalar],
) -> Result<Proof, Error> {
    params.check_dimensions(a, b)?;
    statement.decode()?;
    let mut transcript = Transcript::new(params, statement);
    // Protocol 1: after binding C,c and parameters, randomize U to enforce c.
    let q = params.u * transcript.challenge(b"inner-product-binding");
    let (mut a, mut b) = (a.to_vec(), b.to_vec());
    let (mut g, mut h) = (params.g.clone(), params.h.clone());
    let mut rounds = Vec::with_capacity(params.len().ilog2() as usize);
    while a.len() > 1 {
        let half = a.len() / 2;
        let (al, ar) = a.split_at(half);
        let (bl, br) = b.split_at(half);
        let (gl, gr) = g.split_at(half);
        let (hl, hr) = h.split_at(half);
        let l = vector_commitment(al, gr, br, hl) + q * dot(al, br);
        let r = vector_commitment(ar, gl, bl, hr) + q * dot(ar, bl);
        transcript.round(rounds.len(), &l, &r);
        let x = transcript.challenge(b"fold");
        let inverse = x.invert();
        let next_a = (0..half).map(|i| x * al[i] + inverse * ar[i]).collect();
        let next_b = (0..half).map(|i| inverse * bl[i] + x * br[i]).collect();
        let (next_g, next_h) = fold_generators(&g, &h, x, inverse);
        (a, b, g, h) = (next_a, next_b, next_g, next_h);
        rounds.push((l, r));
    }
    Ok(Proof {
        n: params.len(),
        rounds,
        a: a[0],
        b: b[0],
    })
}

/// Verify against the caller's expected C,c and expected parameters. No witness
/// vectors or transcript challenges are received from the prover.
pub fn verify(params: &Parameters, statement: &Statement, proof: &Proof) -> Result<(), Error> {
    if proof.n != params.len() || proof.rounds.len() != params.len().ilog2() as usize {
        return Err(Error::DimensionMismatch);
    }
    let (commitment, c) = statement.decode()?;
    let mut transcript = Transcript::new(params, statement);
    let q = params.u * transcript.challenge(b"inner-product-binding");
    let mut p = commitment + q * c;
    let (mut g, mut h) = (params.g.clone(), params.h.clone());
    for (round, (l, r)) in proof.rounds.iter().enumerate() {
        transcript.round(round, l, r);
        let x = transcript.challenge(b"fold");
        let inverse = x.invert();
        p += l * (x * x) + r * (inverse * inverse);
        (g, h) = fold_generators(&g, &h, x, inverse);
    }
    let expected = g[0] * proof.a + h[0] * proof.b + q * (proof.a * proof.b);
    if expected == p {
        Ok(())
    } else {
        Err(Error::VerificationFailed)
    }
}

/// A reusable pure helper for the assessed folding exercise.
pub fn inner_product(a: &[Scalar], b: &[Scalar]) -> Result<Scalar, Error> {
    if a.len() != b.len() {
        return Err(Error::DimensionMismatch);
    }
    Ok(dot(a, b))
}

fn check_len(n: usize) -> Result<(), Error> {
    if n == 0 || !n.is_power_of_two() || n > MAX_VECTOR_LEN {
        Err(Error::InvalidLength)
    } else {
        Ok(())
    }
}

fn dot(a: &[Scalar], b: &[Scalar]) -> Scalar {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

fn vector_commitment(
    a: &[Scalar],
    g: &[RistrettoPoint],
    b: &[Scalar],
    h: &[RistrettoPoint],
) -> RistrettoPoint {
    a.iter()
        .zip(g)
        .chain(b.iter().zip(h))
        .fold(RistrettoPoint::identity(), |sum, (scalar, point)| {
            sum + point * scalar
        })
}

fn fold_generators(
    g: &[RistrettoPoint],
    h: &[RistrettoPoint],
    x: Scalar,
    inverse: Scalar,
) -> (Vec<RistrettoPoint>, Vec<RistrettoPoint>) {
    let half = g.len() / 2;
    (
        (0..half)
            .map(|i| g[i] * inverse + g[i + half] * x)
            .collect(),
        (0..half)
            .map(|i| h[i] * x + h[i + half] * inverse)
            .collect(),
    )
}

fn decode_scalar(bytes: [u8; 32]) -> Result<Scalar, Error> {
    Option::<Scalar>::from(Scalar::from_canonical_bytes(bytes)).ok_or(Error::InvalidEncoding)
}

fn decode_point(bytes: [u8; 32]) -> Result<RistrettoPoint, Error> {
    let point = CompressedRistretto(bytes)
        .decompress()
        .ok_or(Error::InvalidEncoding)?;
    if point.compress().to_bytes() != bytes {
        return Err(Error::InvalidEncoding);
    }
    Ok(point)
}

// Every message is unambiguously framed: label length, label, value length,
// value. Lengths are u64 little endian. No concatenation ambiguity is possible.
fn frame(hash: &mut Sha512, label: &[u8], value: &[u8]) {
    hash.update((label.len() as u64).to_le_bytes());
    hash.update(label);
    hash.update((value.len() as u64).to_le_bytes());
    hash.update(value);
}

struct Transcript(Sha512);
impl Transcript {
    fn new(params: &Parameters, statement: &Statement) -> Self {
        let mut hash = Sha512::new();
        frame(&mut hash, b"domain", b"build-halo-lab/ipa/transcript/v1");
        frame(&mut hash, b"parameters", &params.id);
        frame(&mut hash, b"n", &(params.len() as u64).to_le_bytes());
        frame(&mut hash, b"commitment", &statement.commitment);
        frame(&mut hash, b"inner-product", &statement.inner_product);
        Self(hash)
    }

    fn round(&mut self, round: usize, l: &RistrettoPoint, r: &RistrettoPoint) {
        frame(&mut self.0, b"round", &(round as u64).to_le_bytes());
        frame(&mut self.0, b"L", l.compress().as_bytes());
        frame(&mut self.0, b"R", r.compress().as_bytes());
    }

    fn challenge(&mut self, label: &[u8]) -> Scalar {
        for counter in 0u64.. {
            let mut candidate = self.0.clone();
            frame(&mut candidate, b"challenge-label", label);
            frame(&mut candidate, b"challenge-counter", &counter.to_le_bytes());
            let wide: [u8; 64] = candidate.finalize().into();
            let scalar = Scalar::from_bytes_mod_order_wide(&wide);
            if scalar != Scalar::ZERO {
                frame(&mut self.0, b"challenge-label", label);
                frame(&mut self.0, b"challenge-counter", &counter.to_le_bytes());
                frame(&mut self.0, b"challenge-value", scalar.as_bytes());
                return scalar;
            }
        }
        unreachable!("challenge counter exhausted")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn witness(n: usize) -> (Vec<Scalar>, Vec<Scalar>) {
        (
            (1..=n as u64).map(Scalar::from).collect(),
            (11..11 + n as u64).map(Scalar::from).collect(),
        )
    }

    #[test]
    fn honest_lengths_one_two_four_eight_and_sixty_four() {
        for n in [1, 2, 4, 8, 64] {
            let params = Parameters::new(n).unwrap();
            let (a, b) = witness(n);
            let statement = params.commit(&a, &b).unwrap();
            let proof = prove(&params, &statement, &a, &b).unwrap();
            verify(&params, &statement, &proof).unwrap();
            let bytes = proof.to_bytes();
            assert_eq!(bytes.len(), 76 + 64 * n.ilog2() as usize);
            let decoded = Proof::from_bytes(&bytes).unwrap();
            assert_eq!(decoded.to_bytes(), bytes);
            verify(&params, &statement, &decoded).unwrap();
            assert_eq!(
                Statement::from_bytes(&statement.to_bytes()).unwrap(),
                statement
            );
        }
    }

    #[test]
    fn zero_witness_and_identity_commitment_are_valid() {
        let params = Parameters::new(8).unwrap();
        let a = vec![Scalar::ZERO; 8];
        let statement = params.commit(&a, &a).unwrap();
        let proof = prove(&params, &statement, &a, &a).unwrap();
        verify(&params, &statement, &proof).unwrap();
    }

    #[test]
    fn altered_inner_product_rejected_by_proof_equation() {
        for n in [1, 2, 8] {
            let params = Parameters::new(n).unwrap();
            let (a, b) = witness(n);
            let mut statement = params.commit(&a, &b).unwrap();
            let proof = prove(&params, &statement, &a, &b).unwrap();
            statement.inner_product = (dot(&a, &b) + Scalar::ONE).to_bytes();
            assert_eq!(
                verify(&params, &statement, &proof),
                Err(Error::VerificationFailed)
            );
            // Bypass the honest prover's precheck: the verifier still rejects an
            // algebraically constructed proof using a witness for a false c.
            let malicious = prove_inner(&params, &statement, &a, &b).unwrap();
            assert_eq!(
                verify(&params, &statement, &malicious),
                Err(Error::VerificationFailed)
            );
        }
    }

    #[test]
    fn altered_commitment_rejected_by_proof_equation() {
        let params = Parameters::new(4).unwrap();
        let (a, b) = witness(4);
        let mut statement = params.commit(&a, &b).unwrap();
        let proof = prove(&params, &statement, &a, &b).unwrap();
        statement.commitment = (decode_point(statement.commitment).unwrap() + params.u)
            .compress()
            .to_bytes();
        assert_eq!(
            verify(&params, &statement, &proof),
            Err(Error::VerificationFailed)
        );
        let malicious = prove_inner(&params, &statement, &a, &b).unwrap();
        assert_eq!(
            verify(&params, &statement, &malicious),
            Err(Error::VerificationFailed)
        );
    }

    #[test]
    fn wrong_parameters_and_wrong_dimension_rejected() {
        let params = Parameters::new(8).unwrap();
        let (a, b) = witness(8);
        let statement = params.commit(&a, &b).unwrap();
        let proof = prove(&params, &statement, &a, &b).unwrap();
        let wrong = Parameters::derive(8, b"another application").unwrap();
        assert_eq!(
            verify(&wrong, &statement, &proof),
            Err(Error::VerificationFailed)
        );
        assert_eq!(
            verify(&Parameters::new(4).unwrap(), &statement, &proof),
            Err(Error::DimensionMismatch)
        );
    }

    #[test]
    fn modified_round_and_final_scalars_rejected() {
        let params = Parameters::new(8).unwrap();
        let (a, b) = witness(8);
        let statement = params.commit(&a, &b).unwrap();
        let proof = prove(&params, &statement, &a, &b).unwrap();
        let mut changed = proof.clone();
        changed.rounds[1].0 += params.u;
        assert_eq!(
            verify(&params, &statement, &changed),
            Err(Error::VerificationFailed)
        );
        let mut changed = proof.clone();
        changed.rounds[0] = (changed.rounds[0].1, changed.rounds[0].0);
        assert_eq!(
            verify(&params, &statement, &changed),
            Err(Error::VerificationFailed)
        );
        let mut changed = proof.clone();
        changed.a += Scalar::ONE;
        assert_eq!(
            verify(&params, &statement, &changed),
            Err(Error::VerificationFailed)
        );
        let mut changed = proof;
        changed.b += Scalar::ONE;
        assert_eq!(
            verify(&params, &statement, &changed),
            Err(Error::VerificationFailed)
        );
    }

    #[test]
    fn malformed_and_noncanonical_encodings_rejected() {
        let params = Parameters::new(2).unwrap();
        let (a, b) = witness(2);
        let statement = params.commit(&a, &b).unwrap();
        let bytes = prove(&params, &statement, &a, &b).unwrap().to_bytes();
        for len in 0..bytes.len() {
            assert!(Proof::from_bytes(&bytes[..len]).is_err());
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(Proof::from_bytes(&bad).is_err());
        let mut bad = bytes.clone();
        bad[0] ^= 1;
        assert!(Proof::from_bytes(&bad).is_err());
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&3u32.to_le_bytes());
        assert!(Proof::from_bytes(&bad).is_err());
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(Proof::from_bytes(&bad).is_err());
        let mut bad = bytes.clone();
        bad[12..44].fill(255);
        assert!(Proof::from_bytes(&bad).is_err());
        let mut bad = bytes;
        let len = bad.len();
        bad[len - 32..].fill(255);
        assert!(Proof::from_bytes(&bad).is_err());
        let mut bad = statement.to_bytes();
        bad[32..].fill(255);
        assert!(Statement::from_bytes(&bad).is_err());
        let mut bad = statement;
        bad.commitment.fill(255);
        let proof = prove(&params, &params.commit(&a, &b).unwrap(), &a, &b).unwrap();
        assert_eq!(verify(&params, &bad, &proof), Err(Error::InvalidEncoding));
    }

    #[test]
    fn lengths_and_witness_opening_are_checked() {
        for n in [0, 3, 7, 8192] {
            assert!(Parameters::new(n).is_err());
        }
        let params = Parameters::new(4).unwrap();
        let (mut a, b) = witness(4);
        let statement = params.commit(&a, &b).unwrap();
        a[0] += Scalar::ONE;
        assert_eq!(
            prove(&params, &statement, &a, &b).unwrap_err(),
            Error::WitnessMismatch
        );
        assert_eq!(
            params.commit(&a[..2], &b).unwrap_err(),
            Error::DimensionMismatch
        );
        assert_eq!(inner_product(&a[..2], &b), Err(Error::DimensionMismatch));
    }

    #[test]
    fn parameter_derivation_and_framing_are_domain_separated() {
        let params = Parameters::new(4).unwrap();
        assert_eq!(params.id(), Parameters::new(4).unwrap().id());
        assert_ne!(
            params.id(),
            Parameters::derive(4, b"different").unwrap().id()
        );
        assert_ne!(params.id(), Parameters::new(8).unwrap().id());
        let points = params
            .g
            .iter()
            .chain(&params.h)
            .chain(std::iter::once(&params.u))
            .collect::<Vec<_>>();
        for (i, point) in points.iter().enumerate() {
            assert_ne!(**point, RistrettoPoint::identity());
            for other in &points[i + 1..] {
                assert_ne!(point, other);
            }
        }
        let mut first = Sha512::new();
        frame(&mut first, b"ab", b"c");
        let mut second = Sha512::new();
        frame(&mut second, b"a", b"bc");
        assert_ne!(first.finalize(), second.finalize());
    }
}
