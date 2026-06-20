#!/usr/bin/env bash
# Architecture Budget & Fitness Function -- 检查架构预算违规
#
# 验证架构边界约束，包括：
#   1. 文件大小限制（core ≤500 行, shared ≤300 行）
#   2. 函数大小限制（>50 行软警告, >100 行违规）
#   3. 领域依赖边界（禁止 Domain 间直接引用内部类型，events 例外）
#   4. 架构预算清单（core >500 行, shared >300 行的文件列表）
#
# 用法: ./tools/check-architecture-budget.sh [--ci]
#   --ci: CI 模式下输出错误即退出码 1

set -o pipefail

CI_MODE=false
[[ "$1" == "--ci" ]] && CI_MODE=true

SRC_DIR="src"
HAS_ERRORS=false
HAS_WARNINGS=false

# 颜色定义（如果终端支持）
COLOR_RED=''
COLOR_YELLOW=''
COLOR_CYAN=''
COLOR_RESET=''
if [[ -t 1 ]]; then
    COLOR_RED='\033[0;31m'
    COLOR_YELLOW='\033[1;33m'
    COLOR_GREEN='\033[0;32m'
    COLOR_CYAN='\033[0;36m'
    COLOR_RESET='\033[0m'
fi

info()    { echo -e "${COLOR_CYAN}[INFO]${COLOR_RESET} $*"; }
warn()    { echo -e "${COLOR_YELLOW}[WARN]${COLOR_RESET} $*"; }
error()   { echo -e "${COLOR_RED}[ERROR]${COLOR_RESET} $*"; }
success() { echo -e "${COLOR_GREEN}[PASS]${COLOR_RESET} $*"; }
separator() { echo ""; echo "----------------------------------------"; echo ""; }

# 检查命令是否存在
for cmd in find grep wc awk sort; do
    if ! command -v "$cmd" &>/dev/null; then
        error "缺少依赖命令：$cmd"
        exit 1
    fi
done

# ============================================================
# 检查文件大小（不含测试目录和测试文件）
# ============================================================
check_file_sizes() {
    info "检查 1：文件大小限制（core ≤500 行, shared ≤300 行）"

    local found_issues=false
    local dir="$1"
    local max_lines="$2"
    local label="$3"
    local tempfile
    tempfile=$(mktemp)

    # 查找生产代码文件（排除 tests 目录和测试文件）
    find "$dir" -name "*.rs" -not -path "*/tests/*" -not -name "*_test.rs" -not -name "mod.rs" 2>/dev/null \
        | while IFS= read -r file; do
            # 用 wc -l 统计行数，去除前导空格
            lines=$(wc -l < "$file" | tr -d ' ')
            if [[ "$lines" -gt "$max_lines" ]]; then
                echo "$file:$lines"
            fi
        done > "$tempfile"

    if [[ -s "$tempfile" ]]; then
        found_issues=true
        while IFS=: read -r file lines; do
            # 计算相对于 $SRC_DIR 的路径
            local rel_path="${file#$SRC_DIR/}"
            error "$rel_path: ${lines} 行（超过 ${label} 上限 ${max_lines} 行）"
        done < "$tempfile"
        HAS_ERRORS=true
    fi

    if ! $found_issues; then
        info "所有 ${label} 文件均在 ${max_lines} 行以内"
    fi

    rm -f "$tempfile"
}

