# ==============================================================================
# Enterprise-Grade Multi-Platform Local Verification Engine (Justfile)
# ==============================================================================
# Coordinates formatting checks, lint verification, test execution, Criterion
# benchmark compilation, static safety analysis, and supply-chain security
# audits. Any warning or error halts execution instantly.
#
# Usage:
#   just          — List all available recipes
#   just verify   — Run the complete local pipeline (mirrors CI Stage 1-3)
#   just bench    — Compile and run all Criterion benchmark harnesses
#   just watch    — Rerun tests on source changes (requires cargo-watch)
# ==============================================================================

# Configure bash as the default shell interpreter with strict execution parameters
set shell := ["bash", "-euo", "pipefail", "-c"]

# ---------------------------------------------------------------------------
# Dynamic compilation scope.
#
# Linux runners can compile everything including epoll/libc platform-nodes.
# Windows and macOS exclude platform-nodes to prevent kernel-syscall failures.
# ---------------------------------------------------------------------------
cargo_workspace_flags := if os() == "linux" {
    "--workspace --all-features"
} else {
    "--workspace --all-features --exclude platform-nodes --exclude container-engine"
}

# List all available recipes
default:
    @just --list

# ==============================================================================
# Primary Pipeline Orchestrator
# ==============================================================================

# Run the complete local verification loop — mirrors CI Stages 1, 2, and 3.
# Truncates execution instantly if any recipe returns a non-zero exit code.
verify: fmt lint test check-bench static-analysis supply-chain-audit
    @echo "========================================================================"
    @echo -e "\033[0;32mSUCCESS: All verification stages completed with zero violations.\033[0m"
    @echo "========================================================================"

# ==============================================================================
# Stage 1 Recipes — Format & Lint
# ==============================================================================

# 1. Format: Asserts code styling constraints workspace-wide
fmt:
    @echo "=== [1/6] Running cargo fmt checker ==="
    cargo fmt --all -- --check

# 2. Lint: Enforces strict clippy compiler warnings as compilation errors
lint:
    @echo "=== [2/6] Running cargo clippy audit ==="
    cargo clippy {{cargo_workspace_flags}} --all-targets -- -D warnings

# ==============================================================================
# Stage 2 Recipes — Test & Build
# ==============================================================================

# 3. Test: Executes all workspace unit, integration, and doc-tests concurrently
test:
    @echo "=== [3/6] Executing workspace test suites ==="
    cargo test {{cargo_workspace_flags}} --all-targets

# 4. Check-Bench: Asserts compilation validity of all Criterion benchmark harnesses
#    (mirrors the CI Stage 3 bench gate — ensures bench code never drifts from the API)
check-bench:
    @echo "=== [4/6] Verifying Criterion benchmark harnesses ==="
    cargo bench {{cargo_workspace_flags}} --all-targets --no-run

# ==============================================================================
# Stage 3 Recipes — Safety & Supply-Chain
# ==============================================================================

# 5. Static-Analysis: Scans production code to deny unhandled unwraps, expects, or panics
static-analysis:
    @echo "=== [5/6] Executing static safety scan ==="
    @python scripts/deny-unwraps.py

# 6. Supply-Chain-Audit: Validates licensing, lockfile integrity, and CVE advisories
supply-chain-audit:
    @echo "=== [6/6] Executing supply-chain security audit ==="
    @if ! command -v cargo-deny &> /dev/null; then \
        echo -e "\033[0;31mError: 'cargo-deny' command utility not found in PATH.\033[0m" >&2; \
        echo "Please install it locally: cargo install --locked cargo-deny" >&2; \
        exit 1; \
    fi
    cargo deny check

# ==============================================================================
# Developer Convenience Recipes
# ==============================================================================

# Run all Criterion benchmarks with full output (actual measurements, not --no-run)
bench:
    @echo "=== Running Criterion benchmark harnesses (full execution) ==="
    cargo bench {{cargo_workspace_flags}}

# Watch mode: re-run tests automatically on source file changes
# Requires: cargo install cargo-watch
watch:
    @echo "=== Entering watch mode (Ctrl-C to exit) ==="
    cargo watch -x "test {{cargo_workspace_flags}} --all-targets"

# Clean all build artifacts (frees disk space)
clean:
    @echo "=== Cleaning build artifacts ==="
    cargo clean

# Print the current resolved workspace dependency tree
deps:
    @echo "=== Workspace dependency tree ==="
    cargo tree --workspace

# Check for outdated dependencies (requires: cargo install cargo-outdated)
outdated:
    @echo "=== Checking for outdated dependencies ==="
    @if ! command -v cargo-outdated &> /dev/null; then \
        echo -e "\033[0;33mWarning: 'cargo-outdated' not found. Install: cargo install cargo-outdated\033[0m" >&2; \
        exit 0; \
    fi
    cargo outdated --workspace

# ==============================================================================
# Container Engine Recipes
# ==============================================================================

# Download Alpine mini-rootfs for container runtime tests
container-rootfs:
    @echo "=== Downloading Alpine minirootfs for container tests ==="
    @bash container-engine/scripts/download-rootfs.sh

# Build container engine on Linux only
container-build:
    @echo "=== Building container-engine ==="
    cargo build -p container-engine

# Run container integration tests (requires root)
container-test:
    @echo "=== Running container-engine integration tests ==="
    @sudo cargo test -p container-engine --test integration -- --nocapture

# Run container security boundary tests (requires root)
container-security:
    @echo "=== Running container-engine security tests ==="
    @sudo cargo test -p container-engine --test security -- --nocapture

# Full container lifecycle smoke test (requires root + Alpine rootfs)
container-smoke:
    @echo "=== Running container-engine smoke test ==="
    @echo "Requires: sudo + Alpine rootfs at container-engine/tests/rootfs/"
    @sudo cargo run -p container-engine -- run ./container-engine/tests/rootfs/ --readonly --hostname smoke-test -- /bin/echo "Container engine: OK"
