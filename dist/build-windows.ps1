
$ARCH="x86_64"
$PLATFORM="windows"

$FEATURES="built","audio","iced_gif","manual"

# Compile application with features
cargo build -p cvte-mes --release --features=$FEATURES

# Create artifact folder, remove residual files
mkdir -Force -p "target/release/artifact"
Remove-Item "target/release/artifact/*"

# Create archive folder, remove residual files
mkdir -Force -p "target/release/archive"
Remove-Item "target/release/archive/*"

# move and rename cvte-mes.exe to cvte-mes in archive folder
Move-Item -Force "target/release/cvte-mes.exe" "target/release/archive/cvte-mes.exe"

# Copy extras to archive folder
Copy-Item -Force "./assets/manual.txt" "target/release/archive/"
Copy-Item -Force "LICENSE" "target/release/archive/"

# Get version number
$VERSION = cargo run -p cvte-mes --release -- --version | Out-String
$VERSION = $VERSION.Trim() # trim whitespace, includes \n

# Compress archive contents of archive folder to a zip file
$ARTIFACT_PATH = "target/release/artifact/cvte-mes-v$VERSION-$PLATFORM-$ARCH.zip"
Compress-Archive -Force -Path "target/release/archive/*" -DestinationPath $ARTIFACT_PATH 

Write-Output "Packaged archive: $ARTIFACT_PATH"
