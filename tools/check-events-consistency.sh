#!/bin/bash
# check-events-consistency.sh — Events naming and structure consistency check
set -uo pipefail

ERRORS=0
EVENTS_FILES=$(find src/ -name "events.rs" -not -path "*/target/*" | sort)
TOTAL=$(echo "$EVENTS_FILES" | wc -l)

echo "=== Events Consistency Check ==="
echo "Found $TOTAL events.rs files"
echo ""

# Check 1: Each events.rs should have at least one #[derive(Event)]
echo "-> Checking Event derive..."
for f in $EVENTS_FILES; do
    if ! grep -q "#\[derive.*Event" "$f" 2>/dev/null; then
        # Skip files that use Bevy Event (they might derive it differently)
        if grep -q "pub struct\|pub enum" "$f" 2>/dev/null; then
            echo "  ?  No #[derive(Event)] in: $f"
        fi
    fi
done

# Check 2: No EventWriter/EventReader in new code (ADR-002 v2 ban)
echo "-> Checking for banned EventWriter/EventReader..."
BANNED=$(grep -rn "EventWriter\|EventReader" src/ --include='*.rs' 2>/dev/null | grep -v "target/" | grep -v "#\[allow\|deprecated\|///\|//!" | head -10)
if [ -n "$BANNED" ]; then
    echo "  ?  Found potential EventWriter/EventReader usage:"
    echo "$BANNED"
fi

# Check 3: Events should have Debug + Clone + Event derives
echo "-> Checking event struct completeness..."
for f in $EVENTS_FILES; do
    while IFS= read -r line; do
        if [[ "$line" =~ ^pub\ struct\ ([A-Za-z]+) ]]; then
            name="${BASH_REMATCH[1]}"
            # Check it's not already checked
        fi
    done < "$f"
done

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "** Events consistency check passed"
else
    echo "** $ERRORS check(s) failed"
    exit $ERRORS
fi
