Add-Type -AssemblyName System.Drawing

function Create-TallyIcon {
    param([int]$Size, [string]$OutPath, [int]$Width = $Size, [int]$Height = $Size)
    $bmp = New-Object System.Drawing.Bitmap($Width, $Height)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit

    # Background rounded rectangle or full bleed depending on size
    $bgBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 29, 53, 87)) # Deep Slate Navy #1D3557
    $accentBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 230, 57, 70)) # Terracotta Accent #E63946
    $paperBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(255, 248, 249, 250)) # Warm Paper #F8F9FA
    $linePen = New-Object System.Drawing.Pen([System.Drawing.Color]::FromArgb(255, 69, 123, 157), [Math]::Max(1, [int]($Height * 0.04)))

    $g.FillRectangle($bgBrush, 0, 0, $Width, $Height)

    # Document sheet in center
    $docW = [int]([Math]::Min($Width, $Height) * 0.56)
    $docH = [int]([Math]::Min($Width, $Height) * 0.72)
    $docX = [int](($Width - $docW) / 2)
    $docY = [int](($Height - $docH) / 2)

    $g.FillRectangle($paperBrush, $docX, $docY, $docW, $docH)

    # Top accent bar on document
    $barH = [Math]::Max(2, [int]($docH * 0.16))
    $g.FillRectangle($accentBrush, $docX, $docY, $docW, $barH)

    # Lines representing budget items
    $padX = [int]($docW * 0.16)
    $lineY1 = $docY + $barH + [int]($docH * 0.20)
    $lineY2 = $docY + $barH + [int]($docH * 0.40)
    $lineY3 = $docY + $barH + [int]($docH * 0.60)

    $g.DrawLine($linePen, $docX + $padX, $lineY1, $docX + $docW - $padX, $lineY1)
    $g.DrawLine($linePen, $docX + $padX, $lineY2, $docX + $docW - $padX, $lineY2)
    $g.DrawLine($linePen, $docX + $padX, $lineY3, $docX + $docW - $padX, $lineY3)

    $bmp.Save($OutPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
}

# Generate PNGs
Create-TallyIcon -Size 256 -OutPath "assets\icon.png"
Create-TallyIcon -Size 32 -OutPath "src-tauri\icons\32x32.png"
Create-TallyIcon -Size 128 -OutPath "src-tauri\icons\128x128.png"
Create-TallyIcon -Size 256 -OutPath "src-tauri\icons\128x128@2x.png"
Create-TallyIcon -Size 44 -OutPath "assets\Square44x44Logo.png"
Create-TallyIcon -Size 150 -OutPath "assets\Square150x150Logo.png"
Create-TallyIcon -Size 50 -OutPath "assets\StoreLogo.png"
Create-TallyIcon -Size 150 -Width 310 -Height 150 -OutPath "assets\Wide310x150Logo.png"

# Create valid .ico file containing 256x256 PNG payload
$pngBytes = [System.IO.File]::ReadAllBytes("assets\icon.png")
foreach ($icoPath in @("assets\icon.ico", "src-tauri\icons\icon.ico")) {
    $fs = [System.IO.File]::Create($icoPath)
    $bw = New-Object System.IO.BinaryWriter($fs)
    # ICONDIR header
    $bw.Write([UInt16]0)   # Reserved
    $bw.Write([UInt16]1)   # Type: 1 = ICO
    $bw.Write([UInt16]1)   # Count: 1 image
    # ICONDIRENTRY
    $bw.Write([byte]0)     # Width: 0 means 256
    $bw.Write([byte]0)     # Height: 0 means 256
    $bw.Write([byte]0)     # ColorCount
    $bw.Write([byte]0)     # Reserved
    $bw.Write([UInt16]1)   # Planes
    $bw.Write([UInt16]32)  # BitCount
    $bw.Write([UInt32]$pngBytes.Length) # BytesInRes
    $bw.Write([UInt32]22)  # ImageOffset (6 + 16)
    $bw.Write($pngBytes)
    $bw.Close()
    $fs.Close()
}
Write-Output "Icons generated successfully."

