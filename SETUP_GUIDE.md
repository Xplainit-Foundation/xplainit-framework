# 🚀 Xplainit Framework - Quick Setup Guide

## ⚡ Prerequisites Installation

### Step 1: Install Rust (Required)

**Windows (PowerShell)**:
```powershell
# Download and run rustup installer
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe
```

Or visit: https://rustup.rs/ and follow instructions

**After installation, restart your terminal and verify**:
```powershell
cargo --version
rustc --version
```

### Step 2: Install Build Tools

**Windows - Install Microsoft C++ Build Tools**:
1. Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Install "Desktop development with C++" workload

**Or install Visual Studio 2022 Community Edition** (includes build tools)

### Step 3: Install Additional Tools

```powershell
# After Rust is installed
cargo install cargo-edit
cargo install cargo-watch
cargo install cargo-tree
```

---

## 🏗️ Project Initialization (Run After Rust Installation)

```powershell
# Navigate to project directory
cd "c:\Users\siter\Desktop\Xplainit Framework"

# Initialize Cargo workspace
cargo init --lib xplainit-core
cargo init --lib xplainit-python
cargo init --lib xplainit-node
cargo init --lib xplainit-c
cargo init --lib xplainit-java
cargo init --lib xplainit-go
cargo init --bin xplainit-cli

# Create directory structure
New-Item -ItemType Directory -Force -Path "docs\book"
New-Item -ItemType Directory -Force -Path "docs\api"
New-Item -ItemType Directory -Force -Path "docs\examples"
New-Item -ItemType Directory -Force -Path "tests\fixtures"
New-Item -ItemType Directory -Force -Path "tests\cross-lang"
New-Item -ItemType Directory -Force -Path ".github\workflows"
```

---

## 🔧 Quick Start (If You Have Rust Already)

If Rust is already installed, run this script:

```powershell
# Save as: setup.ps1 and run: .\setup.ps1

$projectRoot = "c:\Users\siter\Desktop\Xplainit Framework"
Set-Location $projectRoot

Write-Host "🚀 Initializing Xplainit Framework..." -ForegroundColor Green

# Initialize crates
$crates = @("xplainit-core", "xplainit-python", "xplainit-node", 
            "xplainit-c", "xplainit-java", "xplainit-go", "xplainit-cli")

foreach ($crate in $crates) {
    if ($crate -eq "xplainit-cli") {
        cargo init --bin $crate
    } else {
        cargo init --lib $crate
    }
    Write-Host "✓ Created $crate" -ForegroundColor Cyan
}

# Create directory structure
$dirs = @(
    "docs\book", "docs\api", "docs\examples",
    "tests\fixtures", "tests\cross-lang",
    ".github\workflows"
)

foreach ($dir in $dirs) {
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Write-Host "✓ Created $dir" -ForegroundColor Cyan
}

Write-Host "`n✅ Project structure created!" -ForegroundColor Green
Write-Host "Next: Run .\build.ps1 to set up workspace configuration" -ForegroundColor Yellow
```

---

## 📝 What to Do Next

1. **Install Rust** (if not installed): Run rustup installer
2. **Restart terminal** (important!)
3. **Run setup script** (above)
4. **Continue building** with the framework

---

## ✅ Verification

Check if everything is ready:

```powershell
# Should all work without errors
cargo --version
rustc --version
git --version

# Navigate to project
cd "c:\Users\siter\Desktop\Xplainit Framework"
```

---

**Status**: Prerequisites needed before continuing  
**Next Step**: Install Rust, then continue with project setup 🚀
