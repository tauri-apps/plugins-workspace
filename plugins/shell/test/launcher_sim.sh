#!/bin/bash
# Simulates a launcher that starts the real application and exits right away,
# leaving the application running in its process group.
#
# The grandchild's output is redirected so it doesn't hold the test's pipes open.

sleep 3600 >/dev/null 2>&1 &

echo "WRAPPER_PID=$$"
echo "CHILD_PID=$!"
