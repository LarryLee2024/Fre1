---
name: bond-def-ron-convention
description: BondDef RON uses bnd:name format (colon), no strict prefix validation, HashMap level_effects with Vec<String> modifier references
metadata:
  type: reference
---

BondDef validation only checks non-empty -- no `validate_id_format()` call. 
RON files use `bnd:name` format (e.g., `"bnd:trust"`), matching the Display output of the `define_string_id!` macro for `BondDefId`.

Fields: id (BondDefId), name_key (LocalizationKey), desc_key (LocalizationKey), required_members (Vec<BondRequirement> with required_tags/match_mode), level_effects (HashMap<u32, Vec<String>>), max_level (u32).
