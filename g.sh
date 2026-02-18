#!/bin/sh
set -e
REPO="Spectra010s/portal"
TAG="v0.9.0"
BINARY_NAME="hiverra-portal-aarch64-linux-android.tar.gz"
URL="https://github.com/$REPO/releases/download/$TAG/$BINARY_NAME"

echo "Installing portal ($TAG) for Android..."

TMP_DIR=$(mktemp -d "$PREFIX/tmp/portal-install-XXXXXX")
FILE="$TMP_DIR/hiverra-portal.tar.gz"

curl -sSL "$URL" -o "$FILE"
curl -sSL "$URL.sha256" -o "$TMP_DIR/check.sha256"

EXPECTED_HASH=$(awk '{print $1}' "$TMP_DIR/check.sha256")
ACTUAL_HASH=$(sha256sum "$FILE" | awk '{print $1}')

if [ "$EXPECTED_HASH" != "$ACTUAL_HASH" ]; then
    echo "ERROR: Checksum mismatch!"
    exit 1
fi

tar -xzf "$FILE" -C "$TMP_DIR"
mv "$TMP_DIR/portal" "$PREFIX/bin/portal"
chmod +x "$PREFIX/bin/portal"
rm -rf "$TMP_DIR"

echo "Success! Type 'portal' to start."
