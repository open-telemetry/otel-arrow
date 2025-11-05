#!/bin/bash
set -euo pipefail

# Check disk space before operations
echo "=== Disk space check ==="
df -h

# Clean dnf cache to free up space
echo "=== Cleaning dnf cache ==="
sudo dnf clean all || true

# Clean up Docker to free space
echo "=== Cleaning up Docker ==="
docker system prune -f || true

echo "=== Disk space after cleanup ==="
df -h
