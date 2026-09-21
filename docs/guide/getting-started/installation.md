# Installation

Xplainit is a Cargo workspace with several member crates. What you install
depends on which surface you want.

## Prerequisites

- **Rust** (stable toolchain; the workspace builds on 1.9x). Get it from
  [rustup.rs](https://rustup.rs).
- For the Python binding: **Python 3** and [maturin](https://www.maturin.rs).
- For other bindings: the usual toolchain for that language (a C/C++ compiler,
  a JDK for Java, Go, Node for the Node binding).

## Build the core and tools

From the repository root:

```sh
cargo build --release
```

This builds every workspace member, including the CLI (`xplainit-cli`) and the
dashboard (`xplainit-dashboard`).

Run the test suite to confirm your environment:

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

## Build the Python module

The Python module is built with maturin and imported as `xplainit`:

```sh
# Use a virtualenv (the repo ships a .venv you can activate):
source .venv/bin/activate
cd xplainit-python
maturin develop --release
```

Then, from the repo root, the Python suites should pass:

```sh
python test_automatic_tracing.py        # 3/3
cd xplainit-python
python test_frame_filter.py             # 11/11
python test_depth_accounting.py         # 23/23
python test_decorators.py
```

## Other bindings

- **C/C++** (`xplainit-c`): builds as `cdylib`/`staticlib`; a C header is
  generated via `cbindgen`. Needs a C/C++ compiler.
- **Java** (`xplainit-java`): JNI bindings; needs a JDK.
- **Go** (`xplainit-go` wrapper around the C FFI): needs Go.
- **Node** (`xplainit-node`): Neon/N-API addon. **Note:** the offline
  environment has no `npm`, so the Node addon cannot be rebuilt there.

See each [language guide](../guides/python.md) for specifics, and the
[project status](../../STATUS.md) for what is verified in this environment.
