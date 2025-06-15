# run.ps1

# Run `cargo bootimage`
Write-Host "Running cargo bootimage..."
cargo bootimage

# Check if cargo bootimage was successful
if ($LASTEXITCODE -ne 0) {
    Write-Host "cargo bootimage failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

# Define the path to QEMU and the image file
$qemuPath = "C:\Program Files\qemu\qemu-system-x86_64.exe"
$imagePath = "C:\Users\erfan\Desktop\why_os\target\x86_64-why_os\debug\bootimage-why_os.bin"

# Run QEMU with the generated boot image
Write-Host "Starting QEMU..."
& "$qemuPath" -drive format=raw,file="$imagePath"
