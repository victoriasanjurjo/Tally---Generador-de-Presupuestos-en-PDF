$ErrorActionPreference = "Stop"

$makeappx = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\makeappx.exe"
$signtool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"

if (-not (Test-Path $makeappx)) {
    throw "No se encontró makeappx.exe en la ruta esperada: $makeappx"
}
if (-not (Test-Path $signtool)) {
    throw "No se encontró signtool.exe en la ruta esperada: $signtool"
}

$exePath = "src-tauri\target\release\tally.exe"
if (-not (Test-Path $exePath)) {
    throw "No se encontró el binario compilado en $exePath. Ejecute primero cargo build --release."
}

# 1. Preparar directorio de staging para el paquete MSIX
$stagingDir = "msix_staging"
if (Test-Path $stagingDir) {
    Remove-Item -Recurse -Force $stagingDir
}
New-Item -ItemType Directory -Force -Path "$stagingDir\Assets" | Out-Null

Copy-Item $exePath "$stagingDir\tally.exe" -Force
Copy-Item "config\AppxManifest.xml" "$stagingDir\AppxManifest.xml" -Force
Copy-Item "assets\Square44x44Logo.png" "$stagingDir\Assets\Square44x44Logo.png" -Force
Copy-Item "assets\Square150x150Logo.png" "$stagingDir\Assets\Square150x150Logo.png" -Force
Copy-Item "assets\Wide310x150Logo.png" "$stagingDir\Assets\Wide310x150Logo.png" -Force
Copy-Item "assets\StoreLogo.png" "$stagingDir\Assets\StoreLogo.png" -Force

# 2. Generar certificado de desarrollo autofirmado (CN=Victoria Sanjurjo)
$certDir = "config\certs"
New-Item -ItemType Directory -Force -Path $certDir | Out-Null
$pfxPath = "$certDir\VictoriaSanjurjo_TallyDev.pfx"
$cerPath = "outputs\VictoriaSanjurjo_TallyDev.cer"
$passwordPlain = "TallyLocal2026!"
$securePwd = ConvertTo-SecureString -String $passwordPlain -Force -AsPlainText

$cert = New-SelfSignedCertificate `
    -Type Custom `
    -Subject "CN=Victoria Sanjurjo" `
    -KeyUsage DigitalSignature `
    -FriendlyName "Tally MSIX Developer Certificate - Victoria Sanjurjo" `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}") `
    -NotAfter (Get-Date).AddYears(5)

Export-PfxCertificate -Cert $cert -FilePath $pfxPath -Password $securePwd | Out-Null
Export-Certificate -Cert $cert -FilePath $cerPath | Out-Null

# 3. Empaquetar archivo .msix con MakeAppx
$msixPath = "outputs\Tally_1.0.0_x64.msix"
if (Test-Path $msixPath) {
    Remove-Item -Force $msixPath
}

& $makeappx pack /d $stagingDir /p $msixPath /nv /o
if ($LASTEXITCODE -ne 0) {
    throw "MakeAppx falló con código de salida $LASTEXITCODE"
}

# 4. Firmar el paquete .msix y también copiar el .exe portable a outputs/
& $signtool sign /fd SHA256 /a /f $pfxPath /p $passwordPlain $msixPath
if ($LASTEXITCODE -ne 0) {
    throw "SignTool falló con código de salida $LASTEXITCODE"
}

Copy-Item $exePath "outputs\Tally_1.0.0_x64_Portable.exe" -Force

# Limpiar carpeta temporal de staging
Remove-Item -Recurse -Force $stagingDir

Write-Output "Empaquetado y firma MSIX completados exitosamente: $msixPath"

