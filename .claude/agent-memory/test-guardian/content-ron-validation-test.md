---
name: content-ron-validation-test
description: Created comprehensive content_ron_validation_test.rs that validates all 95 RON config files deserialize and pass DefinitionType::validate()
metadata:
  type: project
---

Created `src/content/tests/unit/content_ron_validation_test.rs` — 15 test functions across 13 active content buckets. The test discovered and fixed 4 real RON data bugs (wrong field names and incompatible tuple struct syntax in quest/targeting files).

**Why:** Previously there was no automated verification that all RON config files in `assets/config/` could actually deserialize and pass validation. Individual `DefinitionType::validate()` tests existed in `def_impls_test.rs` but only tested in-memory samples, never actual RON files.

**How to apply:** Run `cargo nextest run content_ron_validation` to verify all RON files. When adding a new RON file to any bucket, the relevant test will automatically pick it up and verify it. When adding a new content bucket, add a new test function following the existing patterns.
