# Simulates a launcher that starts the real application and exits right away,
# leaving the application running in its job object.

$child = Start-Process -PassThru -WindowStyle Hidden powershell.exe -ArgumentList '-NoProfile', '-Command', 'Start-Sleep -Seconds 3600'

"WRAPPER_PID=$PID"
"CHILD_PID=$($child.Id)"
