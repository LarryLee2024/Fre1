---
name: content-buckets-id-formats
description: Each content bucket's ID format convention in RON files - colon vs underscore vs underscore+digits
metadata:
  type: reference
---

- **bonds/** — `bnd:name` (colon format, no strict validation)
- **camp_events/** — `cmp:name` (colon format, no strict validation)  
- **spell_config/** — single file only
- **enchantments/** — `enc_name` (underscore-text, no strict validation)
- **summon_templates/** — `smt_name` (underscore-text, no strict validation)
- **quests/** — `qst_000001` (underscore+digits, strict validate_id_format)
- **recipes/** — `rcp_000001` (underscore+digits, strict validate_id_format)
- **shops/** — `shp_000001` (underscore+digits, strict validate_id_format)
- **spells/** — `spl_000001` (underscore+digits, strict validate_id_format)
- **effects/** — `eff_000001` (underscore+digits, strict validate_id_format)
- **abilities/** — `abl_000001` (underscore+digits, strict validate_id_format)
- **cues/** — `cue_name` (underscore-text pattern, starts-with check only)
- **targeting/** — no id field in the RON itself (single-file bucket)
- **rules/** — `rule_name` (underscore-text, validate_id_format with rule_ prefix)
- **tags/** — tags use `tag:name` / `DamageType.Elemental.Fire` path convention
- **attributes/** — Vec format, `attr:name` IDs
