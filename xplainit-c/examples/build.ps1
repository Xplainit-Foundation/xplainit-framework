# Build script for C tracing example (Windows with MSVC)
#
# This script compiles the example with MSVC

Write-Host "================================================"
Write-Host "Building Xplainit C Tracing Example (Windows)"
Write-Host "================================================"
Write-Host ""

# Note: MSVC doesn't support -finstrument-functions
# Alternative approaches for Windows:
# 1. Use Detours library for function hooking
# 2. Use ETW (Event Tracing for Windows)
# 3. Use Visual Studio Profiler API

Write-Host "Note: Windows implementation requires different approach"
Write-Host "GCC -finstrument-functions not available on MSVC"
Write-Host ""
Write-Host "Alternative options:"
Write-Host "  1. Use MinGW-w64 (GCC for Windows)"
Write-Host "  2. Use Microsoft Detours library"
Write-Host "  3. Use ETW (Event Tracing for Windows)"
Write-Host ""

# Check if MinGW is available
$mingw = Get-Command gcc -ErrorAction SilentlyContinue

if ($mingw) {
    Write-Host "Found MinGW GCC, building with instrumentation..."
    Write-Host ""
    
    gcc -finstrument-functions `
        -DXPLAINIT_DEBUG=1 `
        -g `
        -O0 `
        -Wall `
        -Wextra `
        example_traced.c `
        ..\lib\trace.c `
        -lpthread `
        -o example_traced.exe
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Build successful!"
        Write-Host ""
        Write-Host "Run with:"
        Write-Host "  `$env:XPLAINIT_DEBUG=1; .\example_traced.exe"
    } else {
        Write-Host "✗ Build failed!"
        exit 1
    }
} else {
    Write-Host "MinGW not found. Please install MinGW-w64 or use Linux/macOS for this example."
    Write-Host ""
    Write-Host "Download MinGW-w64 from: https://www.mingw-w64.org/"
}
