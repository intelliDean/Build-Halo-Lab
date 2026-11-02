use std::{collections::BTreeMap, env, fs, io::Read, path::Path, process, time::Instant};

use edu_ipa::{Parameters, Scalar};
use recursive_demo::{ChainLab, Statement};
use serde_json::json;

// Matches the pinned Ragu transport decoder's limit. Read only one extra byte
// to detect an oversized file, even if it grows after opening or lacks metadata.
const MAX_PROOF_BYTES: usize = 128 * 1024;

fn read_proof(reader: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(MAX_PROOF_BYTES + 1);
    reader
        .take((MAX_PROOF_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_PROOF_BYTES {
        return Err("proof file exceeds 128 KiB limit".into());
    }
    Ok(bytes)
}

const HELP: &str = "Build Halo Lab — educational, non-production cryptography\n\
Usage:\n\
  lab-cli ipa-demo [--length 8]\n\
  lab-cli evaluate --initial 2 --steps 8\n\
  lab-cli prove-chain --initial 2 --steps 8 --out chain.bin\n\
  lab-cli verify-chain --initial 2 --final 0x... --steps 8 --proof chain.bin\n\
The verifier requires all three expected public inputs independently.\n\
Fields accept a u64 decimal or 0x followed by 32 canonical little-endian bytes.\n\
The original IPA plus Ragu demonstration is not an independently implemented complete Halo system.";

fn options(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>, String> {
    let mut args = args;
    let mut result = BTreeMap::new();
    while let Some(key) = args.next() {
        if !key.starts_with("--") {
            return Err(format!("expected named option, got {key}"));
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {key}"))?;
        if value.starts_with("--") || result.insert(key.clone(), value).is_some() {
            return Err(format!("missing value or duplicate option: {key}"));
        }
    }
    Ok(result)
}

fn take(options: &mut BTreeMap<String, String>, key: &str) -> Result<String, String> {
    options
        .remove(key)
        .ok_or_else(|| format!("required option {key}"))
}

fn steps(options: &mut BTreeMap<String, String>) -> Result<u32, String> {
    take(options, "--steps")?
        .parse()
        .map_err(|_| "invalid --steps".into())
}

fn no_extras(options: &BTreeMap<String, String>) -> Result<(), String> {
    if options.is_empty() {
        Ok(())
    } else {
        Err(format!("unknown options: {:?}", options.keys()))
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "--help".into());
    if command == "--help" || command == "help" {
        println!("{HELP}");
        return Ok(());
    }
    let mut opts = options(args)?;
    match command.as_str() {
        "ipa-demo" => {
            let n: usize = opts
                .remove("--length")
                .unwrap_or_else(|| "8".into())
                .parse()
                .map_err(|_| "invalid --length")?;
            no_extras(&opts)?;
            let setup_start = Instant::now();
            let params =
                Parameters::derive(n, b"Build Halo Lab CLI v1").map_err(|e| format!("{e:?}"))?;
            let a: Vec<_> = (0..n).map(|i| Scalar::from(i as u64 + 1)).collect();
            let b: Vec<_> = (0..n).map(|i| Scalar::from(2 * i as u64 + 1)).collect();
            let statement = params.commit(&a, &b).map_err(|e| format!("{e:?}"))?;
            let setup_ms = setup_start.elapsed().as_secs_f64() * 1000.0;
            let start = Instant::now();
            let proof =
                edu_ipa::prove(&params, &statement, &a, &b).map_err(|e| format!("{e:?}"))?;
            let proving_ms = start.elapsed().as_secs_f64() * 1000.0;
            let encoded = proof.to_bytes();
            let decoded = edu_ipa::Proof::from_bytes(&encoded).map_err(|e| format!("{e:?}"))?;
            let start = Instant::now();
            edu_ipa::verify(&params, &statement, &decoded).map_err(|e| format!("{e:?}"))?;
            let verification_ms = start.elapsed().as_secs_f64() * 1000.0;
            println!(
                "{}",
                json!({"command":"ipa-demo","length":n,"verified":true,
                "proof_bytes":encoded.len(),"setup_ms":setup_ms,"proving_ms":proving_ms,
                "verification_ms":verification_ms,"hiding":false})
            );
        }
        "evaluate" => {
            let initial = take(&mut opts, "--initial")?;
            let count = steps(&mut opts)?;
            no_extras(&opts)?;
            println!(
                "{}",
                serde_json::to_string(&recursive_demo::evaluate(&initial, count)?)
                    .map_err(|e| e.to_string())?
            );
        }
        "prove-chain" => {
            let initial = take(&mut opts, "--initial")?;
            let count = steps(&mut opts)?;
            let output = take(&mut opts, "--out")?;
            no_extras(&opts)?;
            let setup_start = Instant::now();
            let lab = ChainLab::new()?;
            let setup_ms = setup_start.elapsed().as_secs_f64() * 1000.0;
            let start = Instant::now();
            let (proof, statement) = lab.prove(&initial, count)?;
            let proving_ms = start.elapsed().as_secs_f64() * 1000.0;
            if let Some(parent) = Path::new(&output)
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
            {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::write(&output, &proof).map_err(|e| e.to_string())?;
            println!(
                "{}",
                json!({"command":"prove-chain","statement":statement,
                "proof_bytes":proof.len(),"setup_ms":setup_ms,"proving_ms":proving_ms,
                "includes_terminal_compression":true,"proof_file":output})
            );
        }
        "verify-chain" => {
            let expected = Statement {
                initial: take(&mut opts, "--initial")?,
                final_value: take(&mut opts, "--final")?,
                steps: steps(&mut opts)?,
            };
            let input = take(&mut opts, "--proof")?;
            no_extras(&opts)?;
            let bytes = read_proof(fs::File::open(input).map_err(|e| e.to_string())?)?;
            let setup_start = Instant::now();
            let lab = ChainLab::new()?;
            let setup_ms = setup_start.elapsed().as_secs_f64() * 1000.0;
            let start = Instant::now();
            if !lab.verify(&bytes, &expected)? {
                return Err("proof rejected for expected public statement".into());
            }
            let verification_ms = start.elapsed().as_secs_f64() * 1000.0;
            println!(
                "{}",
                json!({"command":"verify-chain","verified":true,
                "statement":expected,"proof_bytes":bytes.len(),"setup_ms":setup_ms,
                "verification_ms":verification_ms,"terminal_accumulator_checks":true})
            );
        }
        _ => return Err(format!("unknown command: {command}\n{HELP}")),
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    #[test]
    fn proof_reader_accepts_empty_small_and_exact_limit() {
        for length in [0, 17, MAX_PROOF_BYTES] {
            let expected = vec![42; length];
            assert_eq!(read_proof(Cursor::new(&expected)).unwrap(), expected);
        }
    }

    #[test]
    fn oversized_proof_read_stops_after_limit_plus_one() {
        let mut source = Cursor::new(vec![0; MAX_PROOF_BYTES * 4]);
        assert_eq!(
            read_proof(&mut source).unwrap_err(),
            "proof file exceeds 128 KiB limit"
        );
        assert_eq!(source.position(), (MAX_PROOF_BYTES + 1) as u64);
    }

    #[test]
    fn proof_reader_propagates_io_errors() {
        struct BrokenReader;
        impl Read for BrokenReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("injected read failure"))
            }
        }
        assert_eq!(
            read_proof(BrokenReader).unwrap_err(),
            "injected read failure"
        );
    }
}
