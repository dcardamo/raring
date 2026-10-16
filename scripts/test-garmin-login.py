"""Exercise login privacy without Garmin credentials, dependencies or requests."""

import contextlib
import io
import logging
import os
from pathlib import Path
import runpy
import sys
import tempfile
import types
import unittest
from unittest import mock


class RateLimited(Exception):
    pass


class LoginPrivacyTest(unittest.TestCase):
    def run_login(self, error=None):
        dependency = types.ModuleType("garminconnect")
        dependency.Garmin = mock.Mock()
        dependency.GarminConnectTooManyRequestsError = RateLimited
        with mock.patch.dict(sys.modules, {"garminconnect": dependency}):
            helper = runpy.run_path(str(Path(__file__).with_name("garmin-login.py")))

        def fake_login(out_dir):
            path = Path(out_dir)
            path.mkdir()
            tokens = path / "garmin_tokens.json"
            tokens.write_text('{"access_token":"synthetic-secret"}')
            # Check before the helper's final chmod, including on failure.
            self.assertEqual(path.stat().st_mode & 0o777, 0o700)
            self.assertEqual(tokens.stat().st_mode & 0o777, 0o600)
            logger = logging.getLogger("garminconnect.client")
            logger.warning("Garmin response: synthetic-secret")
            logger.debug("Garmin response: synthetic-secret")
            if error is not None:
                raise error

        dependency.Garmin.return_value.login.side_effect = fake_login
        stdout, stderr = io.StringIO(), io.StringIO()
        logger = logging.getLogger("garminconnect")
        previous_umask = os.umask(0o022)
        try:
            with (
                tempfile.TemporaryDirectory() as temp,
                mock.patch.dict(os.environ, {"GARMIN_EMAIL": "test@example.invalid"}),
                mock.patch.object(helper["getpass"], "getpass", return_value="synthetic-password"),
                mock.patch.object(sys, "argv", ["garmin-login.py", str(Path(temp) / "tokens")]),
                mock.patch.object(logger, "handlers", []),
                mock.patch.object(logger, "propagate", True),
                mock.patch.object(logger, "level", logging.DEBUG),
                contextlib.redirect_stdout(stdout),
                contextlib.redirect_stderr(stderr),
            ):
                result = helper["main"]()
                self.assertEqual(
                    (Path(temp) / "tokens/garmin_tokens.json").stat().st_mode & 0o777,
                    0o600,
                )
        finally:
            os.umask(previous_umask)
        output = stdout.getvalue() + stderr.getvalue()
        self.assertNotIn("synthetic-secret", output)
        self.assertNotIn("synthetic-password", output)
        return result, output

    def test_success_keeps_tokens_private_and_suppresses_dependency_logs(self):
        result, output = self.run_login()
        self.assertEqual(result, 0)
        self.assertIn("Signed in.", output)

    def test_failures_keep_tokens_private_and_hide_exception_details(self):
        for error, expected in [
            (RateLimited("synthetic-secret"), 4),
            (RuntimeError("synthetic-secret"), 2),
        ]:
            with self.subTest(error=type(error).__name__):
                result, _ = self.run_login(error)
                self.assertEqual(result, expected)


if __name__ == "__main__":
    unittest.main()
