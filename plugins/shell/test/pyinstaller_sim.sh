#!/bin/bash
# Simulates a PyInstaller-wrapped application.
#
# PyInstaller bundles a thin "bootloader" that spawns the real Python app
# as a child process. When Tauri kills the bootloader, the real app is
# orphaned unless the entire process group is terminated.
#
# This script mimics that pattern:
#   - It spawns a long-running child ("the real app")
#   - Prints the child's PID so the test harness can verify it was killed
#   - Waits on the child (like PyInstaller's bootloader does)

# "The real application" — a grandchild from Tauri's perspective
sleep 3600 &
CHILD_PID=$!

echo "WRAPPER_PID=$$"
echo "CHILD_PID=$CHILD_PID"

# The bootloader waits for the real app to finish
wait $CHILD_PID
