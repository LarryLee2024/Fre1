---
name: quest-recipe-shop-strict-id-validation
description: QuestDef/RecipeDef/ShopDef use strict validate_id_format requiring prefix + digits (qst_/rcp_/shp_)
metadata:
  type: reference
---

These three Def types call `validate_id_format()`, requiring IDs like `qst_000001`, `rcp_000001`, `shp_000001`. Other Def types (BondDef, CampEventDef, EnchantmentDef, SummonTemplateDef) only check non-empty.
