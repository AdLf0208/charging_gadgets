# Charging Gadgets — a Horizon port

**Charging Gadgets** is a Minecraft mod. For all your RF charging needs. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [Direwolf20-MC/ChargingGadgets](https://github.com/Direwolf20-MC/ChargingGadgets) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Charging Gadgets ([source](https://github.com/Direwolf20-MC/ChargingGadgets)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `charginggadgets.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **The block.** `charginggadgets:charging_station`, a 176 x 166-panel machine on
- **The buffer.** 1,000,000 FE (`Config.GENERAL.chargerMaxPower`), declared
- **The burn.** One step a server tick while the buffer has room for a whole
- **The charge.** Each tick after the burn, `min(stored, 2500)` FE goes into the
- **The item's own charge.** The block item declares the same buffer
- **The menu.** `charginggadgets:charging_station_menu` (2 slots, 4 data slots:
- **Harness controls.** `/charginggadgets info <pos>`, `/charginggadgets energy

## Not yet ported

- **No in-game run at 0.2.0.** `horizon-launcher` is not installed here, so a game carrying
- **The old `content_root` control leg is gone** (traps 11): the new control is the
- **Only this mod's item is chargeable by the tick.** The tick reads the stack's own
- **Breaking the station loses its FE.** Upstream's `getDrops` puts the stored
- **The block item's charge is a capability, not a private component.** It
- **No GUI texture and no bars.** The carried `textures/gui/charging_station.png`
- **No per-slot insert rules.** Upstream's `RestrictedSlot.mayPlace` and
- **A stack of more than one item in the charge slot is left alone** (the

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `charginggadgets.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Charging Gadgets was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/Direwolf20-MC/ChargingGadgets

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run charginggadgets` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/charginggadgets/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
