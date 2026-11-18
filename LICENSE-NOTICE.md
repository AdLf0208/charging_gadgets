# Licence provenance

- Mod: `charginggadgets` (Charging Gadgets) by Direwolf20 and ErrorMikey —
  <https://github.com/Direwolf20-MC/ChargingGadgets>
- Licence: **MIT**, read from the jar's own files for
  `charginggadgets-1.14.1.jar` (`META-INF/neoforge.mods.toml` `license = "MIT"`,
  MIT text in the repository's `LICENSE`). Licence rank 0 (permissive).
- Upstream commit analysed: `5f6d7b63bbc1397d9b7a528b31c431f24da4ec11`
  (2024-08-10, "Update for 1.21.1", `mod_version=1.14.1`). The reading is in the
  port's `FINDINGS.md`; the jar stays as licence evidence in
  `ports/_shared/licence-cache/charginggadgets-1.14.1.jar`.
- The carried assets (`assets/charginggadgets/**`, including the six textures and
  the models), the lang file, the loot table, the recipe, the advancement and the
  tags are upstream's own MIT files, taken from the jar and adapted to 26.3's
  data shapes (`CONTENT.md`).
- This port's Rust source is a clean-room rewrite from the mod's behaviour; no
  Java was copied or translated and no jar was decompiled.
- Local-only until the maintainer approves publishing (ARCHITECTURE.md §8);
  publishable under MIT with this source, this licence and this notice.
