# Exercises the release image on an interactive Windows desktop. No Docker or
# existing app data is used. Protocol registrations are restored in finally.
param([string]$Binary = 'target/release/local-store.exe')
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'This harness requires Windows.' }
if (Get-Process -Name local-store -ErrorAction SilentlyContinue) {
    throw 'Close Local Store before running the isolated native smoke test.'
}
$workspace = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$exe = (Resolve-Path -LiteralPath $Binary).Path
$run = Join-Path $workspace ('.cache/native-smoke-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $run | Out-Null
$config = Join-Path $run 'config'
New-Item -ItemType Directory -Path (Join-Path $config 'local-store') | Out-Null
$socket = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
$socket.Start()
$port = $socket.LocalEndpoint.Port
$socket.Stop()
@{version=2;apps=@(@{id='native-smoke';catalog_id=$null;display_name='Native smoke notes';
    launch_url="http://127.0.0.1:$port";icon_path=$null;runtime=@{kind='external'};
    created_at_unix=1;updated_at_unix=1})} | ConvertTo-Json -Depth 6 |
    Set-Content -LiteralPath (Join-Path $config 'local-store/registry-v2.json') -Encoding utf8NoBOM

Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class NativeSmokeWindows {
    delegate bool Callback(IntPtr window, IntPtr state);
    [DllImport("user32.dll")] static extern bool EnumWindows(Callback callback, IntPtr state);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint pid);
    [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr window, StringBuilder text, int count);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr window, uint message, IntPtr w, IntPtr l);
    public static IntPtr[] Find(int pid, string title) {
        var found = new List<IntPtr>();
        EnumWindows((window, state) => {
            uint owner; GetWindowThreadProcessId(window, out owner);
            var text = new StringBuilder(512); GetWindowText(window, text, 512);
            if (owner == pid && text.ToString() == title) found.Add(window);
            return true;
        }, IntPtr.Zero);
        return found.ToArray();
    }
}
'@
function Wait-Until([scriptblock]$Check, [string]$Description) {
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        if (& $Check) { return }
        Start-Sleep -Milliseconds 200
    } while ([DateTime]::UtcNow -lt $deadline)
    throw "Timed out: $Description"
}
function Run-Secondary([string[]]$Arguments) {
    $info = [Diagnostics.ProcessStartInfo]::new($exe)
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    foreach ($argument in $Arguments) { $info.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::Start($info)
    try {
        if (!$process.WaitForExit(15000)) { $process.Kill($true); throw 'Secondary process did not exit.' }
        if ($process.ExitCode -ne 0) { throw "Secondary process exited $($process.ExitCode)." }
    } finally { $process.Dispose() }
}

$schemes = (Get-Content (Join-Path $workspace 'tauri.conf.json') -Raw | ConvertFrom-Json).plugins.'deep-link'.desktop.schemes
$backups = @()
$checks = [Collections.Generic.List[string]]::new()
$cleanupErrors = [Collections.Generic.List[string]]::new()
$oldAppData = $env:APPDATA
$oldWebviewData = $env:WEBVIEW2_USER_DATA_FOLDER
$primary = $null
$server = $null
$failure = $null
try {
    foreach ($scheme in $schemes) {
        if ($scheme -notmatch '^[a-z]+$') { throw 'Unexpected protocol key.' }
        $key = "HKCU\Software\Classes\$scheme"
        $saved = Join-Path $run "$scheme.reg"
        $existed = Test-Path -LiteralPath "Registry::HKEY_CURRENT_USER\Software\Classes\$scheme"
        if ($existed) {
            & reg.exe export $key $saved /y | Out-Null
            if ($LASTEXITCODE -ne 0) { throw "Cannot back up $scheme registration." }
        }
        $backups += @{key=$key;file=$saved;existed=$existed}
    }
    $env:APPDATA = $config
    $env:WEBVIEW2_USER_DATA_FOLDER = Join-Path $run 'webview'
    $serverScript = '"' + (Join-Path $PSScriptRoot 'native-smoke-server.mjs') + '"'
    $server = Start-Process -FilePath (Get-Command node).Source -ArgumentList @($serverScript, $port) -PassThru -WindowStyle Hidden
    Wait-Until { try { (Invoke-WebRequest "http://127.0.0.1:$port/results" -TimeoutSec 1).StatusCode -eq 200 } catch { $false } } 'fixture server'
    $primary = Start-Process -FilePath $exe -PassThru -WindowStyle Hidden
    Wait-Until { [NativeSmokeWindows]::Find($primary.Id, 'Local Store').Count -eq 1 } 'launcher window'
    $checks.Add('Launcher opens from the release image')
    Run-Secondary @('open', 'native-smoke')
    Wait-Until { [NativeSmokeWindows]::Find($primary.Id, 'Native smoke notes').Count -eq 1 } 'forwarded app window in primary process'
    $checks.Add('Second process forwards native open and exits successfully')
    Run-Secondary @('open', 'native-smoke')
    if ([NativeSmokeWindows]::Find($primary.Id, 'Native smoke notes').Count -ne 1) { throw 'Duplicate app window.' }
    $checks.Add('Repeated activation reuses one app window')
    Wait-Until { (Invoke-RestMethod "http://127.0.0.1:$port/results").Count -gt 0 } 'remote IPC probe'
    $probes = Invoke-RestMethod "http://127.0.0.1:$port/results"
    if ($probes[0].results.Count -ne 4 -or @($probes[0].results | Where-Object { !$_.denied }).Count) {
        throw 'Remote app could access a launcher command.'
    }
    $checks.Add('Remote app cannot invoke the four probed launcher commands')
    foreach ($scheme in $schemes) {
        $window = [NativeSmokeWindows]::Find($primary.Id, 'Native smoke notes')[0]
        [NativeSmokeWindows]::PostMessage($window, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null
        Wait-Until { [NativeSmokeWindows]::Find($primary.Id, 'Native smoke notes').Count -eq 0 } 'app window close'
        Start-Process -FilePath "${scheme}://open/native-smoke" -WindowStyle Hidden
        Wait-Until { [NativeSmokeWindows]::Find($primary.Id, 'Native smoke notes').Count -eq 1 } 'registered protocol activation'
        $checks.Add("Registered $scheme protocol opens the app in the original process")
    }
    Run-Secondary @('--version')
    $checks.Add('CLI still exits successfully while the GUI is running')
} catch { $failure = $_.Exception.Message }
finally {
    foreach ($owned in @($primary, $server)) {
        if ($owned) {
            try { if (!$owned.HasExited) { $owned.Kill($true); $owned.WaitForExit(10000) | Out-Null } }
            catch { $cleanupErrors.Add($_.Exception.Message) }
        }
    }
    $env:APPDATA = $oldAppData
    $env:WEBVIEW2_USER_DATA_FOLDER = $oldWebviewData
    foreach ($backup in $backups) {
        try {
            # Only exact protocol keys backed up above may be restored.
            & reg.exe delete $backup.key /f 2>$null | Out-Null
            if (Test-Path -LiteralPath ('Registry::' + $backup.key.Replace('HKCU', 'HKEY_CURRENT_USER'))) {
                throw "Could not remove test registration $($backup.key). Backup: $($backup.file)"
            }
            if ($backup.existed) {
                & reg.exe import $backup.file | Out-Null
                if ($LASTEXITCODE -ne 0) { throw "Could not restore $($backup.key). Backup: $($backup.file)" }
            }
        } catch { $cleanupErrors.Add($_.Exception.Message) }
    }
    @{binary=$exe;checks=$checks.ToArray();failure=$failure;cleanupErrors=$cleanupErrors.ToArray();
        probes=$probes;finishedUtc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json -Depth 8 |
        Set-Content -LiteralPath (Join-Path $run 'report.json') -Encoding utf8NoBOM
    Write-Output "Native smoke report: $run/report.json"
}
if ($failure) { throw $failure }
if ($cleanupErrors.Count) { throw ($cleanupErrors -join '; ') }
Write-Output ($checks -join "`n")
