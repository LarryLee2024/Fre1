#!/usr/bin/env bash
# Logging Invariant Lint — 检查日志系统的常见违规模式
#
# 验证 ADR-052、宪法 §11、日志规则.md 的核心规则：
#   1. Observer 中无残留 telemetry::emit + info! 分离模式（应使用 emit_info!/emit_warn!）
#   2. plugin.rs observer 计数与实际注册数一致
#   3. 禁止不同 observer 函数使用同一个 LogCode（审计 crafting/reaction logger 的复用 bug）
#   4. 所有 observer 函数都有 #[instrument]
#
# 用法: ./tools/check-logging-invariants.sh [--ci]
#   --ci: CI 模式下输出错误即退出码 1

set -eo pipefail
CI_MODE=false
[[ "$1" == "--ci" ]] && CI_MODE=true

SRC_DIR="src"
INFRA_LOGGING="$SRC_DIR/infra/logging"
HAS_ERRORS=false

echo "=== Logging Invariant Check ==="
echo ""

# Rule 1: 检查 Observer 中是否还有分离的 telemetry::emit + info! 模式
# 排除注释行、测试、rate_limit 模块、metrics 和 telemetry.rs 自身
rule1_result=$(grep -rn "telemetry::emit" "$SRC_DIR" --include="*.rs" 2>/dev/null \
    | grep -v "target/" | grep -v ".codegraph/" \
    | grep -v "//\|///\|//!" \
    | grep -v "tests/" \
    | grep -v "rate_limit" \
    | grep -v "metrics/mod.rs" \
    | grep -v "telemetry.rs" || true)
if [[ -n "$rule1_result" ]]; then
    echo "❌ Rule 1: Observer 中残留 telemetry::emit 调用（应使用 emit_info!/emit_warn!）"
    echo "$rule1_result"
    HAS_ERRORS=true
fi

# Rule 2: plugin.rs 中 add_observer 计数与实际硬编码数字一致
if [[ -f "$INFRA_LOGGING/plugin.rs" ]]; then
    actual_count=$(grep -c "add_observer" "$INFRA_LOGGING/plugin.rs" 2>/dev/null || echo 0)
    # 匹配中文文本中的数字，兼容不同编码
    hardcoded_count=$(grep -Eo '[0-9]+ 个 observer' "$INFRA_LOGGING/plugin.rs" 2>/dev/null | grep -Eo '^[0-9]+' || echo 0)
    if [[ "$actual_count" -ne "$hardcoded_count" ]]; then
        echo "❌ Rule 2: plugin.rs observer 计数不一致（注册: $actual_count, 硬编码: $hardcoded_count）"
        echo "   请同步更新 plugin.rs 中的硬编码 observer 数量"
        HAS_ERRORS=true
    else
        echo "✅ Rule 2: plugin.rs observer 计数一致（$actual_count）"
    fi
fi

# Rule 3: 检查 observer 文件内不同函数是否使用同一个 LogCode
# 每个函数应使用唯一的 LogCode（emit_info!/emit_warn! 的第一个参数）
for observer_file in "$INFRA_LOGGING/observers/"*.rs; do
    filename=$(basename "$observer_file")
    [[ "$filename" == "mod.rs" ]] && continue

    # 从 emit_info!/emit_warn!/emit_debug! 调用中提取 LogCode
    # 模式: emit_info!(LogCode::XXX, 或 emit_warn!(LogCode::XXX,
    unique_codes=$(grep -oP '(emit_info!|emit_warn!|emit_debug!)\(\s*LogCode::([A-Z0-9]+)' "$observer_file" \
        | sed 's/.*LogCode:://' | sort -u || true)
    code_count=$(echo "$unique_codes" | grep -c . || true)
    func_count=$(grep -c "^pub(crate) fn" "$observer_file" 2>/dev/null || echo 0)

    if [[ "$code_count" -ne "$func_count" ]] && [[ "$func_count" -gt 0 ]]; then
        echo "❌ Rule 3: $filename 中存在 LogCode 复用（函数: $func_count, 唯一 LogCode: $code_count）"
        # 找出哪些 LogCode 被重复使用
        duplicate_codes=$(grep -oP '(emit_info!|emit_warn!|emit_debug!)\(\s*LogCode::([A-Z0-9]+)' "$observer_file" \
            | sed 's/.*LogCode:://' | sort | uniq -d || true)
        if [[ -n "$duplicate_codes" ]]; then
            echo "$duplicate_codes" | while read -r code; do
                echo "  - $code 被多个函数使用"
            done
        fi
        HAS_ERRORS=true
    fi
done

# Rule 4: 检查所有 observer 函数是否有 #[instrument]
for observer_file in "$INFRA_LOGGING/observers/"*.rs; do
    filename=$(basename "$observer_file")
    [[ "$filename" == "mod.rs" ]] && continue

    while IFS= read -r line; do
        if [[ "$line" =~ ^pub\(crate\)\ fn ]]; then
            func_name=$(echo "$line" | sed 's/.*fn \([a-z_]*\).*/\1/')
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
