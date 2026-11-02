"""Fail-closed regression tests for the acceptance process-result contract."""
import json
from pathlib import Path
import subprocess
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import acceptance


class CompletedProcessContractTests(unittest.TestCase):
    def setUp(self):
        self.statement = acceptance.expected(2, 1)
        self.verify_metrics = {
            "command": "verify-chain", "verified": True,
            "terminal_accumulator_checks": True, "statement": self.statement,
        }

    @staticmethod
    def completed(returncode=0, stdout="", stderr=""):
        return subprocess.CompletedProcess(["lab-cli"], returncode, stdout, stderr)

    def verify(self, completed, **kwargs):
        return acceptance.validate_completed(["verify-chain"], completed,
                                              expected_statement=self.statement, **kwargs)

    def successful(self, metrics=None, **kwargs):
        return self.completed(stdout=json.dumps(self.verify_metrics if metrics is None else metrics), **kwargs)

    def test_positive_verification_contract(self):
        actual = self.verify(self.successful())
        self.assertEqual(actual, self.verify_metrics)
        ipa = {"command": "ipa-demo", "verified": True, "length": 8}
        self.assertEqual(acceptance.validate_completed(
            ["ipa-demo", "--length", 8], self.completed(stdout=json.dumps(ipa))), ipa)
        prove = {"command": "prove-chain", "statement": self.statement}
        self.assertEqual(acceptance.validate_completed(
            ["prove-chain"], self.completed(stdout=json.dumps(prove)),
            expected_statement=self.statement), prove)

    def test_positive_contract_from_real_subprocess(self):
        # Exercise genuine captured stdout/stderr/status transport without paying
        # for a proof in these unit tests. Real proof runs remain acceptance.py's job.
        completed = subprocess.run(
            [sys.executable, "-c", "import sys; sys.stdout.write(sys.argv[1] + '\\n')",
             json.dumps(self.verify_metrics)], capture_output=True, text=True, check=False)
        self.assertEqual(self.verify(completed), self.verify_metrics)

    def test_exact_application_rejections_pass(self):
        for command, diagnostic in (
                ("verify-chain", acceptance.PROOF_REJECTION),
                ("verify-chain", acceptance.OVERSIZED_PROOF_REJECTION),
                ("ipa-demo", acceptance.INVALID_LENGTH_REJECTION)):
            with self.subTest(diagnostic=diagnostic):
                self.assertIsNone(acceptance.validate_completed(
                    [command], self.completed(1, stderr=diagnostic), expected_stderr=diagnostic))

    def test_panics_signals_and_windows_crashes_never_count_as_rejections(self):
        # Original weakness: every nonzero status counted as a successful negative case.
        for returncode in (2, 101, -6, -9, -11, 0xC0000005, -1073741819, 3221226505):
            for diagnostic in (acceptance.PROOF_REJECTION, "thread 'main' panicked\n", ""):
                with self.subTest(returncode=returncode, diagnostic=diagnostic):
                    with self.assertRaises(AssertionError):
                        self.verify(self.completed(returncode, stderr=diagnostic),
                                    expected_stderr=acceptance.PROOF_REJECTION)

    def test_generic_exit_one_errors_never_count_as_proof_rejections(self):
        for diagnostic in ("", "error: file not found\n", "error: InvalidWitness\n",
                           "error: proof rejected for expected public statement",  # Missing newline.
                           acceptance.PROOF_REJECTION + "panic after rejection\n",
                           "warning\n" + acceptance.PROOF_REJECTION,
                           acceptance.OVERSIZED_PROOF_REJECTION):
            with self.subTest(diagnostic=diagnostic):
                with self.assertRaises(AssertionError):
                    self.verify(self.completed(1, stderr=diagnostic),
                                expected_stderr=acceptance.PROOF_REJECTION)

    def test_rejection_must_have_empty_stdout(self):
        for stdout in ("\n", "{}", json.dumps(self.verify_metrics)):
            with self.subTest(stdout=stdout), self.assertRaises(AssertionError):
                self.verify(self.completed(1, stdout=stdout, stderr=acceptance.PROOF_REJECTION),
                            expected_stderr=acceptance.PROOF_REJECTION)
        with self.assertRaises(AssertionError):
            self.verify(self.completed(0, stderr=acceptance.PROOF_REJECTION),
                        expected_stderr=acceptance.PROOF_REJECTION)

    def test_success_rejects_any_nonzero_status_or_stderr(self):
        for returncode in (1, 101, -11, 0xC0000005):
            with self.subTest(returncode=returncode), self.assertRaises(AssertionError):
                self.verify(self.successful(returncode=returncode))
        for stderr in ("\n", "warning\n", acceptance.PROOF_REJECTION):
            with self.subTest(stderr=stderr), self.assertRaises(AssertionError):
                self.verify(self.successful(stderr=stderr))

    def test_full_stdout_must_be_one_unambiguous_json_object(self):
        valid = json.dumps(self.verify_metrics)
        for stdout in ("", "log\n" + valid, valid + "\n{}", valid + "\nlog", "[]", "null",
                       '"string"', "true", "1", '{"command":"verify-chain",',
                       valid[:-1] + ',"verified":true}',
                       valid[:-1] + ',"number":NaN}', valid[:-1] + ',"number":Infinity}'):
            with self.subTest(stdout=stdout), self.assertRaises(AssertionError):
                self.verify(self.completed(stdout=stdout))
        self.assertEqual(self.verify(self.completed(stdout=" \n" + valid + "\n ")), self.verify_metrics)

    def test_command_must_match_requested_command(self):
        for command in (None, "ipa-demo", "prove-chain", 1):
            metrics = dict(self.verify_metrics, command=command)
            with self.subTest(command=command), self.assertRaises(AssertionError):
                self.verify(self.successful(metrics))
        metrics = dict(self.verify_metrics)
        del metrics["command"]
        with self.assertRaises(AssertionError):
            self.verify(self.successful(metrics))

    def test_verified_requires_boolean_true_not_truthiness(self):
        # Original weakness: even explicit verified:false with exit zero passed.
        for command in ("verify-chain", "ipa-demo"):
            for value in (False, 0, 1, "true", "false", None, [], {}):
                metrics = dict(self.verify_metrics, command=command, verified=value)
                with self.subTest(command=command, value=value), self.assertRaises(AssertionError):
                    acceptance.validate_completed([command], self.successful(metrics),
                                                  expected_statement=self.statement)
            metrics = dict(self.verify_metrics, command=command)
            del metrics["verified"]
            with self.assertRaises(AssertionError):
                acceptance.validate_completed([command], self.successful(metrics),
                                              expected_statement=self.statement)

    def test_terminal_checks_require_boolean_true(self):
        for value in (False, 0, 1, "true", None):
            metrics = dict(self.verify_metrics, terminal_accumulator_checks=value)
            with self.subTest(value=value), self.assertRaises(AssertionError):
                self.verify(self.successful(metrics))
        metrics = dict(self.verify_metrics)
        del metrics["terminal_accumulator_checks"]
        with self.assertRaises(AssertionError):
            self.verify(self.successful(metrics))

    def test_expected_statement_is_independent_and_exact(self):
        for key, value in (("initial", acceptance.field(3)), ("final_value", acceptance.field(8)),
                           ("steps", 2), ("steps", True), ("steps", 1.0), ("steps", "1")):
            wrong = dict(self.statement)
            wrong[key] = value
            with self.subTest(key=key, value=value), self.assertRaises(AssertionError):
                self.verify(self.successful(dict(self.verify_metrics, statement=wrong)))
        for wrong in (None, [], {}, dict(self.statement, extra="unexpected")):
            with self.subTest(wrong=wrong), self.assertRaises(AssertionError):
                self.verify(self.successful(dict(self.verify_metrics, statement=wrong)))
        with self.assertRaises(AssertionError):
            acceptance.validate_completed(["verify-chain"], self.successful())

    def test_invalid_validator_contract_fails_closed(self):
        for args in ([], [None]):
            with self.subTest(args=args), self.assertRaises(AssertionError):
                acceptance.validate_completed(args, self.successful())
        for diagnostic in ("", False):
            with self.subTest(diagnostic=diagnostic), self.assertRaises(AssertionError):
                self.verify(self.completed(1), expected_stderr=diagnostic)


if __name__ == "__main__":
    unittest.main()
