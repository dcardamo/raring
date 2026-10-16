#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
pkg=ca.hld.pepometer.debug
adb shell pm grant "$pkg" android.permission.POST_NOTIFICATIONS
adb shell am force-stop "$pkg"
adb logcat -c
# An explicit debug-only receiver starts the process after force-stop, without launching the UI.
# It seeds a synthetic token, then enqueues one real WorkManager CoroutineWorker run.
adb shell am broadcast -n "$pkg/ca.hld.pepometer.DebugSyncReceiver" \
  -a ca.hld.pepometer.DEBUG_SYNC --ez seedSynthetic true
for attempt in $(seq 1 40); do
  if adb logcat -d -s PepometerSync:I '*:S' | grep -q 'background JNI sync completed'; then
    if adb shell dumpsys notification --noredact | grep -q 'Background sync ran'; then
      echo 'PASS: JNI background sync completed after force-stop; notification posted'
      exit 0
    fi
  fi
  sleep 2
done
adb logcat -d -s PepometerSync:V '*:S'
echo 'FAIL: JNI sync or notification not observed' >&2
exit 1
