# win-vt-dumper.ps1 — raw VT byte dumper for Windows console
# Enables ENABLE_VIRTUAL_TERMINAL_INPUT, reads stdin as raw bytes, hex-dumps.
# Shows exactly what escape sequences the console delivers for each keypress.
# Press Ctrl+Q to exit.

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class WinConsole {
    public const int STD_INPUT_HANDLE = -10;
    public const uint ENABLE_PROCESSED_INPUT = 0x0001;
    public const uint ENABLE_LINE_INPUT      = 0x0002;
    public const uint ENABLE_ECHO_INPUT      = 0x0004;
    public const uint ENABLE_VIRTUAL_TERMINAL_INPUT = 0x0200;

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern IntPtr GetStdHandle(int nStdHandle);

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool GetConsoleMode(IntPtr h, out uint mode);

    [DllImport("kernel32.dll", SetLastError = true)]
    public static extern bool SetConsoleMode(IntPtr h, uint mode);
}
"@

$hIn = [WinConsole]::GetStdHandle([WinConsole]::STD_INPUT_HANDLE)
[uint32]$oldMode = 0
if (-not [WinConsole]::GetConsoleMode($hIn, [ref]$oldMode)) {
    Write-Error "Not a console handle"; exit 1
}

$newMode = ($oldMode -band (-bnot (
    [WinConsole]::ENABLE_PROCESSED_INPUT -bor
    [WinConsole]::ENABLE_LINE_INPUT -bor
    [WinConsole]::ENABLE_ECHO_INPUT
))) -bor [WinConsole]::ENABLE_VIRTUAL_TERMINAL_INPUT

[Console]::TreatControlCAsInput = $true

if (-not [WinConsole]::SetConsoleMode($hIn, $newMode)) {
    Write-Error "VT input mode not supported"; exit 1
}

Write-Host "VT byte dumper active. Press keys to see hex. Ctrl+Q to exit." -ForegroundColor Cyan
Write-Host ("{0,-35} {1}" -f "HEX", "ASCII") -ForegroundColor DarkGray
Write-Host ("{0,-35} {1}" -f ("=" * 35), ("=" * 20)) -ForegroundColor DarkGray

$stdin = [Console]::OpenStandardInput()
$buf = New-Object byte[] 64

try {
    while ($true) {
        $n = $stdin.Read($buf, 0, $buf.Length)
        if ($n -le 0) { break }

        # Ctrl+Q = 0x11
        if ($n -eq 1 -and $buf[0] -eq 0x11) { break }

        $hex = ($buf[0..($n-1)] | ForEach-Object { $_.ToString("X2") }) -join " "
        $ascii = -join ($buf[0..($n-1)] | ForEach-Object {
            if ($_ -ge 0x20 -and $_ -le 0x7E) { [char]$_ } else { "." }
        })
        "{0,-35} {1}" -f $hex, $ascii
    }
}
finally {
    [WinConsole]::SetConsoleMode($hIn, $oldMode) | Out-Null
    Write-Host "`nRestored console mode." -ForegroundColor Cyan
}
