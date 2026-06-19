//! 日志类型编码，按域分组，三位数字递增。
//!
//! 格式: `{域前缀}{3位数字}`，如 `BAT001`。
//! 用于替代文本搜索，支持 AI 可搜索的结构化日志。

/// 日志类型编码，按域分组，三位数字递增。
///
/// 每个变体对应一个唯一的日志事件类型。
/// 格式: `{域前缀}{3位数字}`，如 `BAT001`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogCode {
    // ─── BAT — Combat（战斗）───
    /// 战斗开始
    BAT001,
    /// 战斗结束
    BAT002,
    /// 回合开始
    BAT003,
    /// 回合结束
    BAT004,
    /// 单位回合开始
    BAT005,
    /// 单位回合结束
    BAT006,
    /// 伤害结算完成
    BAT007,
    /// 单位死亡
    BAT008,
    /// 先攻检定完成
    BAT009,
    /// 胜负条件满足
    BAT010,

    // ─── TAC — Tactical（战术/网格）───
    /// 单位移动完成
    TAC001,
    /// 夹击判定完成
    TAC002,
    /// 背刺判定完成
    TAC003,
    /// 掩体判定完成
    TAC004,
    /// 单位位置变更
    TAC005,

    // ─── TER — Terrain（地形）───
    /// 单位进入格子
    TER001,
    /// 格子表面变化
    TER002,
    /// 陷阱触发
    TER003,
    /// 地形效果施加
    TER004,

    // ─── ABL — Ability（技能）───
    /// 技能成功激活
    ABL001,
    /// 技能执行完毕
    ABL002,
    /// 技能被取消/打断
    ABL003,
    /// 冷却开始
    ABL004,

    // ─── EFF — Effect（效果）───
    /// 效果成功施加
    EFF001,
    /// 效果移除
    EFF002,
    /// 周期效果 Tick
    EFF003,
    /// 效果因免疫被阻止
    EFF004,
    /// 执行计算完成
    EFF005,
    /// 执行计算失败
    EFF006,
    /// 自定义执行注册
    EFF007,
    /// 堆叠达到上限触发溢出
    EFF008,

    // ─── TAG — Tag（标签）───
    /// 标签授予实体
    TAG001,
    /// 标签从实体移除
    TAG002,
    /// 标签层级变更
    TAG003,
    /// 标签查询评估完成
    TAG004,

    // ─── MOD — Modifier（修改器）───
    /// 修改器注册到容器
    MOD001,
    /// 修改器从容器移除
    MOD002,
    /// 修改器被高优先级抑制
    MOD003,
    /// 检测到过期修改器
    MOD004,

    // ─── AGG — Aggregator（聚合计算）───
    /// 属性聚合计算完成
    AGG001,
    /// 属性被标记需要重算
    AGG002,
    /// 快照拍摄完成
    AGG003,
    /// 检测到聚合闭环
    AGG004,

    // ─── TRG — Trigger（触发器）───
    /// 触发条件满足
    TRG001,
    /// 触发器注册
    TRG002,
    /// 触发器移除
    TRG003,
    /// 触发器因频率限制被抑制
    TRG004,

    // ─── SPR — Spell（法术）───
    /// 法术施放完成
    SPR001,
    /// 法术位数量变化
    SPR002,
    /// 专注打断
    SPR003,
    /// 豁免检定结果
    SPR004,

    // ─── RCT — Reaction（反应/援护）───
    /// 反应满足触发条件
    RCT001,
    /// 反应执行完毕
    RCT002,
    /// 单位选择不使用反应
    RCT003,
    /// 机会攻击执行完毕
    RCT004,
    /// 法术反制执行完毕
    RCT005,

    // ─── QST — Quest（任务）───
    /// 任务被接受
    QST001,
    /// 单个目标完成
    QST002,
    /// 任务交付完成
    QST003,
    /// 任务失败
    QST004,
    /// 任务进度变化
    QST005,

    // ─── PRG — Progression（成长养成）───
    /// 角色获得经验
    PRG001,
    /// 角色升级
    PRG002,
    /// 天赋解锁
    PRG003,
    /// 子职选择
    PRG004,
    /// 属性提升完成
    PRG005,
    /// 获得新职业等级
    PRG006,

    // ─── INV — Inventory（背包/物品）───
    /// 物品进入背包
    INV001,
    /// 消耗品使用完成
    INV002,
    /// 装备穿戴/卸下
    INV003,
    /// 物品从背包移除
    INV004,
    /// 战利品生成
    INV005,

    // ─── ECO — Economy（经济/交易）───
    /// 交易完成
    ECO001,
    /// 商店价格变化
    ECO002,
    /// 角色货币变化
    ECO003,

    // ─── CRF — Crafting（制作）───
    /// 配方习得
    CRF001,
    /// 制作开始
    CRF002,
    /// 制作完成
    CRF003,
    /// 制作失败
    CRF004,

    // ─── FAC — Faction（阵营）───
    /// 角色声望变化
    FAC001,
    /// 阵营关系变化
    FAC002,
    /// 声望等级提升
    FAC003,
    /// 关系判定完成
    FAC004,

    // ─── PRY — Party（队伍）───
    /// 新成员加入队伍
    PRY001,
    /// 成员离开队伍
    PRY002,
    /// 战斗中换人
    PRY003,
    /// 羁绊激活
    PRY004,
    /// 羁绊解除
    PRY005,

    // ─── CNR — CampRest（营地休息）───
    /// 短休完成
    CNR001,
    /// 长休开始
    CNR002,
    /// 长休完成
    CNR003,
    /// 长休被中断
    CNR004,
    /// 营地事件触发
    CNR005,

    // ─── NAR — Narrative（叙事）───
    /// 对话开始
    NAR001,
    /// 玩家选择分支
    NAR002,
    /// 故事标记设置
    NAR003,
    /// 过场动画开始
    NAR004,
    /// 过场动画结束
    NAR005,

    // ─── SUM — Summon（召唤）───
    /// 召唤物被创建
    SUM001,
    /// 召唤物消失
    SUM002,
    /// 召唤物接受指令
    SUM003,
    /// 召唤占用变化
    SUM004,

    // ─── CNT — Content（内容基础设施）───
    /// 内容加载完成
    CNT001,
    /// 内容校验失败
    CNT002,
    /// 注册中心注册
    CNT003,
    /// 上下文构建完成
    CNT004,
    /// 上下文生命周期结束
    CNT005,
    /// 溯源链检测到循环
    CNT006,
    /// 上下文构建校验失败
    CNT007,
    /// Spec 授予到实体
    CNT008,
    /// Spec 从实体移除
    CNT009,
    /// Spec 等级变更
    CNT010,
    /// EffectSpec 快照
    CNT011,
    /// Cue 触发条件满足
    CNT012,
    /// Cue 被禁用/跳过
    CNT013,
    /// 条件评估通过
    CNT014,
    /// 条件评估不通过
    CNT015,
    /// 免疫条件生效
    CNT016,
    /// 条件进入订阅状态
    CNT017,
    /// 目标选择完成
    CNT018,
    /// 目标选择被修改
    CNT019,
    /// 无合法目标
    CNT020,
    /// 单个目标通过校验
    CNT021,
    /// 执行计算完成
    CNT022,
    /// 执行计算失败
    CNT023,
    /// 自定义执行注册
    CNT024,
    /// 事件被发布到 EventBus
    CNT025,
    /// 事件成功投递到订阅者
    CNT026,
    /// 事件投递到订阅者失败
    CNT027,
    /// 检测到事件循环触发
    CNT028,

    // ─── SAV — Save（存档基础设施）───
    /// 存档创建
    SAV001,
    /// 存档加载
    SAV002,
    /// 存档删除
    SAV003,

    // ─── RPL — Replay（回放基础设施）───
    /// 回放开始
    RPL001,
    /// 回放帧录制
    RPL002,
    /// 回放校验不一致
    RPL003,
}

