#!/bin/bash

# Create the build directory
mkdir -p build/dist

# 1. Build for the current platform (Android/Termux)
cargo build --release

# 2. Prepare the distribution folder
# Copy 'portal' but rename it to 'hiverra-portal' for the archive
cp target/release/portal build/dist/hiverra-portal

# 3. Create the compressed archive
# The installer looks for: hiverra-portal-android.tar.gz
tar -czvf build/hiverra-portal-android.tar.gz -C build/dist hiverra-portal

# 4. Generate the Checksum file
# The installer looks for: hiverra-portal-android.tar.gz.sha256
sha256sum build/hiverra-portal-android.tar.gz | awk '{print $1}' > build/hiverra-portal-android.tar.gz.sha256

echo "--------------------------------------------------"
echo "Done! Upload these files to your GitHub Release:"
echo "1. build/hiverra-portal-android.tar.gz"
echo "2. build/hiverra-portal-android.tar.gz.sha256"
