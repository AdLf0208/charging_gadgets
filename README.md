# Charging Gadgets — Horizon port (slice 1: the charging station)

A native Horizon port of **Charging Gadgets 1.14.1**
(`Direwolf20-MC/ChargingGadgets`, **MIT**): a fuelled station that burns furnace
fuel into a 1,000,000 FE buffer and fills the FE buffer of the item in its
charge slot at up to 2,500 FE a tick.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
MIT is licence rank 0 (permissive); approval is still required.

Provenance, hashes and the upstream commit are in `LICENSE-NOTICE.md` and
`ports/_shared/carry-work/charginggadgets/SOURCE.md`; the behaviour reading is
in `FINDINGS.md`; the ladder, deviations and gaps are in `notes.md`; the content
decisions are in `CONTENT.md`.

## Layout (the flat SDK package, mod API 0.2)

```
horizon-mod.toml                   the only manifest and the only identity
Cargo.toml                         one crate; horizon-mod-api from .horizon/sdk/
src/lib.rs                         the one #[mod_api::horizon_mod] impl Hooks
src/runtime.rs                     the pure rules (burn, charge, data slots), 16 tests
src/station.rs                     glue: the block entity Part, the declarations,
                                   the item's charge, the screen, the commands
assets/charginggadgets/**          the carried models, textures, lang, item definition
data/**                            the carried block, menu, recipe, loot, advancement, tags
LICENSE, LICENSE-NOTICE.md         inside the package
charginggadgets.hmod               the delivered package
```

One `.hmod` carries the content **and** the code: Horizon retired `.hnmod` on
2026-11-18, so there is no embedded-content helper any more and no second
package to install.

## Commands (harness controls)

```
/charginggadgets info <x> <y> <z>     the buffer, the burn counter, both slots
/charginggadgets energy <x> <y> <z> <amount>
/charginggadgets item <x> <y> <z> <amount>
/charginggadgets held <amount>
```

## Verify

```sh
cd ports/charginggadgets
cargo test                                   # runtime + glue tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
rm -f Cargo.lock && cargo horizon-mod build   # -> target/horizon-mod/charginggadgets.hmod
cargo horizon-mod check charginggadgets.hmod
cargo horizon-mod fits charginggadgets.hmod --build nightly
```

The in-game scenario needs a **launcher build carrying this package**
(`horizon-launcher native-build . --instance <i> --trust-native-code`) and runs
as `GAME=<that client> tools/e2e.sh`; see `notes.md`.
