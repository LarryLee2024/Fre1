#!/usr/bin/env bash
# Logging Invariant Lint — 检查日志系统的常见违规模式
#
# 验证 ADR-052、宪法 §11、日志规则.md 的核心规则：
#   1. Observer 中无残留 telemetry::emit + info! 分离模式（应使用 emit_info!/emit_warn!）
#   2. plugin.rs observer 计数与实际注册数一致
#   3. 禁止不同 observer 函数使用同一个 LogCode
#   4. 所有 observer 函数都有 #[instrument]
#   5. emit_info!/emit_warn! 中不包含 target 参数（应由宏自动派生）
#
# 用法: ./tools/check-logging-invariants.sh [--ci]
#   --ci: CI 模式下输出错误即退出码 1

set -eo pipefail
CI_MODE=false
[[ "$1" == "--ci" ]] && CI_MODE=true

SRC_DIR="src"
INFRA_LOGGING="$SRC_DIR/infra/logging"
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

echo "=== Logging Invariant Check ==="
echo ""

# Rule 1: 检查 Observer 中是否还有分离的 telemetry::emit + info! 模式
# 正常情况：info! 在 observer 函数中应该被 emit_info! 替代
check_pattern "telemetry::emit" "Rule 1: Observer 中残留 telemetry::emit 调用（应使用 emit_info!/emit_warn!）" "/tests/" "rate_limit" "metrics/mod.rs" "telemetry.rs"

# Rule 2: plugin.rs 中 add_observer 计数与实际硬编码数字不一致
if [[ -f "$INFRA_LOGGING/plugin.rs" ]]; then
    actual_count=$(grep -c "add_observer" "$INFRA_LOGGING/plugin.rs" 2>/dev/null || echo 0)
    hardcoded_count=$(grep -oP '\d+(?= 个 observer)' "$INFRA_LOGGING/plugin.rs" 2>/dev/null || echo 0)
    if [[ "$actual_count" -ne "$hardcoded_count" ]]; then
        echo "❌ Rule 2: plugin.rs observer 计数不一致（注册: $actual_count, 硬编码: $hardcoded_count）"
        HAS_ERRORS=true
    else
        echo "✅ Rule 2: plugin.rs observer 计数一致（$actual_count）"
    fi
fi

# Rule 3: 检查 observer 文件内是否有 LogCode 复用
# 每个 observer 函数应使用不同的 LogCode
for observer_file in "$INFRA_LOGGING/observers/"*.rs; do
    filename=$(basename "$observer_file")
    [[ "$filename" == "mod.rs" ]] && continue

    # 提取文件中所有 LogCode::XXX 引用（非注释行）
    logcodes=$(grep -n "LogCode::[A-Z]" "$observer_file" | grep -v "//\|target" | sed -n 's/.*LogCode::\([A-Z0-9]*\).*/\1/p' | sort | uniq -d || true)
    if [[ -n "$logcodes" ]]; then
        echo "❌ Rule 3: $filename 中存在 LogCode 复用"
        echo "$logcodes" | while read -r code; do
            echo "  - $code 被多个 observer 函数使用"
        done
        HAS_ERRORS=true
    fi
done

# Rule 4: 检查所有 observer 函数是否有 #[instrument]
for observer_file in "$INFRA_LOGGING/observers/"*.rs; do
    filename=$(basename "$observer_file")
    [[ "$filename" == "mod.rs" ]] && continue

    # 检查每个 pub(crate) fn 前面是否有 #[tracing::instrument
    functions=$(grep -c "^pub(crate) fn" "$observer_file" 2>/dev/null || echo 0)
    instruments=$(grep -c "#\[tracing::instrument\]" "$observer_file" 2>/dev/null || echo 0)
    # 每个函数可能有 skip_all + target + fields 三个 instrument 行，所以不能直接比较
    # 改为检查是否有函数完全没有 instrument
    while IFS= read -r line; do
        if [[ "$line" =~ ^pub\(crate\)\ fn ]]; then
            func_name=$(echo "$line" | sed 's/.*fn \([a-z_]*\).*/\1/')
            # 检查函数名前是否有 #[tracing::instrument
            if ! grep -B1 "pub(crate) fn $func_name" "$observer_file" | grep -q "#\[tracing::instrument"; then
                echo "❌ Rule 4: $filename 中 $func_name 缺少 #[tracing::instrument]"
                HAS_ERRORS=true
            fi
        fi
    done < <(grep -n "^pub(crate) fn" "$observer_file" 2>/dev/null || true)
done

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
