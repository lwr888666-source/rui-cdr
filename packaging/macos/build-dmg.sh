#!/usr/bin/env bash
# Build Rui.app and a DMG. Usage: build-dmg.sh <version> <release-dir> <arch-label>
# Unsigned: first launch needs right-click > Open (or xattr -d com.apple.quarantine).
set -euo pipefail
VERSION="$1"; BIN="$2"; ARCH="$3"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
APP="$ROOT/dist/Rui.app"
rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
sed "s/__VERSION__/$VERSION/g" "$ROOT/packaging/macos/Info.plist" > "$APP/Contents/Info.plist"
cp "$BIN/rui" "$BIN/rui-cli" "$APP/Contents/MacOS/"
chmod +x "$APP/Contents/MacOS/"*

# Icon: build .icns from the PNG set.
ICONSET="$ROOT/dist/rui.iconset"; rm -rf "$ICONSET"; mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  cp "$ROOT/assets/icon/rui-$s.png" "$ICONSET/icon_${s}x${s}.png"
  d=$((s*2)); cp "$ROOT/assets/icon/rui-$d.png" "$ICONSET/icon_${s}x${s}@2x.png"
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/rui.icns"

# Ad-hoc signature so Gatekeeper reports a consistent bundle.
codesign --force --deep --sign - "$APP"

DMG="$ROOT/dist/Rui-$VERSION-macos-$ARCH.dmg"
rm -f "$DMG"
STAGE="$ROOT/dist/dmg-stage"; rm -rf "$STAGE"; mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/"; ln -s /Applications "$STAGE/Applications"
# Leave explicit space for filesystem metadata and file-copy overhead.
# Automatic APFS sizing can run out of space inside the image on runners.
STAGE_KB=$(du -sk "$STAGE" | awk '{print $1}')
IMAGE_MB=$(((STAGE_KB * 5 / 4 + 1023) / 1024 + 128))
echo "DMG source: ${STAGE_KB} KiB; image capacity: ${IMAGE_MB} MiB"
# hdiutil on the build machines now and then fails with "Resource busy"
# while the system still scans the staged files: try again a few times,
# waiting longer each time. A last failure is reported as a build
# annotation, which shows without the job's log.
for attempt in 1 2 3 4 5 6 7; do
  if out=$(hdiutil create -volname "Rui" -fs HFS+ -size "${IMAGE_MB}m" -srcfolder "$STAGE" -ov -format UDZO "$DMG" 2>&1); then
    echo "$out"
    break
  fi
  echo "$out" >&2
  if [ "$attempt" = 7 ]; then
    echo "::error::hdiutil create failed $attempt times: $(echo "$out" | tr '\n' ' ' | cut -c1-400)"
    exit 1
  fi
  sleep $((attempt * 15))
done
hdiutil verify "$DMG"
MOUNT=$(mktemp -d)
trap 'hdiutil detach "$MOUNT" >/dev/null 2>&1 || true; rmdir "$MOUNT" 2>/dev/null || true' EXIT
hdiutil attach -readonly -nobrowse -mountpoint "$MOUNT" "$DMG"
cmp "$APP/Contents/MacOS/rui" "$MOUNT/Rui.app/Contents/MacOS/rui"
cmp "$APP/Contents/MacOS/rui-cli" "$MOUNT/Rui.app/Contents/MacOS/rui-cli"
test "$(/usr/libexec/PlistBuddy -c 'Print :CFBundleDisplayName' "$MOUNT/Rui.app/Contents/Info.plist")" = Rui
test -L "$MOUNT/Applications"
echo "DMG mounted; Rui name, application and CLI verified"
hdiutil detach "$MOUNT"
rmdir "$MOUNT"
trap - EXIT
rm -rf "$STAGE" "$ICONSET"
echo "built $DMG"
