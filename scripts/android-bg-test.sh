#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
pkg=ca.hld.pepometer.debug
adb shell pm grant "$pkg" android.permission.POST_NOTIFICATIONS
# Isolate the debug probe from any prior debug sign-in; never touch the release vault.
adb shell run-as "$pkg" rm -f no_backup/tokens.enc
adb shell am force-stop "$pkg"
adb logcat -c
adb shell am broadcast -n "$pkg/ca.hld.pepometer.DebugSyncReceiver" -a ca.hld.pepometer.DEBUG_SYNC
for attempt in $(seq 1 40); do
  if adb logcat -d -s PepometerSync:I '*:S' | grep -q 'notification posted: auth-required' &&
     adb shell dumpsys notification --noredact | grep -q 'Sign in to Garmin again'; then
    echo 'PASS: no tokens after force-stop posts sign-in notification'
    break
  fi
  if [[ $attempt == 40 ]]; then
    adb logcat -d -s PepometerSync:V '*:S'
    exit 1
  fi
  sleep 2
done
# Invalid synthetic JSON exercises the real JNI authentication result against the demo DB
# without contacting Garmin or ever installing a usable credential.
adb logcat -c
adb shell am force-stop "$pkg"
adb shell am broadcast -n "$pkg/ca.hld.pepometer.DebugSyncReceiver" \
  -a ca.hld.pepometer.DEBUG_SYNC --ez seedSynthetic true
for attempt in $(seq 1 40); do
  if adb logcat -d -s PepometerSync:I '*:S' | grep -q 'notification posted: auth-required' &&
     ! adb logcat -d -s PepometerSync:E '*:S' | grep -q 'background sync failed' &&
     ! adb shell run-as "$pkg" ls no_backup/tokens.enc 2>/dev/null; then
    echo 'PASS: JNI rejects synthetic credentials after force-stop without worker crash'
    exit 0
  fi
  sleep 2
done
adb logcat -d -s PepometerSync:V '*:S'
exit 1
