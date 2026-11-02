"""Run real CLI proofs and fresh verifier processes. No third-party Python packages."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import platform
import subprocess
import time

P = int("40000000000000000000000000000000224698fc094cf91b992d30ed00000001", 16)
PROOF_REJECTION = "error: proof rejected for expected public statement\n"
INVALID_LENGTH_REJECTION = "error: InvalidLength\n"
OVERSIZED_PROOF_REJECTION = "error: proof file exceeds 128 KiB limit\n"


def _json_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _nonfinite_json(value):
    raise ValueError(f"non-finite JSON number: {value}")


def validate_completed(arguments, completed, *, expected_stderr=None,
                       expected_statement=None):
    """Validate one CLI result, without launching processes or reading files.

    A rejection is a specific exit-1 application response, never an arbitrary
    failure. A success is one complete JSON object with the requested command
    and strict boolean verification flags. Caller-owned statements are checked
    independently of the process output, including each JSON value's type.
    """
    if not arguments or not isinstance(arguments[0], str):
        raise AssertionError("a requested CLI command is required")
    command = arguments[0]
    if expected_stderr is not None:
        if not isinstance(expected_stderr, str) or not expected_stderr:
            raise AssertionError("expected rejection must name exact nonempty stderr")
        if completed.returncode != 1:
            raise AssertionError(f"expected application rejection exit 1; got {completed.returncode}")
        if completed.stdout != "":
            raise AssertionError("expected rejection must have empty stdout")
        if completed.stderr != expected_stderr:
            raise AssertionError(f"unexpected rejection diagnostic: {completed.stderr!r}")
        return None

    if completed.returncode != 0:
        raise AssertionError(f"expected successful exit 0; got {completed.returncode}")
    if completed.stderr != "":
        raise AssertionError("successful command must have empty stderr")
    try:
        metrics = json.loads(completed.stdout, object_pairs_hook=_json_object,
                             parse_constant=_nonfinite_json)
    except (ValueError, TypeError) as error:
        raise AssertionError("successful stdout must be exactly one valid JSON object") from error
    if not isinstance(metrics, dict):
        raise AssertionError("successful stdout must be a JSON object")
    if metrics.get("command") != command:
        raise AssertionError("reported command differs from requested command")
    if command in ("ipa-demo", "verify-chain") and metrics.get("verified") is not True:
        raise AssertionError("successful verification requires verified: true")
    if command == "verify-chain":
        if metrics.get("terminal_accumulator_checks") is not True:
            raise AssertionError("successful chain verification requires terminal_accumulator_checks: true")
        if expected_statement is None:
            raise AssertionError("chain verifier success requires an independently expected statement")
    if expected_statement is not None:
        actual = metrics.get("statement")
        if (not isinstance(actual, dict) or actual.keys() != expected_statement.keys()
                or any(type(actual[key]) is not type(value) or actual[key] != value
                       for key, value in expected_statement.items())):
            raise AssertionError("reported statement differs from independently expected statement")
    return metrics


def field(value):
    return "0x" + value.to_bytes(32, "little").hex()


def expected(initial, count):
    value = initial
    for _ in range(count):
        value = (value * value + 3) % P
    return {"initial": field(initial), "final_value": field(value), "steps": count}


def hardware():
    result = {"os": platform.platform(), "machine": platform.machine(),
              "logical_cpus": os.cpu_count(), "cpu": platform.processor()}
    if os.name == "nt":
        import winreg
        with winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE,
                           r"HARDWARE\DESCRIPTION\System\CentralProcessor\0") as key:
            result["cpu"] = winreg.QueryValueEx(key, "ProcessorNameString")[0]
        class Memory(ctypes.Structure):
            _fields_ = [("length", ctypes.c_ulong), ("load", ctypes.c_ulong)] + [
                (name, ctypes.c_ulonglong) for name in
                ("total_phys", "avail_phys", "total_page", "avail_page", "total_virtual", "avail_virtual", "extended")]
        mem = Memory()
        mem.length = ctypes.sizeof(mem)
        if ctypes.windll.kernel32.GlobalMemoryStatusEx(ctypes.byref(mem)):
            result["ram_bytes"] = mem.total_phys
    elif Path("/proc/cpuinfo").exists():
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                result["cpu"] = line.split(":", 1)[1].strip()
                break
        for line in Path("/proc/meminfo").read_text().splitlines():
            if line.startswith("MemTotal:"):
                result["ram_bytes"] = int(line.split()[1]) * 1024
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", default="target/release/lab-cli.exe" if os.name == "nt" else "target/release/lab-cli")
    parser.add_argument("--output", default="evidence/generated")
    args = parser.parse_args()
    binary = str(Path(args.binary).resolve())
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    report = {"schema_version": 2,
              "gate_contract": {
                  "success": "exit 0, empty stderr, one JSON object, matching command; verification flags strictly true",
                  "chain_statement": "independently supplied and matched with exact value types",
                  "rejection": "exit 1, empty stdout, exact caller-specified stderr; crashes never count as rejection",
              },
              "hardware": hardware(), "mode": "release", "samples_per_case": 1,
              "timing_note": "Single real run, no statistical confidence claimed. CLI times separate setup; proving includes compression.",
              "started_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
              "passed": False, "runs": []}
    report_path = output / "acceptance.json"

    def save():
        report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    save()

    def run(arguments, *, expected_stderr=None, expected_statement=None):
        command = [binary] + [str(x) for x in arguments]
        start = time.perf_counter()
        completed = subprocess.run(command, capture_output=True, text=True, timeout=1800)
        entry = {"command": command, "returncode": completed.returncode,
                 "wall_ms": (time.perf_counter() - start) * 1000,
                 "stdout": completed.stdout, "stderr": completed.stderr,
                 "expected_success": expected_stderr is None,
                 "expected_stderr": expected_stderr, "contract_validated": False}
        report["runs"].append(entry)
        save()
        try:
            metrics = validate_completed(arguments, completed, expected_stderr=expected_stderr,
                                         expected_statement=expected_statement)
        except AssertionError as error:
            entry["validation_error"] = str(error)
            save()
            raise
        entry["contract_validated"] = True
        if metrics is not None:
            entry["metrics"] = metrics
        save()
        return metrics

    for n in [1, 2, 4, 8]:
        run(["ipa-demo", "--length", n])
    run(["ipa-demo", "--length", 3], expected_stderr=INVALID_LENGTH_REJECTION)

    for count in [1, 2, 4, 8]:
        statement = expected(2, count)
        artifact = output / f"chain-{count}.bin"
        run(["prove-chain", "--initial", 2, "--steps", count, "--out", artifact],
            expected_statement=statement)
        def verify(path=artifact, stmt=statement, expected_stderr=None):
            return run(["verify-chain", "--initial", stmt["initial"], "--final", stmt["final_value"],
                        "--steps", stmt["steps"], "--proof", path],
                       expected_stderr=expected_stderr, expected_statement=stmt)
        verify()  # A new OS process, supplied only compressed artifact and expected statement.
        if count == 1:
            for key, wrong in [("initial", field(3)), ("final_value", field(11)), ("steps", 2)]:
                bad = dict(statement)
                bad[key] = wrong
                verify(stmt=bad, expected_stderr=PROOF_REJECTION)
            original = artifact.read_bytes()
            variants = {"truncated": original[:-1], "trailing": original + b"\0"}
            modified = bytearray(original)
            modified[len(modified) // 2] ^= 1
            variants["modified"] = bytes(modified)
            for name, data in variants.items():
                bad_path = output / f"{name}.bin"
                bad_path.write_bytes(data)
                verify(path=bad_path, expected_stderr=PROOF_REJECTION)
            oversized_path = output / "oversized.bin"
            oversized_path.write_bytes(b"\0" * (128 * 1024 + 1))
            verify(path=oversized_path, expected_stderr=OVERSIZED_PROOF_REJECTION)
    report["completed_utc"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    report["passed"] = True
    save()
    print(f"PASS: {len(report['runs'])} subprocess checks; evidence: {report_path}")


if __name__ == "__main__":
    main()