# ============================================================
# 检查函数大小
# ============================================================
check_function_sizes() {
    info "检查 2：函数大小限制（>50 行软警告, >100 行违规）"

    local found_violation=false
    local found_warning=false
    local tempfile
    tempfile=$(mktemp)

    # 用 awk 解析函数边界，统计函数体行数
    # 策略：
    #   1. 匹配 `fn ` 开头的函数定义（排除注释行、宏调用）
    #   2. 追踪花括号深度
    #   3. 深度归零时函数结束，报告行数
    find "$SRC_DIR" -name "*.rs" -not -path "*/tests/*" -not -name "*_test.rs" 2>/dev/null \
        | while IFS= read -r file; do
            awk '
            BEGIN {
                in_func = 0
                func_name = ""
                func_line = 0
                depth = 0
                body_lines = 0
                has_body = 0
            }

            # 跳过注释行和宏调用
            /^\s*\/\// { next }
            /^\s*#/ { next }

            # 匹配函数定义：pub fn, pub(crate) fn, fn, pub(super) fn 等
            /^\s*(pub(\([^)]*\))?\s+)?fn\s+[a-zA-Z_]/ {
                # 如果已经在跟踪一个函数则先结算
                if (in_func) {
                    check_result()
                }

                in_func = 1
                func_name = $0
                func_line = NR
                depth = 0
                body_lines = 0
                has_body = 0

                # 检查当前行是否有函数体开始
                count_braces()
                if (depth > 0) {
                    has_body = 1
                    body_lines = 1
                }
                next
            }

            in_func {
                body_lines++
                count_braces()
                if (depth <= 0 && has_body) {
                    check_result()
                    in_func = 0
                }
            }

            END {
                if (in_func) check_result()
            }

            function count_braces() {
                line = $0
                n = length(line)
                for (i = 1; i <= n; i++) {
                    c = substr(line, i, 1)
                    if (c == "{") { depth++; has_body = 1 }
                    if (c == "}") depth--
                }
                # 防止负深度（文件末尾 } 与外部对齐）
                if (depth < 0) depth = 0
            }

            function check_result() {
                if (has_body && body_lines > 50) {
                    printf "%s:%d %d %d\n", FILENAME, func_line, body_lines, (body_lines > 100 ? 1 : 0)
                }
                in_func = 0
                func_name = ""
                func_line = 0
                depth = 0
                body_lines = 0
                has_body = 0
            }
            ' "$file" 2>/dev/null
        done > "$tempfile"

    while IFS=' ' read -r location lines is_violation; do
        local rel_path="${location#./}"
        rel_path="${rel_path#$SRC_DIR/}"
        if [[ "$is_violation" == "1" ]]; then
            error "函数超过 100 行: $rel_path （${lines} 行）"
            found_violation=true
            HAS_ERRORS=true
        else
            warn "函数超过 50 行: $rel_path （${lines} 行）"
            found_warning=true
            HAS_WARNINGS=true
        fi
    done < "$tempfile"

    if ! $found_violation && ! $found_warning; then
        info "未发现超长函数"
    fi

    if $found_warning && ! $found_violation; then
        info "有超 50 行的函数（软警告），无超 100 行的函数"
    fi

    rm -f "$tempfile"
}

# ============================================================
# 检查领域依赖边界
# ============================================================
check_domain_dependencies() {
    info "检查 3：领域依赖边界（禁止 Domain 间直接引用内部类型）"

    local DOMAINS_DIR="$SRC_DIR/core/domains"
    if [[ ! -d "$DOMAINS_DIR" ]]; then
        warn "领域目录不存在: $DOMAINS_DIR"
        return
    fi

    local found_violation=false
    local tempfile
    tempfile=$(mktemp)

    # 获取所有领域子目录名
    for domain_dir in "$DOMAINS_DIR"/*/; do
        local domain_name
        domain_name=$(basename "$domain_dir")

        # 在每个领域的 .rs 文件中搜索对其他领域的引用
        # 排除：events 导入、self 导入、注释行
        find "$domain_dir" -name "*.rs" -not -path "*/tests/*" 2>/dev/null \
            | while IFS= read -r file; do
                grep -n "use crate::core::domains::" "$file" 2>/dev/null || true \
                    | grep -v "//.*use crate::core::domains::" \
                    | grep -v "///.*use crate::core::domains::" \
                    | while IFS=: read -r line_no content; do
                        # 提取被导入的领域名
                        local imported_domain
                        imported_domain=$(echo "$content" \
                            | sed -n 's/.*use crate::core::domains::\([a-zA-Z_]*\).*/\1/p')

                        # 跳过自引用（领域导入自己）
                        [[ "$imported_domain" == "$domain_name" ]] && continue

                        # 跳过 events 导入（P0 规则：Domain 间通过 Event 通信）
                        if echo "$content" | grep -q "::events"; then
                            continue
                        fi
                        # 也允许 `use crate::core::domains::combat::events` 这类形式
                        if echo "$content" | grep -qE "(from|use )${imported_domain}::events"; then
                            continue
                        fi

                        echo "$file:$line_no: $content"
                        found_violation=true
                    done
            done
    done > "$tempfile"

    if [[ -s "$tempfile" ]]; then
        error "发现 Domain 间直接引用（仅 events 允许）："
        while IFS= read -r line; do
            local rel_path="${line#$SRC_DIR/}"
            echo "  $rel_path"
        done < "$tempfile"
        HAS_ERRORS=true
    else
        info "未发现 Domain 间违规引用"
    fi

    rm -f "$tempfile"
}

# ============================================================
# 架构预算清单
# ============================================================
check_architecture_budget() {
    info "检查 4：架构预算清单"

    local core_budget=500
    local shared_budget=300
    local has_over_budget=false
    local tempfile
    tempfile=$(mktemp)

    echo ""
    info "--- Core 超预算文件（>${core_budget} 行） ---"

    find "$SRC_DIR/core" -name "*.rs" -not -path "*/tests/*" -not -name "*_test.rs" -not -name "mod.rs" 2>/dev/null \
        | while IFS= read -r file; do
            lines=$(wc -l < "$file" | tr -d ' ')
            if [[ "$lines" -gt "$core_budget" ]]; then
                rel_path="${file#$SRC_DIR/}"
                echo "  $rel_path: ${lines} 行"
                has_over_budget=true
            fi
        done > "$tempfile"

    if [[ -s "$tempfile" ]]; then
        cat "$tempfile"
        HAS_WARNINGS=true
    else
        info "  Core 层无超预算文件"
    fi

    echo ""
    info "--- Shared 超预算文件（>${shared_budget} 行） ---"

    find "$SRC_DIR/shared" -name "*.rs" -not -path "*/tests/*" -not -name "*_test.rs" -not -name "mod.rs" 2>/dev/null \
        | while IFS= read -r file; do
            lines=$(wc -l < "$file" | tr -d ' ')
            if [[ "$lines" -gt "$shared_budget" ]]; then
                rel_path="${file#$SRC_DIR/}"
                echo "  $rel_path: ${lines} 行"
                has_over_budget=true
            fi
        done > "$tempfile"

    if [[ -s "$tempfile" ]]; then
        cat "$tempfile"
        HAS_WARNINGS=true
    else
        info "  Shared 层无超预算文件"
    fi

    if ! $has_over_budget; then
        info "所有文件均在预算范围内"
    fi

    rm -f "$tempfile"
}

