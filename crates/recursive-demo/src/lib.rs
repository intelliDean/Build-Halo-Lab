//! Original educational state-machine circuit backed by Ragu's recursive PCD.
//! Cryptographic proving, compression, and terminal verification are reused Ragu.
use ragu_circuits::polynomials::ProductionRank;
use ragu_core::{
    Result as RaguResult,
    drivers::{Driver, DriverValue},
    gadgets::{Bound, Kind},
    maybe::Maybe,
    pasta::{Fp, Pasta},
};
use ragu_pcd::{
    Application, ApplicationBuilder, CompressedProof,
    header::{Header, Suffix},
    step::{Encoded, Index, Step},
};
use ragu_primitives::{
    Element,
    allocator::{Allocator, Standard},
};
use rand::{SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use udon::field::Field;
pub const MAX_STEPS: u32 = 8;
const HEADER_SIZE: usize = 4;
type App = Application<'static, Pasta, ProductionRank, HEADER_SIZE>;
type Data = [Fp; 3];
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statement {
    pub initial: String,
    pub final_value: String,
    pub steps: u32,
}
pub struct ChainState;
impl Header<Fp> for ChainState {
    const SUFFIX: Suffix = Suffix::new(0);
    type Data = Data;
    type Output = [core::marker::PhantomData<Kind![Fp; Element<'_, _>]>; 3];
    fn encode<'dr, D: Driver<'dr, F = Fp>, A: Allocator<'dr, D>>(
        dr: &mut D,
        a: &mut A,
        w: DriverValue<D, Data>,
    ) -> RaguResult<Bound<'dr, D, Self::Output>> {
        Ok([
            Element::alloc(dr, a, w.as_ref().map(|v| v[0]))?,
            Element::alloc(dr, a, w.as_ref().map(|v| v[1]))?,
            Element::alloc(dr, a, w.map(|v| v[2]))?,
        ])
    }
}
fn base_constraints<'dr, D: Driver<'dr, F = Fp>>(
    dr: &mut D,
    out: &[Element<'dr, D>; 3],
) -> RaguResult<()> {
    dr.enforce_equal(out[0].wire(), out[1].wire())?;
    out[2].enforce_zero(dr)
}
fn transition_constraints<'dr, D: Driver<'dr, F = Fp>>(
    dr: &mut D,
    left: &[Element<'dr, D>; 3],
    out: &[Element<'dr, D>; 3],
) -> RaguResult<()> {
    dr.enforce_equal(left[0].wire(), out[0].wire())?;
    let three = Element::constant(dr, Fp::from(3));
    let next = left[1].square(dr)?.add(dr, &three);
    dr.enforce_equal(next.wire(), out[1].wire())?;
    let count = left[2].add(dr, &Element::one());
    dr.enforce_equal(count.wire(), out[2].wire())?;
    // Range constraint is in the proof: product_{i=1}^{8}(count-i)=0.
    let mut product = Element::one();
    for i in 1..=MAX_STEPS {
        let k = Element::constant(dr, Fp::from(i as u64));
        let term = out[2].sub(dr, &k);
        product = product.mul(dr, &term)?;
    }
    product.enforce_zero(dr)
}
pub struct Seed;
pub struct Advance;
macro_rules! step_impl {
    ($ty:ty,$index:expr,$left:ty,$right:ty,$check:ident) => {
        impl Step<Pasta> for $ty {
            const INDEX: Index = Index::new($index);
            type Witness<'source> = Data;
            type Aux<'source> = ();
            type Left = $left;
            type Right = $right;
            type Output = ChainState;
            fn witness<'dr, 'source: 'dr, D: Driver<'dr, F = Fp>, const HS: usize>(
                &self,
                dr: &mut D,
                w: DriverValue<D, Data>,
                left: DriverValue<D, <Self::Left as Header<Fp>>::Data>,
                right: DriverValue<D, <Self::Right as Header<Fp>>::Data>,
            ) -> RaguResult<(
                (
                    Encoded<'dr, D, Self::Left, HS>,
                    Encoded<'dr, D, Self::Right, HS>,
                    Encoded<'dr, D, ChainState, HS>,
                ),
                DriverValue<D, Data>,
                DriverValue<D, ()>,
            )>
            where
                Self: 'dr,
            {
                let data = w.as_ref().map(|v| *v);
                let a = &mut Standard::new();
                let left = Encoded::new(dr, a, left)?;
                let right = Encoded::new(dr, a, right)?;
                let out = Encoded::new(dr, a, w)?;
                $check(dr, left.as_gadget(), out.as_gadget())?;
                Ok(((left, right, out), data, D::unit()))
            }
        }
    };
}
fn seed_check<'dr, D: Driver<'dr, F = Fp>>(
    dr: &mut D,
    _: &(),
    out: &[Element<'dr, D>; 3],
) -> RaguResult<()> {
    base_constraints(dr, out)
}
step_impl!(Seed, 0, (), (), seed_check);
step_impl!(Advance, 1, ChainState, ChainState, transition_constraints);
/// Rebuilds a fixed educational Ragu application. Fixed registry tags are insecure test parameters.
pub struct ChainLab {
    app: App,
}
impl ChainLab {
    pub fn new() -> Result<Self, String> {
        let app = ApplicationBuilder::<Pasta, ProductionRank, HEADER_SIZE>::new()
            .register(Seed)
            .map_err(err)?
            .register(Advance)
            .map_err(err)?
            .finalize(ragu_pcd::pasta::baked())
            .map_err(err)?;
        Ok(Self { app })
    }
    /// Returns one compressed recursive proof without intermediate history.
    pub fn prove(&self, initial: &str, steps: u32) -> Result<(Vec<u8>, Statement), String> {
        if !(1..=MAX_STEPS).contains(&steps) {
            return Err("steps must be in 1..=8".into());
        }
        let initial = parse_field(initial)?;
        let mut rng = StdRng::from_rng(&mut rand::rng());
        let (mut pcd, _) = self
            .app
            .seed(&mut rng, Seed, [initial, initial, Fp::ZERO])
            .map_err(err)?;
        let mut current = initial;
        for step in 1..=steps {
            current = current.square() + Fp::from(3);
            (pcd, _) = self
                .app
                .fuse(
                    &mut rng,
                    Advance,
                    [initial, current, Fp::from(step as u64)],
                    pcd.clone(),
                    pcd,
                )
                .map_err(err)?;
        }
        let compressed = self.app.compress(&pcd, &mut rng).map_err(err)?;
        Ok((
            compressed.proof().to_canonical_bytes(),
            Statement {
                initial: format_field(initial),
                final_value: format_field(current),
                steps,
            },
        ))
    }
    /// Runs the full terminal decider, including both IPA MSM evaluations.
    pub fn verify(&self, bytes: &[u8], expected: &Statement) -> Result<bool, String> {
        if !(1..=MAX_STEPS).contains(&expected.steps) {
            return Ok(false);
        }
        let data = [
            parse_field(&expected.initial)?,
            parse_field(&expected.final_value)?,
            Fp::from(expected.steps as u64),
        ];
        let proof =
            match CompressedProof::<Pasta>::from_canonical_bytes::<ProductionRank, HEADER_SIZE>(
                bytes,
            ) {
                Ok(p) => p,
                Err(_) => return Ok(false),
            };
        self.app
            .verify_compressed(&proof.carry::<ChainState>(data))
            .map_err(err)
    }
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn format_field(value: Fp) -> String {
    format!("0x{}", hex::encode(value.to_bytes()))
}
pub fn parse_field(text: &str) -> Result<Fp, String> {
    if let Some(hex) = text.strip_prefix("0x") {
        let bytes = hex::decode(hex).map_err(err)?;
        let bytes: [u8; 32] = bytes
            .try_into()
            .map_err(|_| "field hex must contain exactly 32 bytes")?;
        Fp::from_bytes(bytes).ok_or_else(|| "noncanonical field element".into())
    } else {
        text.parse::<u64>().map(Fp::from).map_err(err)
    }
}
pub fn evaluate(initial: &str, steps: u32) -> Result<Statement, String> {
    if !(1..=MAX_STEPS).contains(&steps) {
        return Err("steps must be in 1..=8".into());
    }
    let initial = parse_field(initial)?;
    let mut current = initial;
    for _ in 0..steps {
        current = current.square() + Fp::from(3);
    }
    Ok(Statement {
        initial: format_field(initial),
        final_value: format_field(current),
        steps,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use ragu_primitives::Simulator;
    fn base(data: Data) -> bool {
        Simulator::<Fp>::simulate(data, |dr, w| {
            let out = ChainState::encode(dr, &mut Standard::new(), w)?;
            base_constraints(dr, &out)
        })
        .is_ok()
    }
    fn transition(left: Data, out: Data) -> bool {
        Simulator::<Fp>::simulate((left, out), |dr, w| {
            let left = ChainState::encode(dr, &mut Standard::new(), w.as_ref().map(|v| v.0))?;
            let out = ChainState::encode(dr, &mut Standard::new(), w.map(|v| v.1))?;
            transition_constraints(dr, &left, &out)
        })
        .is_ok()
    }
    fn f(a: u64, b: u64, c: u64) -> Data {
        [Fp::from(a), Fp::from(b), Fp::from(c)]
    }
    #[test]
    fn base_reset_and_nonzero_count_fail_constraints() {
        assert!(base(f(2, 2, 0)));
        assert!(!base(f(2, 3, 0)));
        assert!(!base(f(2, 2, 1)));
    }
    #[test]
    fn intermediate_transition_and_continuity_fail_constraints() {
        assert!(transition(f(2, 2, 0), f(2, 7, 1)));
        assert!(!transition(f(2, 2, 0), f(2, 8, 1)));
        assert!(!transition(f(2, 2, 0), f(3, 7, 1)));
        assert!(!transition(f(2, 2, 0), f(2, 7, 0)));
    }
    #[test]
    fn ninth_step_fails_proof_constraint() {
        assert!(!transition(f(2, 2, 8), f(2, 7, 9)));
    }
    #[test]
    fn malformed_fields_fail() {
        assert!(parse_field("0x00").is_err());
        assert!(parse_field(&format!("0x{}", "ff".repeat(32))).is_err());
        assert_eq!(
            parse_field(&format_field(Fp::from(7))).unwrap(),
            Fp::from(7)
        );
    }
}

#[cfg(test)]
mod recursive_soundness_tests {
    use super::*;
    fn f(a: u64, b: u64, c: u64) -> Data {
        [Fp::from(a), Fp::from(b), Fp::from(c)]
    }
    #[test]
    fn recursive_predecessor_and_base_reset_attacks_are_rejected() {
        let lab = ChainLab::new().unwrap();
        let app = &lab.app;
        let mut rng = StdRng::seed_from_u64(0x48414c4f);
        let (seed, _) = app.seed(&mut rng, Seed, f(2, 2, 0)).unwrap();
        assert!(app.verify(&seed, &mut rng).unwrap(), "honest seed control");
        let (honest, _) = app
            .fuse(&mut rng, Advance, f(2, 7, 1), seed.clone(), seed.clone())
            .unwrap();
        assert!(
            app.verify(&honest, &mut rng).unwrap(),
            "honest predecessor control"
        );
        let compressed = app.compress(&honest, &mut rng).unwrap();
        let expected = evaluate("2", 1).unwrap();
        let bytes = compressed.proof().to_canonical_bytes();
        assert!(lab.verify(&bytes, &expected).unwrap());
        let decoded =
            CompressedProof::<Pasta>::from_canonical_bytes::<ProductionRank, HEADER_SIZE>(&bytes)
                .unwrap();
        assert_eq!(decoded.to_canonical_bytes(), bytes, "canonical roundtrip");
        assert!(!lab.verify(&bytes[..bytes.len() - 1], &expected).unwrap());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(!lab.verify(&trailing, &expected).unwrap());
        let mut wrong_rank = bytes.clone();
        wrong_rank[8] ^= 1;
        assert!(!lab.verify(&wrong_rank, &expected).unwrap());
        let mut noncanonical = bytes.clone();
        noncanonical[24..56].fill(255);
        assert!(!lab.verify(&noncanonical, &expected).unwrap());
        let mut bad_point = bytes.clone();
        bad_point[288..320].fill(255);
        assert!(!lab.verify(&bad_point, &expected).unwrap());
        let mut huge_length = bytes.clone();
        huge_length[284..288].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(!lab.verify(&huge_length, &expected).unwrap());
        let mut changed = bytes.clone();
        let last = changed.len() - 1;
        changed[last] ^= 1;
        assert!(!lab.verify(&changed, &expected).unwrap());
        let mut altered = expected.clone();
        altered.final_value = format_field(Fp::from(8));
        assert!(!lab.verify(&bytes, &altered).unwrap());
        let mut altered = expected.clone();
        altered.initial = format_field(Fp::from(3));
        assert!(!lab.verify(&bytes, &altered).unwrap());
        let mut altered = expected.clone();
        altered.steps = 2;
        assert!(!lab.verify(&bytes, &altered).unwrap());
        // The child proof is real but relabeled with false state. The parent's local
        // recurrence is correct for that false state, so only recursive binding rejects.
        let false_child = honest.proof().clone().carry::<ChainState>(f(2, 8, 1));
        assert!(!app.verify(&false_child, &mut rng).unwrap());
        let (parent, _) = app
            .fuse(&mut rng, Advance, f(2, 67, 2), false_child, seed.clone())
            .unwrap();
        assert!(
            !app.verify(&parent, &mut rng).unwrap(),
            "invalid predecessor must propagate"
        );
        let compressed_parent = app.compress(&parent, &mut rng).unwrap();
        assert!(
            !app.verify_compressed(&compressed_parent).unwrap(),
            "invalid predecessor terminal compressed check"
        );
        let (grandparent, _) = app
            .fuse(&mut rng, Advance, f(2, 4492, 3), parent, seed.clone())
            .unwrap();
        assert!(
            !app.verify(&grandparent, &mut rng).unwrap(),
            "invalid predecessor through two generations"
        );
        let (bad_transition, _) = app
            .fuse(&mut rng, Advance, f(2, 8, 1), seed.clone(), seed.clone())
            .unwrap();
        assert!(
            !app.verify(&bad_transition, &mut rng).unwrap(),
            "invalid transition proof constraints"
        );
        let (reset, _) = app.seed(&mut rng, Seed, f(2, 999, 0)).unwrap();
        assert!(
            !app.verify(&reset, &mut rng).unwrap(),
            "seed cannot reset initial state"
        );
        let (reset_count, _) = app.seed(&mut rng, Seed, f(2, 2, 1)).unwrap();
        assert!(
            !app.verify(&reset_count, &mut rng).unwrap(),
            "seed cannot start at count one"
        );
        let other = ApplicationBuilder::<Pasta, ProductionRank, HEADER_SIZE>::new()
            .with_registry_tags(ragu_pcd::RegistryTags {
                native: ragu_circuits::registry::Tag::new(Fp::from(123)),
                nested: ragu_circuits::registry::Tag::new(ragu_core::pasta::Fq::from(456)),
            })
            .register(Seed)
            .unwrap()
            .register(Advance)
            .unwrap()
            .finalize(ragu_pcd::pasta::baked())
            .unwrap();
        assert!(
            !other.verify_compressed(&compressed).unwrap(),
            "different application registry parameters"
        );
    }
}
