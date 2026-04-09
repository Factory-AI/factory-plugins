# win-key-dumper.ps1 — Win32 console key event dumper
# Uses $host.UI.RawUI.ReadKey for lossless key metadata.
# Shows VirtualKeyCode, ControlKeyState, Character for each keypress.
# Captures distinctions VT mode loses (e.g., Shift+Enter vs Enter).
# Press Ctrl+Q to exit.

Write-Host "Key event dumper active. Press keys to inspect. Ctrl+Q to exit." -ForegroundColor Cyan
Write-Host ""

$fmt = "{0,-6} {1,-10} {2,-20} {3,-35} {4}"
Write-Host ($fmt -f "Down", "Char", "VKey", "ControlKeyState", "CharHex") -ForegroundColor DarkGray
Write-Host ($fmt -f ("=" * 6), ("=" * 10), ("=" * 20), ("=" * 35), ("=" * 8)) -ForegroundColor DarkGray

while ($true) {
    $k = $host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown,IncludeKeyUp')

    $charCode = [int][char]$k.Character
    $charHex = "U+{0:X4}" -f $charCode
    $charDisplay = if ($charCode -ge 0x20 -and $charCode -le 0x7E) { $k.Character } else { "." }

    # Ctrl+Q keydown
    if ($k.KeyDown -and $k.VirtualKeyCode -eq 81 -and
        ($k.ControlKeyState -band 0x0008)) { break }  # LEFT_CTRL_PRESSED

    if ($k.KeyDown) {
        $line = $fmt -f $k.KeyDown, $charDisplay, $k.VirtualKeyCode, $k.ControlKeyState, $charHex
        Write-Host $line
    }
}

Write-Host "`nDone." -ForegroundColor Cyan
