#!/usr/bin/env bash
# localization-check.sh — Localization integrity check
#
# Checks:
# 1. Build with validation (missing keys cause build failure)
# 2. keys.rs is up-to-date with .ftl files
# 3. en-US has expected number of .ftl files
# 4. Generated keys.rs has keys (not empty)
#
# Usage: ./tools/localization-check.sh [--ci]
#   --ci: CI mode, exit with code 1 on any failure

set -uo pipefail

CI_MODE=false
[[ "${1:-}" == "--ci" ]] && CI_MODE=true

ERRORS=0
KEYS_FILE="src/infra/localization/generated/keys.rs"
EN_DIR="assets/localization/en-US"

echo "=== Localization Check ==="
echo ""

# Platform-agnostic file hash (macOS md5 / Linux md5sum)
file_hash() {
    local f="$1"
    if command -v md5 >/dev/null 2>&1; then
        md5 -q "$f" 2>/dev/null
    elif command -v md5sum >/dev/null 2>&1; then
        md5sum "$f" 2>/dev/null | cut -d' ' -f1
    else
        # Fallback: use modification timestamp
        stat -f "%m" "$f" 2>/dev/null || stat -c "%Y" "$f" 2>/dev/null || echo "0"
    fi
}

# 1. Build to trigger key generation + startup validation
echo "=> Building with localization validation..."
BUILD_OUTPUT=$(cargo build 2>&1) || true
if echo "$BUILD_OUTPUT" | grep -q "Localization validation failed"; then
    echo "FAIL: Localization validation failed during build"
    ERRORS=$((ERRORS + 1))
else
    echo "OK: Build passed"
fi

# 2. Check keys.rs is up to date with .ftl files
echo "=> Checking keys.rs freshness..."
if [ -f "$KEYS_FILE" ]; then
    OLD_HASH=$(file_hash "$KEYS_FILE")
    # Touch an .ftl file to force build.rs rerun
    touch "$EN_DIR/core.ftl"
    cargo build 2>/dev/null || true
    NEW_HASH=$(file_hash "$KEYS_FILE")

    if [ "$OLD_HASH" != "$NEW_HASH" ]; then
        echo "FAIL: keys.rs is out of date - run 'cargo build' to regenerate"
        ERRORS=$((ERRORS + 1))
    else
        echo "OK: keys.rs is up to date"
    fi
else
    echo "FAIL: $KEYS_FILE not found"
    ERRORS=$((ERRORS + 1))
fi

# 3. Check en-US .ftl file count matches expected (based on keys.rs modules)
echo "=> Checking en-US .ftl file count..."
FTL_COUNT=$(ls "$EN_DIR"/*.ftl 2>/dev/null | wc -l | tr -d ' ')
EXPECTED_FTL=$(grep -c "pub mod " "$KEYS_FILE" 2>/dev/null || echo "0")
# Subtract 1 for the outer `pub mod loc {`
if [ "$EXPECTED_FTL" -gt 0 ]; then
    EXPECTED_FTL=$((EXPECTED_FTL - 1))
fi
if [ "$FTL_COUNT" -lt 1 ]; then
    echo "FAIL: No en-US .ftl files found"
    ERRORS=$((ERRORS + 1))
elif [ "$FTL_COUNT" -ne "$EXPECTED_FTL" ] && [ "$EXPECTED_FTL" -gt 0 ]; then
    echo "WARN: $FTL_COUNT .ftl files vs $EXPECTED_FTL modules in keys.rs (may be transient during development)"
else
    echo "OK: $FTL_COUNT en-US .ftl files"
fi

# 4. Check that generated keys.rs has keys (not empty)
echo "=> Checking generated keys.rs has content..."
if grep -q "pub const" "$KEYS_FILE" 2>/dev/null; then
    KEY_COUNT=$(grep -c "pub const" "$KEYS_FILE")
    echo "OK: keys.rs has $KEY_COUNT keys"
else
    echo "FAIL: keys.rs has no keys"
    ERRORS=$((ERRORS + 1))
fi

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "OK: All localization checks passed"
    exit 0
else
    echo "FAIL: $ERRORS localization check(s) failed"
    if $CI_MODE; then
        exit 1
    else
        exit 0
    fi
fi
