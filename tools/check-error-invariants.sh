#!/usr/bin/env bash
# Error Invariant Lint — 检查错误处理系统的常见违规模式
#
# 验证 ADR-051、.trae/rules/错误规则.md 的核心规则：
#   1. 禁止 pub use error::*;（必须 pub(crate)）
#   2. 禁止 pub use failure::*;（必须 pub(crate)）
#   3. 禁止手动 Display/Error impl（必须使用 thiserror）
#   4. 所有 Failure 枚举必须实现 RuleFailure
#   5. 禁止裸 Entity 作为错误变体字段类型
#
# 用法: ./tools/check-error-invariants.sh [--ci]
#   --ci: CI 模式下输出错误即退出码 1

set -eo pipefail
CI_MODE=false
[[ "$1" == "--ci" ]] && CI_MODE=true

SRC_DIR="src"
HAS_ERRORS=false

check_pattern() {
    local pattern="$1"
    local description="$2"
    local exclude=("${@:3}")
    local result
    result=$(grep -rn "$pattern" "$SRC_DIR" --include="*.rs" 2>/dev/null | grep -v "target/" | grep -v ".codegraph/" || true)
    for ex in "${exclude[@]}"; do
        result=$(echo "$result" | grep -v "$ex" || true)
    done
    if [[ -n "$result" ]]; then
        echo "❌ $description"
        echo "$result"
        HAS_ERRORS=true
    fi
}

echo "=== Error Invariant Check ==="
echo ""

# Rule 1: 禁止 pub use error::*
check_pattern "pub use error::\*" "Rule 1: pub use error::* (应为 pub(crate) mod error)"

# Rule 2: 禁止 pub use failure::*
check_pattern "pub use failure::\*" "Rule 2: pub use failure::* (应为 pub(crate) mod failure)"

# Rule 3: 检查手动 Display impl for Error 类型（排除非 Error 类型和测试）
# 已知例外（追踪中）：
# - attribute/mechanism/lifecycle.rs - AttributeRegistrationError (小，非 foundation)
# - modifier/mechanism/lifecycle.rs - ModifierValidationError (小，非 foundation)
# - tag/mechanism/lifecycle.rs - TagRegistrationError (小，非 foundation)
check_pattern "impl.*Display.*for.*Error" "Rule 3: 手动 Display impl 在 error 文件中" "/tests/" "shared/error/mod.rs"

# Rule 4: 所有 Failure 枚举必须实现 RuleFailure
# 检查 *Failure 枚举是否都有对应的 impl RuleFailure
failure_enums=$(grep -rn "pub enum.*Failure" "$SRC_DIR" --include="*.rs" 2>/dev/null | grep -v "target/" | grep -v ".codegraph/" || true)
if [[ -n "$failure_enums" ]]; then
    while IFS= read -r line; do
        failure_type=$(echo "$line" | sed -n 's/.*pub enum \([A-Za-z]*Failure\).*/\1/p')
        if [[ -n "$failure_type" ]]; then
            impl_count=$(grep -rn "impl RuleFailure for $failure_type" "$SRC_DIR" --include="*.rs" 2>/dev/null | grep -v "target/" | wc -l | tr -d ' ')
            if [[ "$impl_count" -eq 0 ]]; then
                echo "❌ Rule 4: $failure_type 未实现 RuleFailure trait"
                echo "$line"
                HAS_ERRORS=true
            fi
        fi
    done <<< "$failure_enums"
fi

# Rule 5: 检查 error.rs 中是否有裸 Entity 作为字段
check_pattern "Entity)" "Rule 5: error.rs 中使用裸 Entity 作为字段" "/tests/"

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
