# Pepometer

A readiness, sleep and training app for Garmin watches: open it and see how
you're doing, why, and what to do today, from Garmin's own metrics.

Status: first test build (Android). See [docs/spec.md](docs/spec.md) and the
[mockup](design/mockup/) (sample data).

![Pepometer mockup: Today, Sleep, Training, Workout and Health screens](design/mockup/screens.png)

## Install

Android only for now. Install and update with
[Obtainium](https://github.com/ImranR98/Obtainium): add
`https://github.com/dcardamo/pepometer` as an app. Each GitHub release carries
a signed arm64 APK, and Obtainium updates the app when a new release appears.

While this repository is private, Obtainium needs a GitHub fine-grained token
with read-only access to its contents (Obtainium → Settings → GitHub personal
access token).

Pepometer is not affiliated with Garmin. It uses Garmin Connect's
unofficial API, which Garmin can change at any time.
