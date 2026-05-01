#!/usr/bin/env bash
# Build and install the Folio Python bindings into a local .venv,
# then run the pytest smoke suite. Idempotent — rerun freely.
#
# Why this script exists:
#   1. `maturin develop` requires VIRTUAL_ENV to be set; otherwise it
#      may install into Anaconda or a system Python instead of .venv.
#   2. On macOS, files written into a freshly-created venv occasionally
#      inherit the BSD `hidden` flag (likely from per-process provenance
#      tracking). Python 3.13+ skips .pth files marked hidden as a
#      security hardening, so the editable install silently fails to
#      put the package on sys.path. We clear the flag after install.

set -euo pipefail

cd "$(dirname "$0")/.."

PYTHON_BIN="${PYTHON:-python3}"

if [[ ! -d .venv ]]; then
    echo "→ creating .venv with $PYTHON_BIN"
    "$PYTHON_BIN" -m venv .venv
    .venv/bin/pip install --quiet --upgrade pip
fi

.venv/bin/pip install --quiet --upgrade maturin pytest

echo "→ building and installing folio editable into .venv"
# Run maturin in a clean env so CONDA_PREFIX (if set) doesn't conflict
# with VIRTUAL_ENV. Inherit only what's needed to compile Rust + find
# the venv interpreter.
env -i \
    HOME="$HOME" \
    PATH="$PWD/.venv/bin:/opt/homebrew/opt/rustup/bin:/usr/local/cargo/bin:$HOME/.cargo/bin:/usr/bin:/bin" \
    VIRTUAL_ENV="$PWD/.venv" \
    .venv/bin/maturin develop --release

# Clear macOS BSD hidden flag from the .pth so site.py loads it.
if [[ "$(uname)" == "Darwin" ]]; then
    PTH=".venv/lib/python"*"/site-packages/folio_docx.pth"
    if compgen -G "$PTH" > /dev/null; then
        chflags nohidden $PTH || true
    fi
fi

echo "→ running pytest"
.venv/bin/pytest python/tests "$@"
