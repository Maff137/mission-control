# Optional hosted-desktop keyboard acceptance; no app hooks or external modules.
param([int]$AppProcessId, [string]$EvidenceRoot)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public class AcceptanceWindow {
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out Rect r);
    public struct Rect { public int Left, Top, Right, Bottom; }
}
'@
function Assert-OwnedForeground {
    [uint32]$owner = 0
    [void][AcceptanceWindow]::GetWindowThreadProcessId([AcceptanceWindow]::GetForegroundWindow(), [ref]$owner)
    if ($owner -ne $AppProcessId) { throw 'BLOCKED: foreground window is not owned by the acceptance app' }
}
function Send-Keys([string]$keys) {
    Assert-OwnedForeground
    [System.Windows.Forms.SendKeys]::SendWait($keys)
    Start-Sleep -Milliseconds 250
}
function Paste-Literal([string]$text) {
    Assert-OwnedForeground
    [System.Windows.Forms.Clipboard]::SetText($text)
    Send-Keys '^v'
}
$app = Get-Process -Id $AppProcessId
$app.Refresh()
if ($app.MainWindowHandle -eq [IntPtr]::Zero -or $app.MainWindowTitle -ne 'Mission Control') {
    throw 'BLOCKED: normal application window unavailable'
}
[void][AcceptanceWindow]::ShowWindow($app.MainWindowHandle, 9)
[void][AcceptanceWindow]::SetForegroundWindow($app.MainWindowHandle)
Start-Sleep -Seconds 2
Assert-OwnedForeground
Send-Keys '{TAB}'
Paste-Literal 'CI UI save through bundled WebView'
Send-Keys '{ENTER}'
Start-Sleep -Seconds 3
$rect = New-Object AcceptanceWindow+Rect
if (-not [AcceptanceWindow]::GetWindowRect($app.MainWindowHandle, [ref]$rect)) { throw 'Screenshot window unavailable' }
$bitmap = [System.Drawing.Bitmap]::new(($rect.Right-$rect.Left), ($rect.Bottom-$rect.Top))
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)
try {
    $graphics.CopyFromScreen($rect.Left, $rect.Top, 0, 0, $bitmap.Size)
    $bitmap.Save((Join-Path $EvidenceRoot 'windows-ui.png'), [System.Drawing.Imaging.ImageFormat]::Png)
} finally {
    $graphics.Dispose()
    $bitmap.Dispose()
}
# Empty input keeps Save disabled; the next tab reaches Export Backup.
Send-Keys '{TAB}'
Send-Keys '{ENTER}'
Start-Sleep -Seconds 3
Send-Keys '^a'
Paste-Literal (Join-Path $EvidenceRoot 'ui-export.mcbackup')
Send-Keys '{ENTER}'
Start-Sleep -Seconds 3
