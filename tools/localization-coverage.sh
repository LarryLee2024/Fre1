#!/usr/bin/env bash
# localization-coverage.sh — Translation coverage report
#
# Reports per-locale .ftl file counts and coverage relative to en-US.
#
# Usage: ./tools/localization-coverage.sh [--ci]
#   --ci: CI mode, exit with code 1 if any locale has < 50% file coverage

set -uo pipefail

CI_MODE=false
[[ "${1:-}" == "--ci" ]] && CI_MODE=true

EN_DIR="assets/localization/en-US"
LOCALE_BASE="assets/localization"
KEYS_FILE="src/infra/localization/generated/keys.rs"

echo "=== Localization Coverage Report ==="
echo ""

# Reference: en-US keys count
EN_KEYS=0
if [ -f "$KEYS_FILE" ]; then
    EN_KEYS=$(grep -c "pub const" "$KEYS_FILE" 2>/dev/null || echo "0")
fi

# en-US .ftl file count
EN_FTL_COUNT=$(ls "$EN_DIR"/*.ftl 2>/dev/null | wc -l | tr -d ' ')

echo "Reference locale: en-US"
echo "  FTL files: $EN_FTL_COUNT"
echo "  Keys: $EN_KEYS"
echo ""

# Get the list of en-US .ftl filenames (basenames)
EN_FILES=()
while IFS= read -r -d '' f; do
    EN_FILES+=("$(basename "$f")")
done < <(find "$EN_DIR" -maxdepth 1 -name '*.ftl' -print0)

# Check each locale directory
HAS_WARNINGS=false
for locale_dir in "$LOCALE_BASE"/*/; do
    [ -d "$locale_dir" ] || continue
    locale=$(basename "$locale_dir")
    [ "$locale" = "en-US" ] && continue

    LOCALE_FTL_COUNT=$(ls "$locale_dir"*.ftl 2>/dev/null | wc -l | tr -d ' ')
    if [ "$EN_FTL_COUNT" -gt 0 ]; then
        PCT=$(( LOCALE_FTL_COUNT * 100 / EN_FTL_COUNT ))
    else
        PCT=0
    fi

    # List missing files
    MISSING=()
    for en_file in "${EN_FILES[@]}"; do
        if [ ! -f "${locale_dir}${en_file}" ]; then
            MISSING+=("$en_file")
        fi
    done

    echo "Locale: $locale"
    echo "  FTL files: $LOCALE_FTL_COUNT/$EN_FTL_COUNT (${PCT}%)"
    if [ ${#MISSING[@]} -gt 0 ]; then
        echo "  Missing: ${MISSING[*]}"
        if [ "$PCT" -lt 50 ]; then
            HAS_WARNINGS=true
        fi
    fi
    echo ""
done

echo "Done."
if $HAS_WARNINGS; then
    echo "WARN: Some locales have less than 50% file coverage"
    if $CI_MODE; then
        exit 1
    fi
fi
exit 0
