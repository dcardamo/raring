#!/usr/bin/env bash
# Creates Raring's emulator (Pixel 7 sized, Android 15, Google Play image).
# avdmanager fails on the Nix SDK's read-only device list, so the files are
# written directly. Run inside `nix develop`.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p "$ANDROID_AVD_HOME/pepo35.avd"
cat > "$ANDROID_AVD_HOME/pepo35.ini" <<INI
avd.ini.encoding=UTF-8
path=$ANDROID_AVD_HOME/pepo35.avd
target=android-35
INI
cat > "$ANDROID_AVD_HOME/pepo35.avd/config.ini" <<'INI'
avd.ini.encoding=UTF-8
AvdId=pepo35
avd.name=pepo35
abi.type=x86_64
hw.cpu.arch=x86_64
hw.cpu.ncore=4
hw.ramSize=4096
disk.dataPartition.size=8G
image.sysdir.1=system-images/android-35/google_apis_playstore/x86_64/
tag.id=google_apis_playstore
tag.display=Google Play
PlayStore.enabled=true
hw.lcd.width=1080
hw.lcd.height=2400
hw.lcd.density=420
hw.keyboard=no
hw.gpu.enabled=yes
hw.gpu.mode=swiftshader_indirect
hw.audioInput=no
showDeviceFrame=no
INI
echo "created pepo35 in $ANDROID_AVD_HOME"
