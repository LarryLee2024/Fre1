#!/usr/bin/env bash
# UI Architecture Compliance Check
#
# CI script that verifies src/ui/ code follows Presentation Layer
# architecture rules.
#
# Usage: ./scripts/ui-arch-check.sh [--strict]
#
# See docs/06-ui/01-architecture/architecture.md §10

set -euo pipefail

UI_DIR="src/ui"
HAS_ERRORS=0

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  UI Architecture Compliance Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ──────────────────────────────────────────────
# Rule 1: No direct Query<&DomainComponent> in src/ui/
# ──────────────────────────────────────────────
# Domain components (Health, Spellbook, Inventory, etc.) must not be
# queried directly in UI code. Use ViewModel (Res<BattleHudVm>) instead.
# UI-internal Queries (ButtonState, MenuAction, etc.) are allowed.

echo ""
echo -e "${YELLOW}[RULE 1]${NC} No raw Domain Component Query in src/ui/"

# Check for Query<& pattern, excluding known UI-internal components
# UI-internal component whitelist:
#   ButtonState, ButtonInteraction, ButtonVariant, ButtonClicked
#   MenuAction, BattleAction, ActionType, CharacterAction
#   SkillSlotAction, InventoryItemAction, ModalButtonRole
#   PanelState, ListState, ProgressBarState, TextWidget
#   BuffIconState, CharacterCardState, SkillSlotState
#   InventoryItemRowState, ActionMenuState
#   ModalState, ModalVariant
#   Interaction, Node, Button
#   ChildOf, Parent, Name, Text, TextFont, TextColor
VIOLATIONS=$(grep -rn 'Query<&' "$UI_DIR" \
    --include='*.rs' \
    --exclude-dir='tests' \
    --exclude='*test*' \
    | grep -vE 'Query<&(mut )?(Button|Menu|Battle|Action|Character|SkillSlot|Inventory|BuffIcon|Panel|List|ProgressBar|Text|Modal|Interaction|Node|ChildOf|Parent|Name|FocusGroup|Dirty|UiBinding|Focusable|FocusManager)' \
    | grep -v '///' \
    || true)

if [ -n "$VIOLATIONS" ]; then
    echo -e "${RED}  FAIL: Found raw Query<&DomainComponent> in src/ui/${NC}"
    echo ""
    echo "$VIOLATIONS"
    echo ""
    echo -e "  ${YELLOW}Fix:${NC} Use ViewModel (Res<BattleHudVm>) instead of direct Query."
    echo -e "  ${YELLOW}Fix:${NC} For Projection layer, use Domain QueryParam wrappers."
    HAS_ERRORS=1
else
    echo -e "${GREEN}  PASS${NC}"
fi

# ──────────────────────────────────────────────
# Rule 2: No hardcoded text strings (must use LocalizedText)
# ──────────────────────────────────────────────
# User-visible strings must use LocalizedText::static_text(key) or
# spawn_localized_text() instead of Text::new("hardcoded string").

echo ""
echo -e "${YELLOW}[RULE 2]${NC} No hardcoded user-visible text in ui factories"

# Check for spawn_text with string literals in factory files (likely not localized)
FACTORY_VIOLATIONS=$(grep -rn 'spawn_text.*"[A-Z]' "$UI_DIR" \
    --include='*factory*.rs' \
    --exclude='*test*' \
    || true)

if [ -n "$FACTORY_VIOLATIONS" ]; then
    echo -e "${YELLOW}  WARN: Found spawn_text with capitalized string (might miss localization)${NC}"
    echo ""
    echo "$FACTORY_VIOLATIONS"
    echo ""
    echo -e "  ${YELLOW}Review:${NC} Consider using spawn_localized_text() with a UiTextKey."
    # Not a hard failure for warnings
fi

# ──────────────────────────────────────────────
# Summary
# ──────────────────────────────────────────────
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ "$HAS_ERRORS" -eq 0 ]; then
    echo -e "${GREEN}  All UI architecture checks passed.${NC}"
else
    echo -e "${RED}  Some UI architecture checks failed.${NC}"
    echo -e "${RED}  Fix above violations before merging.${NC}"
fi
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
exit $HAS_ERRORS
