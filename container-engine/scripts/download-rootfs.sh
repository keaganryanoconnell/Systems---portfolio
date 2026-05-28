#!/usr/bin/env bash
# Download Alpine Linux minirootfs for container runtime integration tests.
set -euo pipefail

ROOTFS_DIR="$(cd "$(dirname "$0")/../tests" && pwd)/rootfs"
ALPINE_VERSION="3.20"
ARCH="x86_64"
TARBALL="alpine-minirootfs-${ALPINE_VERSION}-${ARCH}.tar.gz"
URL="https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VERSION}/releases/${ARCH}/${TARBALL}"

if [ -d "$ROOTFS_DIR" ] && [ -f "$ROOTFS_DIR/.container-test-rootfs" ]; then
    echo "Rootfs already exists at $ROOTFS_DIR"
    exit 0
fi

echo "Downloading Alpine $ALPINE_VERSION minirootfs..."
mkdir -p "$ROOTFS_DIR"
curl -fsSL "$URL" -o "/tmp/${TARBALL}"

echo "Extracting to $ROOTFS_DIR..."
tar -xzf "/tmp/${TARBALL}" -C "$ROOTFS_DIR"

# Mark this rootfs as valid
touch "$ROOTFS_DIR/.container-test-rootfs"

# Verify key directories exist
for dir in bin etc lib proc sbin sys tmp usr var dev; do
    if [ ! -d "$ROOTFS_DIR/$dir" ]; then
        echo "ERROR: $ROOTFS_DIR/$dir missing after extraction!"
        exit 1
    fi
done

# Fix /etc/resolv.conf for network tests
echo "nameserver 8.8.8.8" > "$ROOTFS_DIR/etc/resolv.conf"
echo "nameserver 1.1.1.1" >> "$ROOTFS_DIR/etc/resolv.conf"

# Create /etc/hosts
echo "127.0.0.1 localhost" > "$ROOTFS_DIR/etc/hosts"

echo "Rootfs ready at $ROOTFS_DIR"
echo "Size: $(du -sh "$ROOTFS_DIR" | cut -f1)"
