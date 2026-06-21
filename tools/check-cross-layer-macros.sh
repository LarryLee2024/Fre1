#!/bin/bash
# check-cross-layer-macros.sh
#
# 宏治理第3原则（禁止跨层宏依赖）CI 门禁：
#   Domain/Capability 层不得直接使用 Infra 层的宏（emit_info! 等）
#
# 跨层通信应通过 Observer + 事件，而非直接宏调用。

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

violations=0

# 检查 Domain 层是否使用了 Infra 日志宏
if grep -rn 'emit_info!\|emit_warn!\|emit_debug!' src/core/domains/ --include="*.rs" 2>/dev/null | grep -v '/tests/' | grep -v 'target/'; then
    echo -e "${RED}❌ 跨层宏违规：core/domains/ 中使用了 Infra 日志宏${NC}"
    violations=$((violations + 1))
fi

# 检查 Capability 层是否使用了 Infra 日志宏
if grep -rn 'emit_info!\|emit_warn!\|emit_debug!' src/core/capabilities/ --include="*.rs" 2>/dev/null | grep -v '/tests/' | grep -v 'target/'; then
    echo -e "${RED}❌ 跨层宏违规：core/capabilities/ 中使用了 Infra 日志宏${NC}"
    violations=$((violations + 1))
fi

# 检查是否存在全局 src/macros/ 目录或 src/macros.rs（第2原则）
if [ -f src/macros.rs ]; then
    echo -e "${RED}❌ 全局宏文件违规：src/macros.rs 存在（应拆除）${NC}"
    violations=$((violations + 1))
fi

if [ -d src/macros ]; then
    echo -e "${RED}❌ 全局宏目录违规：src/macros/ 存在（禁止）${NC}"
    violations=$((violations + 1))
fi

if [ "$violations" -eq 0 ]; then
    echo -e "${GREEN}✅ 跨层宏检查通过${NC}"
    exit 0
else
    echo -e "${RED}❌ 发现 $violations 处宏治理违规${NC}"
    exit 1
fi
