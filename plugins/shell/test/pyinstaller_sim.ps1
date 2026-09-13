# Simulates a PyInstaller-wrapped application.
#
# PyInstaller bundles a thin "bootloader" that spawns the real Python app
# as a child process. When Tauri kills the bootloader, the real app is
# orphaned unless the entire job object is terminated.
#
# This script mimics that pattern:
#   - It spawns a long-running child ("the real app")
#   - Prints the child's PID so the test harness can verify it was killed
#   - Waits on the child (like PyInstaller's bootloader does)

# "The real application" — a grandchild from Tauri's perspective
$child = Start-Process -PassThru -WindowStyle Hidden powershell.exe -ArgumentList '-NoProfile', '-Command', 'Start-Sleep -Seconds 3600'

"WRAPPER_PID=$PID"
"CHILD_PID=$($child.Id)"

# The bootloader waits for the real app to finish
Wait-Process -Id $child.Id
