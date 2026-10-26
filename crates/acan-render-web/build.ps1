# Builds the wasm module into www/pkg. Size-tuned profile settings are passed
# through the environment because profiles only apply from the workspace root.
$ErrorActionPreference = 'Stop'
$crate = $PSScriptRoot
$root = Resolve-Path (Join-Path $crate '..\..')
$env:CARGO_PROFILE_RELEASE_OPT_LEVEL = 's'
$env:CARGO_PROFILE_RELEASE_LTO = 'true'
$env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = '1'
$env:CARGO_PROFILE_RELEASE_PANIC = 'abort'
cargo build --manifest-path (Join-Path $root 'Cargo.toml') -p acan-render-web --release --target wasm32-unknown-unknown
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$wasm = Join-Path $root 'target\wasm32-unknown-unknown\release\acan_render_web.wasm'
wasm-bindgen --target web --no-typescript --out-dir (Join-Path $crate 'www\pkg') $wasm
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$out = Join-Path $crate 'www\pkg\acan_render_web_bg.wasm'
$bytes = [IO.File]::ReadAllBytes($out)
$ms = New-Object IO.MemoryStream
$gz = New-Object IO.Compression.GZipStream($ms, [IO.Compression.CompressionLevel]::Optimal)
$gz.Write($bytes, 0, $bytes.Length)
$gz.Close()
"wasm raw: $($bytes.Length) bytes, gzip: $($ms.ToArray().Length) bytes"
