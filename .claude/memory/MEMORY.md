# Memory Index

> Pointers only — the actual content lives in the linked files, never here.
> Soft cap 150 lines / 20 KB (warn). Hard cap 200 lines / 25 KB (compact first).
> One line per memory: `- [<title>](<slug>.md) — <one-line hook>`

## user
<!-- who the user is: identity, role, durable preferences -->

- [user-role](user-role.md) — 独立游戏开发者，Fre SRPG

## feedback
<!-- how the agent should behave (procedural). Each file carries Why: / How to apply: -->

- [feedback-codegraph-first](feedback-codegraph-first.md) — 查代码优先 CodeGraph
- [feedback-agent-delegation](feedback-agent-delegation.md) — 复杂多步任务派发 agent 并行处理
- [feedback-terse-response](feedback-terse-response.md) — 回复简洁直接

## project
<!-- current work state not in code/git. Absolute dates. Why: / How to apply: -->

- [project-architecture](project-architecture.md) — DDD 三层+横切四层，架构不变式
- [project-constraints](project-constraints.md) — 技术约束和禁止事项
- [project-bevy-translations](project-bevy-translations.md) — Bevy 版本翻译进度与规范
- [battle-hud-data-flow](battle-hud-data-flow-2026-06-22.md) — BattleHudData Resource + UiBinding + Dirty消费链
- [skill-panel-widget](skill-panel-widget-implementation.md) — SkillPanel Organism: spawn_skill_slot wrapper, skill_id override, Z7 integration, SkillPanelPlugin
- [character-status-panel-widget](character-status-panel-widget.md) — CharacterStatusPanel Organism: factory, components, plugin, UI tree structure
- [project-ui-review-2026-06-22](project-ui-review-2026-06-22.md) — UI 设计-代码偏移评审及 11 agent 批量修复

## decision
<!-- key architectural decisions -->

- [decision-capability-not-service](decision-capability-not-service.md) — 能力系统采用 Capability 非 Service

## reference
<!-- pointers to external resources: URLs, dashboards, tickets, log paths -->

- [reference-github](reference-github.md) — GitHub Issue/CI
- [reference-bevy-source](reference-bevy-source.md) — Bevy 源码 crates/ + examples/ 本地路径