impl std::fmt::Display for LogCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl LogCode {
    /// 返回编码字符串，如 `"BAT001"`。
    pub fn code(&self) -> &'static str {
        match self {
            Self::BAT001 => "BAT001",
            Self::BAT002 => "BAT002",
            Self::BAT003 => "BAT003",
            Self::BAT004 => "BAT004",
            Self::BAT005 => "BAT005",
            Self::BAT006 => "BAT006",
            Self::BAT007 => "BAT007",
            Self::BAT008 => "BAT008",
            Self::BAT009 => "BAT009",
            Self::BAT010 => "BAT010",

            Self::TAC001 => "TAC001",
            Self::TAC002 => "TAC002",
            Self::TAC003 => "TAC003",
            Self::TAC004 => "TAC004",
            Self::TAC005 => "TAC005",

            Self::TER001 => "TER001",
            Self::TER002 => "TER002",
            Self::TER003 => "TER003",
            Self::TER004 => "TER004",

            Self::ABL001 => "ABL001",
            Self::ABL002 => "ABL002",
            Self::ABL003 => "ABL003",
            Self::ABL004 => "ABL004",

            Self::EFF001 => "EFF001",
            Self::EFF002 => "EFF002",
            Self::EFF003 => "EFF003",
            Self::EFF004 => "EFF004",
            Self::EFF005 => "EFF005",
            Self::EFF006 => "EFF006",
            Self::EFF007 => "EFF007",
            Self::EFF008 => "EFF008",

            Self::TAG001 => "TAG001",
            Self::TAG002 => "TAG002",
            Self::TAG003 => "TAG003",
            Self::TAG004 => "TAG004",

            Self::MOD001 => "MOD001",
            Self::MOD002 => "MOD002",
            Self::MOD003 => "MOD003",
            Self::MOD004 => "MOD004",

            Self::AGG001 => "AGG001",
            Self::AGG002 => "AGG002",
            Self::AGG003 => "AGG003",
            Self::AGG004 => "AGG004",

            Self::TRG001 => "TRG001",
            Self::TRG002 => "TRG002",
            Self::TRG003 => "TRG003",
            Self::TRG004 => "TRG004",

            Self::SPR001 => "SPR001",
            Self::SPR002 => "SPR002",
            Self::SPR003 => "SPR003",
            Self::SPR004 => "SPR004",

            Self::RCT001 => "RCT001",
            Self::RCT002 => "RCT002",
            Self::RCT003 => "RCT003",
            Self::RCT004 => "RCT004",
            Self::RCT005 => "RCT005",

            Self::QST001 => "QST001",
            Self::QST002 => "QST002",
            Self::QST003 => "QST003",
            Self::QST004 => "QST004",
            Self::QST005 => "QST005",

            Self::PRG001 => "PRG001",
            Self::PRG002 => "PRG002",
            Self::PRG003 => "PRG003",
            Self::PRG004 => "PRG004",
            Self::PRG005 => "PRG005",
            Self::PRG006 => "PRG006",

            Self::INV001 => "INV001",
            Self::INV002 => "INV002",
            Self::INV003 => "INV003",
            Self::INV004 => "INV004",
            Self::INV005 => "INV005",

            Self::ECO001 => "ECO001",
            Self::ECO002 => "ECO002",
            Self::ECO003 => "ECO003",

            Self::CRF001 => "CRF001",
            Self::CRF002 => "CRF002",
            Self::CRF003 => "CRF003",
            Self::CRF004 => "CRF004",

            Self::FAC001 => "FAC001",
            Self::FAC002 => "FAC002",
            Self::FAC003 => "FAC003",
            Self::FAC004 => "FAC004",

            Self::PRY001 => "PRY001",
            Self::PRY002 => "PRY002",
            Self::PRY003 => "PRY003",
            Self::PRY004 => "PRY004",
            Self::PRY005 => "PRY005",

            Self::CNR001 => "CNR001",
            Self::CNR002 => "CNR002",
            Self::CNR003 => "CNR003",
            Self::CNR004 => "CNR004",
            Self::CNR005 => "CNR005",

            Self::NAR001 => "NAR001",
            Self::NAR002 => "NAR002",
            Self::NAR003 => "NAR003",
            Self::NAR004 => "NAR004",
            Self::NAR005 => "NAR005",

            Self::SUM001 => "SUM001",
            Self::SUM002 => "SUM002",
            Self::SUM003 => "SUM003",
            Self::SUM004 => "SUM004",

            Self::CNT001 => "CNT001",
            Self::CNT002 => "CNT002",
            Self::CNT003 => "CNT003",
            Self::CNT004 => "CNT004",
            Self::CNT005 => "CNT005",
            Self::CNT006 => "CNT006",
            Self::CNT007 => "CNT007",
            Self::CNT008 => "CNT008",
            Self::CNT009 => "CNT009",
            Self::CNT010 => "CNT010",
            Self::CNT011 => "CNT011",
            Self::CNT012 => "CNT012",
            Self::CNT013 => "CNT013",
            Self::CNT014 => "CNT014",
            Self::CNT015 => "CNT015",
            Self::CNT016 => "CNT016",
            Self::CNT017 => "CNT017",
            Self::CNT018 => "CNT018",
            Self::CNT019 => "CNT019",
            Self::CNT020 => "CNT020",
            Self::CNT021 => "CNT021",
            Self::CNT022 => "CNT022",
            Self::CNT023 => "CNT023",
            Self::CNT024 => "CNT024",
            Self::CNT025 => "CNT025",
            Self::CNT026 => "CNT026",
            Self::CNT027 => "CNT027",
            Self::CNT028 => "CNT028",

            Self::SAV001 => "SAV001",
            Self::SAV002 => "SAV002",
            Self::SAV003 => "SAV003",

            Self::RPL001 => "RPL001",
            Self::RPL002 => "RPL002",
            Self::RPL003 => "RPL003",
        }
    }

    /// 返回中文描述。
    pub fn description(&self) -> &'static str {
        match self {
            Self::BAT001 => "战斗开始",
            Self::BAT002 => "战斗结束",
            Self::BAT003 => "回合开始",
            Self::BAT004 => "回合结束",
            Self::BAT005 => "单位回合开始",
            Self::BAT006 => "单位回合结束",
            Self::BAT007 => "伤害结算完成",
            Self::BAT008 => "单位死亡",
            Self::BAT009 => "先攻检定完成",
            Self::BAT010 => "胜负条件满足",

            Self::TAC001 => "单位移动完成",
            Self::TAC002 => "夹击判定完成",
            Self::TAC003 => "背刺判定完成",
            Self::TAC004 => "掩体判定完成",
            Self::TAC005 => "单位位置变更",

            Self::TER001 => "单位进入格子",
            Self::TER002 => "格子表面变化",
            Self::TER003 => "陷阱触发",
            Self::TER004 => "地形效果施加",

            Self::ABL001 => "技能成功激活",
            Self::ABL002 => "技能执行完毕",
            Self::ABL003 => "技能被取消/打断",
            Self::ABL004 => "冷却开始",

            Self::EFF001 => "效果成功施加",
            Self::EFF002 => "效果移除",
            Self::EFF003 => "周期效果 Tick",
            Self::EFF004 => "效果因免疫被阻止",
            Self::EFF005 => "执行计算完成",
            Self::EFF006 => "执行计算失败",
            Self::EFF007 => "自定义执行注册",
            Self::EFF008 => "堆叠达到上限触发溢出",

            Self::TAG001 => "标签授予实体",
            Self::TAG002 => "标签从实体移除",
            Self::TAG003 => "标签层级变更",
            Self::TAG004 => "标签查询评估完成",

            Self::MOD001 => "修改器注册到容器",
            Self::MOD002 => "修改器从容器移除",
            Self::MOD003 => "修改器被高优先级抑制",
            Self::MOD004 => "检测到过期修改器",

            Self::AGG001 => "属性聚合计算完成",
            Self::AGG002 => "属性被标记需要重算",
            Self::AGG003 => "快照拍摄完成",
            Self::AGG004 => "检测到聚合闭环",

            Self::TRG001 => "触发条件满足",
            Self::TRG002 => "触发器注册",
            Self::TRG003 => "触发器移除",
            Self::TRG004 => "触发器因频率限制被抑制",

            Self::SPR001 => "法术施放完成",
            Self::SPR002 => "法术位数量变化",
            Self::SPR003 => "专注打断",
            Self::SPR004 => "豁免检定结果",

            Self::RCT001 => "反应满足触发条件",
            Self::RCT002 => "反应执行完毕",
            Self::RCT003 => "单位选择不使用反应",
            Self::RCT004 => "机会攻击执行完毕",
            Self::RCT005 => "法术反制执行完毕",

            Self::QST001 => "任务被接受",
            Self::QST002 => "单个目标完成",
            Self::QST003 => "任务交付完成",
            Self::QST004 => "任务失败",
            Self::QST005 => "任务进度变化",

            Self::PRG001 => "角色获得经验",
            Self::PRG002 => "角色升级",
            Self::PRG003 => "天赋解锁",
            Self::PRG004 => "子职选择",
            Self::PRG005 => "属性提升完成",
            Self::PRG006 => "获得新职业等级",

            Self::INV001 => "物品进入背包",
            Self::INV002 => "消耗品使用完成",
            Self::INV003 => "装备穿戴/卸下",
            Self::INV004 => "物品从背包移除",
            Self::INV005 => "战利品生成",

            Self::ECO001 => "交易完成",
            Self::ECO002 => "商店价格变化",
            Self::ECO003 => "角色货币变化",

            Self::CRF001 => "配方习得",
            Self::CRF002 => "制作开始",
            Self::CRF003 => "制作完成",
            Self::CRF004 => "制作失败",

            Self::FAC001 => "角色声望变化",
            Self::FAC002 => "阵营关系变化",
            Self::FAC003 => "声望等级提升",
            Self::FAC004 => "关系判定完成",

            Self::PRY001 => "新成员加入队伍",
            Self::PRY002 => "成员离开队伍",
            Self::PRY003 => "战斗中换人",
            Self::PRY004 => "羁绊激活",
            Self::PRY005 => "羁绊解除",

            Self::CNR001 => "短休完成",
            Self::CNR002 => "长休开始",
            Self::CNR003 => "长休完成",
            Self::CNR004 => "长休被中断",
            Self::CNR005 => "营地事件触发",

            Self::NAR001 => "对话开始",
            Self::NAR002 => "玩家选择分支",
            Self::NAR003 => "故事标记设置",
            Self::NAR004 => "过场动画开始",
            Self::NAR005 => "过场动画结束",

            Self::SUM001 => "召唤物被创建",
            Self::SUM002 => "召唤物消失",
            Self::SUM003 => "召唤物接受指令",
            Self::SUM004 => "召唤占用变化",

            Self::CNT001 => "内容加载完成",
            Self::CNT002 => "内容校验失败",
            Self::CNT003 => "注册中心注册",
            Self::CNT004 => "上下文构建完成",
            Self::CNT005 => "上下文生命周期结束",
            Self::CNT006 => "溯源链检测到循环",
            Self::CNT007 => "上下文构建校验失败",
            Self::CNT008 => "Spec 授予到实体",
            Self::CNT009 => "Spec 从实体移除",
            Self::CNT010 => "Spec 等级变更",
            Self::CNT011 => "EffectSpec 快照",
            Self::CNT012 => "Cue 触发条件满足",
            Self::CNT013 => "Cue 被禁用/跳过",
            Self::CNT014 => "条件评估通过",
            Self::CNT015 => "条件评估不通过",
            Self::CNT016 => "免疫条件生效",
            Self::CNT017 => "条件进入订阅状态",
            Self::CNT018 => "目标选择完成",
            Self::CNT019 => "目标选择被修改",
            Self::CNT020 => "无合法目标",
            Self::CNT021 => "单个目标通过校验",
            Self::CNT022 => "执行计算完成",
            Self::CNT023 => "执行计算失败",
            Self::CNT024 => "自定义执行注册",
            Self::CNT025 => "事件被发布到 EventBus",
            Self::CNT026 => "事件成功投递到订阅者",
            Self::CNT027 => "事件投递到订阅者失败",
            Self::CNT028 => "检测到事件循环触发",

            Self::SAV001 => "存档创建",
            Self::SAV002 => "存档加载",
            Self::SAV003 => "存档删除",

            Self::RPL001 => "回放开始",
            Self::RPL002 => "回放帧录制",
            Self::RPL003 => "回放校验不一致",
        }
    }
}
