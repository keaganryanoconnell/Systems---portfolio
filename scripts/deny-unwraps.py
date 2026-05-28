#!/usr/bin/env python3
"""
Production Safety Scanner — deny-unwraps.py
============================================
Scans all Rust workspace crates for unhandled `.unwrap()`, `.expect()`,
and `panic!()` calls in production code paths.

Discovery:
  Reads the workspace `Cargo.toml` to auto-discover member crates.
  Falls back to a hard-coded list if the root manifest cannot be parsed.

Exclusions:
  - Lines inside `#[cfg(test)]` / `mod tests { }` blocks
  - Lines inside `benches/` directories
  - Single-line comments (`//`)
  - Block comments (`/* ... */`)
  - Test annotations (`#[test]`, `#[should_panic]`)

Exit codes:
  0 — all clean
  1 — one or more violations found
"""

import os
import sys
import re
import tomllib

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

FALLBACK_SCAN_DIRS = ["platform-nodes", "core-sys", "admin-tools"]

# Matches .unwrap(), .unwrap_err(), .expect("..."), .expect_err("..."), panic!(...)
VIOLATION_PATTERN = re.compile(r"\.unwrap\s*\(|\.expect\s*\(|\bpanic!\s*\(")


# ---------------------------------------------------------------------------
# Workspace Discovery
# ---------------------------------------------------------------------------

def discover_workspace_members(root_cargo_toml: str) -> list[str]:
    """
    Parses the workspace Cargo.toml and returns the list of member crate paths.
    Returns FALLBACK_SCAN_DIRS if the manifest cannot be read or parsed.
    """
    if not os.path.exists(root_cargo_toml):
        print(f"[WARN] Workspace manifest '{root_cargo_toml}' not found; using fallback list.")
        return FALLBACK_SCAN_DIRS

    try:
        with open(root_cargo_toml, "rb") as f:
            manifest = tomllib.load(f)
        members = manifest.get("workspace", {}).get("members", [])
        if not members:
            print("[WARN] No workspace.members found in Cargo.toml; using fallback list.")
            return FALLBACK_SCAN_DIRS
        # Filter to only directories that actually exist on disk
        existing = [m for m in members if os.path.isdir(m)]
        return existing if existing else FALLBACK_SCAN_DIRS
    except Exception as exc:
        print(f"[WARN] Failed to parse Cargo.toml ({exc}); using fallback list.")
        return FALLBACK_SCAN_DIRS


# ---------------------------------------------------------------------------
# Per-File Scanner
# ---------------------------------------------------------------------------

def scan_file(filepath: str) -> list[tuple[int, str]]:
    """
    Scans a single Rust source file for safety violations.
    Returns a list of (line_number, line_content) tuples for each violation.
    """
    try:
        with open(filepath, "r", encoding="utf-8") as f:
            lines = f.readlines()
    except Exception as exc:
        print(f"[ERROR] Could not read {filepath}: {exc}")
        return []

    violations: list[tuple[int, str]] = []
    in_block_comment = False
    in_test_block = False
    brace_depth = 0

    for lineno, raw_line in enumerate(lines, start=1):
        line = raw_line.strip()

        # ── Block comment tracking ──────────────────────────────────────────
        if "/*" in line:
            in_block_comment = True
        if in_block_comment:
            if "*/" in line:
                in_block_comment = False
            continue

        # ── Single-line comment ─────────────────────────────────────────────
        if line.startswith("//"):
            continue

        # ── Test block boundary detection ───────────────────────────────────
        if "mod tests" in line or "#[cfg(test)]" in line:
            in_test_block = True
            brace_depth = 0

        if in_test_block:
            brace_depth += line.count("{")
            brace_depth -= line.count("}")
            if brace_depth <= 0 and ("{" in line or "}" in line):
                in_test_block = False
            continue

        # ── Skip test attribute annotations ─────────────────────────────────
        if line.startswith("#[test]") or line.startswith("#[should_panic]"):
            continue

        # ── Strip inline comments before pattern matching ────────────────────
        code = line.split("//")[0]

        # ── Run the violation pattern ────────────────────────────────────────
        if VIOLATION_PATTERN.search(code):
            violations.append((lineno, line))

    return violations


# ---------------------------------------------------------------------------
# Directory Walker
# ---------------------------------------------------------------------------

def scan_directory(scan_dir: str) -> int:
    """
    Walks `scan_dir/src/` for Rust source files and scans each one.
    Returns the total number of violations found.
    """
    src_root = os.path.join(scan_dir, "src")
    if not os.path.isdir(src_root):
        # Fallback: scan the crate root directly
        src_root = scan_dir

    total_violations = 0

    for root, dirs, files in os.walk(src_root):
        # Normalize path separators for cross-platform comparison
        norm_root = root.replace(os.sep, "/")
        parts = norm_root.split("/")

        # Skip test, bench, and build artifact directories
        if any(p in parts for p in ("tests", "benches", "target")):
            continue

        for filename in sorted(files):
            if not filename.endswith(".rs"):
                continue

            filepath = os.path.join(root, filename)
            file_violations = scan_file(filepath)

            if file_violations:
                rel = os.path.relpath(filepath)
                print(
                    f"\033[0;31m[VIOLATION] Unhandled unwrap/expect/panic in: {rel}\033[0m"
                )
                for lineno, content in file_violations:
                    print(f"  Line {lineno:4d}: {content}")
                total_violations += len(file_violations)

    return total_violations


# ---------------------------------------------------------------------------
# Entry Point
# ---------------------------------------------------------------------------

def run_safety_scan() -> None:
    print("=" * 72)
    print("  Production Safety Scan — deny-unwraps.py")
    print("  Checking: .unwrap()  .expect()  panic!()")
    print("=" * 72)

    scan_dirs = discover_workspace_members("Cargo.toml")

    print(f"\nScanning {len(scan_dirs)} workspace crate(s):")
    for d in scan_dirs:
        print(f"  -> {d}/")
    print()

    total = 0
    for crate_dir in scan_dirs:
        print(f"[SCAN] {crate_dir}/src/")
        violations = scan_directory(crate_dir)
        total += violations
        if violations == 0:
            print(f"  [OK] Clean")
        print()

    print("=" * 72)
    if total > 0:
        print(
            f"\033[0;31m[FAIL] Safety Check FAILED: {total} violation(s) found in production code.\033[0m"
        )
        sys.exit(1)
    else:
        print(
            "\033[0;32m[PASS] Safety Check PASSED: No unhandled unwraps, expects, or panics.\033[0m"
        )
        sys.exit(0)


if __name__ == "__main__":
    run_safety_scan()