# ============================================================
# 主流程
# ============================================================
echo ""
echo "=== Architectural Fitness Function ==="
echo ""

# 检查 1: 文件大小
check_file_sizes "$SRC_DIR/core" 500 "Core"
separator

# 检查 2: 函数大小
check_function_sizes
separator

# 检查 3: 领域依赖
check_domain_dependencies
separator

# 检查 4: 架构预算
check_architecture_budget
separator

# 检查 5: 系统互调禁令
check_system_mutual_calls() {
    info "检查 5：系统互调禁令（禁止系统函数互相直接调用）"
    echo ""

    local found_violation=false
    local tempfile
    tempfile=$(mktemp)

    # 策略：查找 `fn ` 定义的系统函数，然后检查这些函数是否被其他函数直接调用
    # 系统函数命名约定：一般是 `fn *_system` 或以 `on_` 开头的 Observer
    # 更精确：排除 Bevy 调度注册和测试代码

    # 收集所有系统函数名
    grep -rn "^pub(crate) fn\|^pub fn\|^fn " "$SRC_DIR" --include="*.rs" 2>/dev/null \
        | grep -v "target/" \
        | grep -v "/tests/" \
        | grep -v "test_" \
        | grep -E "_system\|_handler\|on_" \
        | sed 's/.*fn \([a-zA-Z_0-9]*\).*/\1/' \
        | sort -u > "$tempfile"

    # 对每个系统函数，检查它是否被其他函数直接调用（除了在 plugin.rs 中的注册）
    while IFS= read -r func_name; do
        [[ -z "$func_name" ]] && continue

        # 查找调用该函数的位置
        callers=$(grep -rn "\b${func_name}(" "$SRC_DIR" --include="*.rs" 2>/dev/null \
            | grep -v "target/" \
            | grep -v "/tests/" \
            | grep -v "//\|///\|//!" \
            | grep -v "add_observer\|add_systems\|add_system" \
            | grep -v "\.observe(\|\.add_systems" \
            | grep -v "Self::\|self\." \
            || true)

        if [[ -n "$callers" ]]; then
            # 检查调用者是否来自定义该函数以外的文件
            while IFS=: read -r caller_file caller_line caller_content; do
                # 忽略在同一个文件内的自身定义
                if grep -q "^\s*\(pub\|pub(crate)\)\?\s*fn ${func_name}" "$caller_file" 2>/dev/null; then
                    continue
                fi
                # 忽略在同一个文件内的自身调用
                def_file=$(grep -l "^\s*\(pub\|pub(crate)\)\?\s*fn ${func_name}" "$caller_file" 2>/dev/null || true)
                if [[ -z "$def_file" ]]; then
                    rel_file="${caller_file#$SRC_DIR/}"
                    echo "  ${rel_file}:${caller_line} — ${caller_content}" >> "$tempfile.tmp"
                    found_violation=true
                fi
            done <<< "$callers"
        fi
    done < "$tempfile"

    if [[ -f "$tempfile.tmp" ]] && [[ -s "$tempfile.tmp" ]]; then
        error "发现系统函数被其他模块直接调用："
        cat "$tempfile.tmp"
        echo ""
        info "提示：系统间必须通过事件通信（trigger + Observer），禁止直接调用系统函数"
        echo ""
        HAS_ERRORS=true
    else
        success "零违规 — 所有系统通过事件通信，未发现直接调用"
    fi

    rm -f "$tempfile" "$tempfile.tmp"
}

check_system_mutual_calls
separator

# ============================================================
# 汇总
# ============================================================
echo ""
if $HAS_ERRORS; then
    echo -e "${COLOR_RED}违规总结：发现硬性违规${COLOR_RESET}"
    if $CI_MODE; then
        exit 1
    fi
elif $HAS_WARNINGS; then
    echo -e "${COLOR_YELLOW}违规总结：仅发现软警告（预算超限），无硬性违规${COLOR_RESET}"
else
    echo -e "${COLOR_CYAN}违规总结：未发现任何问题${COLOR_RESET}"
fi
exit 0
