#!/usr/bin/env bash
# Identity Invariant Lint — 检查 ID 系统中的常见违规模式
#
# 验证 id-taxonomy.md §10 的 13 条铁律：
#   1. ID 不参与业务逻辑（无 .id() == SomeId(N)）
#   2. ID 不隐含排序（无 sort_by_key(|x| x.id())）
#   6. 无 Null ID（无 SomeId(0) 或 id == 0）
#
# 用法: ./tools/check-identity-invariants.sh [--ci]
#   --ci: CI 模式下输出错误即退出码 1

set -eo pipefail
CI_MODE=false
[[ "$1" == "--ci" ]] && CI_MODE=true

SRC_DIR="src"
HAS_ERRORS=false

check_pattern() {
    local pattern="$1"
    local description="$2"
    local result
    result=$(grep -rn "$pattern" "$SRC_DIR" --include="*.rs" 2>/dev/null | grep -v "/tests/" | grep -v "target/" || true)
    if [[ -n "$result" ]]; then
        echo "❌ $description"
        echo "$result"
        HAS_ERRORS=true
    fi
}

echo "=== Identity Invariant Check ==="
echo ""

# Rule 1: ID 不参与业务逻辑
check_pattern "\.id()\s*==" "Rule 1: ID 用于业务逻辑比较 (id() == N)"
check_pattern "==\s*SomeId(" "Rule 1: ID 硬编码比较 (== SomeId(...))"

# Rule 2: ID 不隐含排序
check_pattern "sort_by_key.*\.id()" "Rule 2: ID 用于排序 (sort_by_key id)"

# Rule 6: Null ID 反模式
check_pattern "SomeId(0)" "Rule 6: Null ID 反模式 (SomeId(0))"
check_pattern "\.id\s*==\s*0" "Rule 6: Null ID 反模式 (.id == 0)"

# Rule 7: 跨层隐式转换
check_pattern "impl\s+From<.*Id>\s+for\s+.*Id" "Rule 7: 跨层 ID 隐式转换 (From<IdA> for IdB)"

echo ""
if $HAS_ERRORS; then
    echo "⚠️  发现违规模式"
    if $CI_MODE; then
        exit 1
    fi
else
    echo "✅ 未发现违规模式"
fi
exit 0
