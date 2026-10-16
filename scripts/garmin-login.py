"""Sign in to Garmin Connect once and save tokens for the Pepometer CLI.

Uses python-garminconnect, whose multi-strategy login is the most reliable
way to sign in from a computer. This creates a new Garmin session with its
own tokens; it does not touch tokens used by any other tool.

Usage: nix run .#garmin-login  (writes .data/garmin_tokens.json)
Then:  nix develop -c cargo run -p pepometer-cli -- login --import .data/garmin_tokens.json
"""

import getpass
import os
import sys

from garminconnect import (
    Garmin,
    GarminConnectAuthenticationError,
    GarminConnectConnectionError,
    GarminConnectTooManyRequestsError,
)


def main() -> int:
    out_dir = sys.argv[1]
    email = os.environ.get("GARMIN_EMAIL") or input("Garmin Connect email: ").strip()
    password = getpass.getpass(f"Password for {email}: ")

    def prompt_mfa() -> str:
        return input("MFA code: ").strip()

    try:
        Garmin(email=email, password=password, prompt_mfa=prompt_mfa).login(out_dir)
    except GarminConnectTooManyRequestsError as e:
        print(f"Garmin is rate-limiting sign-ins; wait before retrying: {e}", file=sys.stderr)
        return 4
    except (GarminConnectAuthenticationError, GarminConnectConnectionError) as e:
        print(f"Sign-in failed: {e}", file=sys.stderr)
        return 2

    path = os.path.join(out_dir, "garmin_tokens.json")
    os.chmod(path, 0o600)
    print(f"Signed in. Tokens saved to {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
