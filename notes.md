# Charging Gadgets — port notes (slice 1: the charging station)

Ported from **Charging Gadgets 1.14.1** (`charginggadgets-1.14.1.jar`, SHA-1
`3208ea2ca10d786895d03c2cd3a124d14393b357`, SHA-256
`411bae6c95c1c4ab19e0431785341dabe39b6eaf5f132abd00514264243ba2ac`) for
Minecraft Java 1.21.1 / NeoForge. **MIT → licence rank 0**; local-only until
publishing is approved (ARCHITECTURE.md §8).

**Format: the flat SDK package (2026-11-18), version `0.2.0`.** The retired
`.hnmod` bundle (`0.1.0`) was migrated to the flat `.hmod` — see
[`## Migration to the SDK format`](#migration-to-the-sdk-format) for the file
map, the helper→shape table, the measured `check`/`fits` output and every trap.
`src/runtime.rs` is **byte-identical** to the pre-migration
`mixins/charginggadgets/runtime.rs` (sha256
`234fdaac9de2e55482b592f539dddf4cbb3e41fe7620e608bc1b378457ca632a`), including
the file inside the built package. The retired tree is
`ports/_shared/evidence/charginggadgets/retired-hnmod-0.1.0/`.
Pins: nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78.

Status: **ported-partial** — slice 1 (the fuelled FE buffer, the charge slot,
the two-slot menu and its readout, the block item's charge and the content
carry) is implemented; its behaviour was verified in game at `0.1.0`, and the
migration keeps the rules byte-identical (see the gaps for the one run this
migration could not repeat). The gap list below is the rest of the mod.

**Why this mod now:** it is the rank-0 `content`/`machines` core mod
`python3 ports/tools/precheck-mod.py charginggadgets` ranks first (jar licence
MIT, rank 0, no unported dependency, no worldgen). It exited **3** at this
touch for the one reason that is correct here — *already ported-partial — not a
new dispatch* — and the job is its migration, briefed by
`ports/_shared/queue/brief-charginggadgets-sdk.md`.

## Class + tier and the approach

- **Class:** `content`; **tier:** `machines` (`ports/ledger.csv`,
  `ports/_shared/source-map/atm10-sources.csv`).
- **Approach** (`horizon/docs/modpacks.md`, "What packs need and what runs
  today"): a **machines** shape — a block entity with a server tick, a declared
  energy capability, a container and a menu/screen — plus the carried block
  assets/lang/recipe/loot. Not a data-only block, and nothing here is `deep`.
- **Carry-over vs agent port:** the blockstate, both models, six textures, the
  lang file, the recipe, the loot table, the advancement and the pickaxe tag are
  carried from the jar (`tools/assemble_content.py`, decisions in
  [`CONTENT.md`](CONTENT.md)); the behaviour, the block entity, the container,
  the menu, the screen and the commands are the agent port.

## What the slice does

- **The block.** `charginggadgets:charging_station`, a 176 x 166-panel machine on
  the **`minecraft:stonecutter`** template (vanilla's only 4-value-`facing`
  template whose class opens a menu on use), hardness 2.0 / blast resistance 2.0
  / stone sounds, a full cube. Placed with `facing` opposite the player, as
  upstream's `getStateForPlacement`. The carried model is
  `block/orientable_with_bottom` with the jar's five textures.
- **The buffer.** 1,000,000 FE (`Config.GENERAL.chargerMaxPower`), declared
  through `block_entity_energy` (API 142, `Capabilities.Energy.BLOCK`): any side
  may fill it up to 1,000,000 a call, **nothing** may drain it — upstream's
  `extractEnergy` returns 0. The amount is the int `energy` in the block
  entity's compound, as upstream saves it.
- **The burn.** One step a server tick while the buffer has room for a whole
  step, 625 FE a step; `counter = floor(burnTime / 50)`, so **one coal = 32
  steps = 20,000 FE**. The burn time is the engine's own `minecraft:cooking_fuel`
  value (`Providers::burn_time`), so modded fuels work; the crafting remainder
  is honoured (a lava bucket leaves a bucket). The next fuel loads in the tick
  the counter reaches zero, as upstream's `burn()` calls `initBurn()`.
- **The charge.** Each tick after the burn, `min(stored, 2500)` FE goes into the
  item in slot 1 through `Capabilities.Energy.ITEM` (API 307:
  `native_item_energy` / `native_item_energy_insert`), and only what the item
  accepted is debited. A non-chargeable item, a full item and an empty buffer
  take nothing.
- **The item's own charge.** The block item declares the same buffer
  (`item_energy_buffer`, API 307), which is the pin's form of upstream's
  `charginggadgets:energy` component: the tooltip shows
  `screen.charginggadgets.energy` when charged, and `place_block` (API 14) fills
  the block the item becomes (upstream's `updateCustomBlockEntityTag`).
- **The menu.** `charginggadgets:charging_station_menu` (2 slots, 4 data slots:
  `energy/32`, `capacity/32`, `counter`, `maxburn`, upstream's own
  `ContainerData`), slots at upstream's (65, 43) and (119, 43) pixels in the
  176 x 166 panel; the screen draws the FE and the burn steps as numeric
  readouts.
- **Harness controls.** `/charginggadgets info <pos>`, `/charginggadgets energy
  <pos> <amount>`, `/charginggadgets item <pos> <amount>`, `/charginggadgets
  held <amount>` — the first three read/set a station, the last charges the held
  item. Upstream has no commands of its own (its config/GUI are the only
  controls); precedent `/powah info|set`, `/mininggadgets energy`.

## Hooks (0.1.0, kept as history)

*The 0.1.0 wiring. The 0.2.0 mapping is the helper→shape table in
[`## Migration to the SDK format`](#migration-to-the-sdk-format); these
injections and helpers no longer exist.*

| Hook (API) | Helper | What it does |
|---|---|---|
| `block_entity` (15) | `block_entity.rs` | the station's block entity |
| `block_entity_ticks` (15) | `block_entity_ticks.rs` | ticks on the server |
| `block_entity_container` (16/17) | `block_entity_container.rs` | two slots, every face in and out, the menu type |
| `block_entity_energy` (142) | `block_entity_energy.rs` | the 1,000,000 FE insert-only buffer |
| `block_entity_tick` (17) | `block_entity_tick.rs` | burn, then charge, then the data slots |
| `screen` (19) | `screen.rs` | the menu's two slots and the panel size |
| `screen_draw` (19) | `screen_draw.rs` | the FE and burn readouts |
| `item_energy_buffer` (307) | `item_energy_buffer.rs` | the block item's FE buffer |
| `place_block` (14) | `place_block.rs` | a charged item fills the block it becomes |
| `tooltip` (12) | `tooltip.rs` | the item's charge line |
| `register_commands` (8) | `register_commands.rs` | the four harness commands |
| `perform_command` (8) | `perform_command.rs` | their bodies |
| `content_root` (raw target) | `embedded_content.rs` | writes the bundle's own content out and loads it |

13 injections (12 named hook points + one raw `content_root` target), 2 modules,
**0 unstamped**.

## Pin facts that shaped this slice

1. **Item-level FE is real at this pin.** `item_energy_buffer` (API 307) is
   stamped and the engine keeps the amount in the stack's `custom_data`
   (`game/native_item_energy.rs`), so the station charges a genuine
   `Capabilities.Energy.ITEM` buffer — not a durability stand-in. ARCHITECTURE
   §3's remaining energy gap is specifically *block-to-block* transfer, which
   this slice does not need.
2. **A block's own tick cannot read a neighbour's buffer** (`native_energy_at`
   refuses the ticking entity), but the station only reads *its own* compound and
   the *item* it holds, both of which are in hand inside the tick.
3. **A fuel item is consumed when the counter loads, not while it burns**, so an
   empty fuel slot after `ctl container` means the coal is already burning —
   which is why the e2e keeps a fuel stack in the slot while it inserts the
   charge item.
4. **No capability-aware slot rule**: `menu_slot_rules` (API 130) is a free
   function in the library crate and cannot name the gameplay-side bundle, so
   the tick refuses what it cannot burn or charge instead of the slot refusing
   the insert (gap 4 below).

## Content (`assets/` + `data/` in the package)

The carried trees moved up from the retired `content/` unchanged: `assets/` and
`data/` are the package root's, beside `src/`. They were assembled from the jar
by `tools/assemble_content.py` (now retired into
`ports/_shared/evidence/charginggadgets/retired-hnmod-0.1.0/tools/`);
[`CONTENT.md`](CONTENT.md) lists every carried and every written file and why.
24 entries: the blockstate (4 facings), the block and item models, 26.3's item
definition, six textures (five block + the unused GUI background), `en_us.json`,
the loot table and recipe (upgraded to 26.3), the advancement, the pickaxe tag,
the block/menu/creative-tab declarations and five `c:` item tags.

## Embedded content (AE2 pattern) — historical, removed by the SDK format

*The `.hnmod` container is retired (2026-11-18): one flat `.hmod` carries the
content and the code, so this whole workaround is gone — there is no
`content_root` injection, no cache file and no `EMBEDDED`/`CONTROL` leg. What
follows is what 0.1.0 did and the in-game evidence it left; the control leg in
the new `tools/e2e.sh` is an unmodified game (`PLAIN_GAME`).*
The registry stores **one file per project** (playbook §1.0), so a port with both
halves could not be listed faithfully: an `.hnmod` is code-only and an `.hmod`
is content. This bundle now carries its content, exactly as the merged AE2 entry
does:

- **`mixins/charginggadgets/data/charginggadgets-content.hmod`** is the built
  `content/charginggadgets.hmod`, byte for byte (20,248 B,
  `f926468a…6117`). `tools/package.py` packs `data/**` as
  `mixins/<id>/data/…` (top-level `.rs`/`.toml` plus everything under
  `data/`), with the same fixed zip timestamps, so a rebuild reproduces the
  SHA-256 (two consecutive runs: identical). It refuses a
  `data/<id>-content.hmod` that is not byte-identical to `content/<id>.hmod`,
  so a rebuilt content mod can never ship stale inside the bundle.
- **`mixins/charginggadgets/embedded_content.rs`** with the `embedded-content`
  `[[injection]]` (`target = "horizon_client::game::mods::content_root"`,
  `kind = "HEAD"`, `cancellable = true`, bindings `root`/`enabled`/`dev`)
  compiles the file in with `native_data!` (API 41), writes it to
  `.horizon-cache/charginggadgets-embedded-content.hmod` when the copy on disk
  differs, appends it to the dev-mod list and returns its `content_root`. The
  helper calls `content_root` once more and stops at its own
  `dev.contains(&file)` guard, so the second call runs the vanilla body — the
  same one-level recursion AE2 uses. An unwritable cache logs a warning and
  `Continue`s (no content) instead of failing the game.

Verified in game with **only the bundle** (`EMBEDDED=1`, no `--dev-mod`): the
whole scenario A–H passes, and the file the helper wrote is byte-identical to
`content/charginggadgets.hmod`. The control (`CONTROL=1`) launches the same
binary with `.horizon-cache` replaced by a plain file, so the helper refuses its
content: the station item never arrives and
`setblock … charginggadgets:charging_station` leaves `minecraft:air` — the ids
are unknown without the embedded file, so the PASS is not vacuous.

This makes the `.hnmod` the single listable artifact of this port; it does not
by itself publish it (licence approval is still required, ARCHITECTURE.md §8).

### In-game transcript (quoted from `e2e-embedded.log`)

```
== control: the same game with no embedded content file ==
  without the embedded file: no station item, the block stayed minecraft:air
  launch: no --dev-mod; the bundle carries its own content
  embedded content: 20248 B at /tmp/opencode/charginggadgets-home.bClTko/.horizon-cache/charginggadgets-embedded-content.hmod, identical to content/charginggadgets.hmod
== E: one coal is 32 burn steps, exactly 20,000 FE ==
  one coal: 20000 FE, counter 0/0
== F: 2,500 FE a tick, and the station loses exactly what the item gains ==
  76 ticks: item 35000 -> 225000 (+190000), station 965000 -> 775000 (-190000)
PASS: … one coal is exactly 20,000 FE, … the charge slot takes exactly 2,500 FE a tick with no FE created or lost …
EXIT:0
```

The same scenario with `SELFTEST=1` ends `FAIL` + `EXIT:1`
(`e2e-embedded-selftest.log`).

## Migration to the SDK format (version 0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70, PR #17992) retired `.hnmod`; the registry
closed every `.hnmod` submission (this port's was **#871**) with one reason. The unit is now
**one flat `.hmod`**: content, native code, or both. Pins for this migration: nightly
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78, SDK slice cached at
`ports/_shared/sdk/0b14f8a/`. The recipe followed is `ports/imfast/notes.md` (first migration)
with `ports/mekanismgenerators/notes.md` for the content+code and Part/declaration shapes.

### The line that makes this a migration and not a rewrite

`src/runtime.rs` is `mixins/charginggadgets/runtime.rs` **byte-identical** (SHA-256
`234fdaac9de2e55482b592f539dddf4cbb3e41fe7620e608bc1b378457ca632a`; `cmp` exit 0, and the
packaged entry is again the same bytes). Its **16 tests were green before any glue was
written** (`sdk-runtime-first.log`: `running 16 tests … 16 passed; 0 failed`, with `src/lib.rs`
holding only `pub mod runtime;`), and they still pass with the glue (24 tests). Evidence:
`ports/_shared/evidence/charginggadgets/runtime-carryover.txt`.

### File map

| 0.2.0 path | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest and the only identity: `id`, `version = "0.2.0"`, `api = "0.2"`, `side`, `capabilities`, `license`/`homepage`/`issues`, `[native]` game/`horizon`/`mixins` |
| `Cargo.toml` | one crate, edition 2024; its only dependency is `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`: `register`, `register_commands`, `run_mod_command` |
| `src/runtime.rs` | the pure logic (burn, charge, menu data slots), byte-identical |
| `src/station.rs` | the glue: `StationPart` (a `BlockEntityPart`), `StationItemPart` (an `ItemPart`), `StationEnergy` (the item's `EnergyHandler`), `container()`, `battery()`, `item_buffer()`, `menu()`, `read`/`write`, `info_line`, `set_energy`/`set_item_energy`/`set_held_energy`, and the tests of the declarations |
| `assets/charginggadgets/**`, `data/**` | the carried content, moved up from the retired `content/` **unchanged** |
| `LICENSE`, `LICENSE-NOTICE.md`, `README.md` | inside the package (LICENSE-NOTICE is new; it holds the provenance the old packager injected) |
| `tools/e2e.sh`, `tools/parse_info.py` | the scenario, rewritten for a launcher-built game; the line parser unchanged |
| `CONTENT.md`, `FINDINGS.md`, `notes.md` | the record (not packed) |

Moved to `ports/_shared/evidence/charginggadgets/retired-hnmod-0.1.0/`: `mixins/`
(`horizon-mixin.toml`, `runtime.rs`, `support.rs` and the twelve helpers), the
`charginggadgets.hnmod` (35,305 B), `compatibility.json`, `rust-toolchain.toml`, the 0.1.0
`Cargo.toml`/`Cargo.lock`/`AGENTS.md`/`PLAN.md`, the built `content/charginggadgets.hmod`
(20,248 B) and `content/horizon-mod.toml`, and the nine old tools (`horizon.py`,
`package.py`, `package.sh`, `assemble_content.py`, `notice.py`, `test_stage.py`,
`test_template.py`, `hmod-load.sh`, `tools/audit/`). Nothing was deleted: the old tree is the
evidence.

### Manifest mapping

| 0.1.0 | 0.2.0 |
|---|---|
| `horizon-mixin.toml` `[mod] id = "charginggadgets"` | `id = "charginggadgets"` (also the namespace) |
| `[mod] version = "0.1.0"` | `version = "0.2.0"` — a resubmission is a **new version** |
| `[mod] priority = 100` | gone; ordering is `#[listener(priority = …)]` on a hook |
| `[compatibility] horizon_revision = "1d63e0a5…"` | `[native] horizon = "0b14f8ac…"` (40 hex, exactly) |
| mixins `5a64af93…` (bundle) | `[native] mixins = "e59da9094f0c…"` — **it moved** |
| game 26.3 (content manifest) | `[native] game = "26.3"` |
| `[[module]] runtime` + `state` | **no plan section**: `src/runtime.rs` is a module, `src/station.rs` its glue |
| 12 `[[injection]]` + 1 raw `content_root` target | one `Hooks::register` of declarations and Parts (table below) |
| `horizon-native.json`, `compatibility.json` | gone |
| `content/horizon-mod.toml` | the root manifest (same `api`/`side`/`capabilities`, plus `[native]`) |
| — | `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]` (the content declares a block/item/menu), `license`, `homepage`, `issues` |

### Helper → shape table

| 0.1.0 injection (API) | 0.2.0 shape | kept / gap |
|---|---|---|
| `block_entity` (15) | `BlockEntityPart::block_entity`, on the type built by `block_entity_type("charging_station").blocks(&["charging_station"]).build()` | **kept** — returns `Some(Compound::new())` |
| `block_entity_ticks` (15) | `BlockEntityPart::block_entity_ticks` sets `event.ticks` | **kept** |
| `block_entity_container` (16/17) | `Registrar::block_container(&block, ModContainer)`, still naming the content menu `charginggadgets:charging_station_menu` | **kept** — the same two slots and face lists |
| `block_entity_energy` (142) | `Registrar::block_energy(&block, ModEnergyBuffer)` | **kept** — 1,000,000 FE, `max_insert` 1,000,000, `max_extract` 0, every face in, none out; the amount is still the int `energy` in the saved compound |
| `block_entity_tick` (17) | `BlockEntityPart::block_entity_tick` | **kept**; `mark_modified` became the `Changed`/`Unchanged` reply |
| `screen` (19) | `BlockEntityPart::screen` returns `Some(Screen)` | **kept** — 176 x 166, slots at (65, 43)/(119, 43), inventory at (8, 84) |
| `screen_draw` (19) | `BlockEntityPart::screen_draw`, drawing on the event's `DrawContext` | **kept** — the same two numeric readouts |
| `item_energy_buffer` (307) | `Registrar::item_capability(&ITEM_ENERGY, id, \|_, place\| …)` with `StationEnergy` | **kept, different mechanism**: a provider, not a declaration |
| `place_block` (14) | `ItemPart::on_block_placed_by` | **kept** — a charged item fills the block it becomes |
| `tooltip` (12) | `ItemPart::tooltip` | **kept** — `screen.charginggadgets.energy` when charged |
| `register_commands` (8) | `Hooks::register_commands`, pushing typed `Command`s (`<pos:block_pos>`, `<amount:integer>`) | **kept** |
| `perform_command` (8) | `Hooks::run_mod_command`, reading the typed `CommandInvocation` | **kept, simpler**: no word parsing, positions resolved by the dispatcher |
| `content_root` (raw target) | — | **gone by design, not a gap**: one `.hmod` carries `assets/`, `data/` and `src/`, so the helper that wrote an embedded content file out has nothing left to work around |

Host calls, 0.1.0 → 0.2.0:

| 0.1.0 (`Game`) | 0.2.0 |
|---|---|
| `game.res.providers().burn_time(items, stack, false)` | `host.item_burn_time(stack.item())` |
| `items().get(&stack.item).remainder` | `host.item_crafting_remainder(&stack)` (then `ItemStack::shrink(1)`) |
| `game.native_item_energy(stack)` / `native_item_energy_insert` | the item's `minecraft:custom_data.horizon_energy` int, read and written by `stack_charge`/`charged` in the tick and by `StationEnergy` through `ITEM_ENERGY` at a place |
| `game.native_with_mod_block_entity(pos, f)` | `host.block_entity(level, pos)` + `BlockEntityRef::{data, item_slot, merge_data}` |
| `game.world.block_entities.get(&key)` | `host.block_entity(host.level(host.dimension())?, pos)` |
| `game.world.game_time` | `LevelRef::game_time(host)` |
| `game.world.modified.insert(chunk)` | nothing: the tick reply dirties the chunk, `Changed` only when something moved |
| `game.native_text(batch, …)` | `event.draw.text(…)` |
| `game.native_feedback(Text)` | `host.commands().feedback(Text)` |
| `game.held_slot()` / `stack_in_mut` | `ItemPlace::PlayerHand { player, hand: Hand::Main }` from the command's `CommandSource::Player` |

The twelve stamped hooks and the one target injection are therefore **12 mapped, 0 recorded as
gaps**; the thirteenth (the embedded-content target) has no successor because the format
removed the reason it existed.

### Build and check (the SDK ladder, measured)

`build-slot.sh` runs the command **from the campaign root** while an SDK package has no staged
tree, so every step passes `bash -c 'cd ports/charginggadgets && …'` and the runner is used for
the slot accounting only (the task's own command shape):

```sh
cd /home/adlf/Desktop/all_the_mods_10_rust_rewrite
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-cgd --port ports/charginggadgets --step test \
    -- bash -c 'cd ports/charginggadgets && CARGO_INCREMENTAL=0 cargo test'
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-cgd --port ports/charginggadgets --step lint \
    -- bash -c 'cd ports/charginggadgets && cargo clippy --all-targets -- -D warnings'
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-cgd --port ports/charginggadgets --step fmt \
    -- bash -c 'cd ports/charginggadgets && cargo fmt --check'
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-cgd --port ports/charginggadgets --step build \
    -- bash -c 'cd ports/charginggadgets && rm -f Cargo.lock && cargo horizon-mod build'
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-cgd --port ports/charginggadgets --step check \
    -- bash -c 'cd ports/charginggadgets && cargo horizon-mod check charginggadgets.hmod'
```

| Check | Result | Evidence |
|---|---|---|
| runtime carried over | `cmp` exit 0, SHA-256 `234fdaac…632a`, packaged entry identical | `runtime-carryover.txt` |
| runtime tests before glue | **16 passed**, 0 failed (`src/lib.rs` = `pub mod runtime;` only) | `sdk-runtime-first.log` |
| the glue's tests can fail | 8 new tests, every body `todo!()` → **16 passed, 8 failed**, each with `not yet implemented` | `sdk-glue-red.log` |
| all tests | **24 passed**, 0 failed | `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` exit 0, **0 warnings** | `sdk-clippy.log` |
| formatting | `cargo fmt --check` clean (no `--all`: it would walk the fetched SDK) | `sdk-fmt.log` |
| package | `cargo horizon-mod build` → `target/horizon-mod/charginggadgets.hmod` | `sdk-build.log` |
| determinism | two consecutive builds **byte-identical**, same SHA-256 | this file, §package |
| registry check | `charginggadgets 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)`, **48,087 B**, SHA-256 `87c46de5…ea12`, unsigned, **EXIT 0** | `sdk-check.log`, `sdk-check-json.log` |
| what it uses | 369 surface entries (172 type, 86 handle, 40 method, 35 hook, 27 trait, 5 part, 2 capability, 2 macro) | `sdk-uses-summary.log`, `sdk-check-uses.log` |
| fits the pinned build | `0b14f8a made for`, **EXIT 0** | `sdk-fits.log` |
| carried content | `assets/` + `data/` are the retired `content/` trees, byte for byte | §File map |
| fresh-clone repeat | `.horizon/`, `target/` and `Cargo.lock` removed, then `cargo horizon-mod build` | SDK fetched by itself; **same size, same SHA-256** (`sdk-fresh-clone.log`) |
| package | `charginggadgets.hmod` **48,087 B**, SHA-256 `87c46de5705e5eddd399c1a8c970d863fc3f1996bacca0a1f42cb09a5610ea12`, **32 entries** (`horizon-mod.toml`, `Cargo.toml`, `LICENSE`, `LICENSE-NOTICE.md`, `README.md`, 11 asset files, 12 data files, `src/lib.rs`, `src/station.rs`, `src/runtime.rs`, `uses.json`) | `sdk-sha256.txt`, `unzip -l` |
| skills line | `python3 ports/tools/check-skills.py --notes ports/charginggadgets` | §Skills below |

### Traps this migration hit (beyond `imfast`'s and `mekanismgenerators`')

1. **The block entity is out of the world during its own tick.** The hook's own doc says the
   block entity at `pos` reads as a `Mod` with an empty id and a save leaves it out, so asking
   `ItemPlace::BlockEntity { pos, slot: 1 }` for `ITEM_ENERGY` inside the tick can answer
   nothing. The tick therefore charges the stack the event hands it — the same reason 0.1.0
   used `native_item_energy(stack)` — while `/charginggadgets item` and `info` (outside a tick)
   use the capability.
2. **Item energy is a provider, not a declaration.** There is no `Registrar::item_energy`; the
   shape is `registrar.item_capability(&ITEM_ENERGY, id, |_, place| Some(Box::new(handler)))`,
   and the handler owns the `ItemPlace` (which is `Clone` but `#[non_exhaustive]`: clone it, do
   not build a variant you have not read).
3. **The stock `ItemComponentEnergy` writes an int component, not this port's key.** It would
   have changed the item's stored shape from `minecraft:custom_data.horizon_energy` (the 0.1.0
   and API-307 shape) to a bare int under a chosen component. `StationEnergy` keeps the old key
   through `ItemComponentHandler` over `minecraft:custom_data`, so an item charged before the
   migration keeps its charge and the `info` line still reads it.
4. **`Compound::get` takes a `JavaString` the facade does not export.** `Tag::get(&str)` is the
   public reader; for a `Compound` write `data.get(&KEY.into())` (the parameter type names it),
   and insert with `KEY.into()`.
5. **`#[horizon_part] impl BlockPart for …` is required beside the `BlockEntityPart` impl**,
   even when it is empty, or the block's subscription is missing.
6. **`registrar.item_part(registrar.item_key(id), part)` is a double mutable borrow.** Bind the
   key to a local first (the compiler says *cannot borrow `*registrar` as mutable more than once*).
7. **A menu is still a content declaration.** `ModContainer.menu` names
   `data/charginggadgets/horizon/menu/charging_station_menu.json` exactly as before, and the
   slots a `screen` hides are hidden on the server too (so shift-clicks skip them).
8. **The package root is the old `content/`.** `assets/` and `data/` move up unchanged; the
   content-only `horizon-mod.toml` is replaced by the root manifest, which must gain `[native]`
   and keep `api`/`side`/`capabilities`.
9. **`cargo test` needs `.horizon/sdk/` before the first `cargo horizon-mod` command.** Copying
   the slice from another port (or one `cargo horizon-mod hooks …`) fetches it; the fresh-clone
   run at the end proves it also fetches itself.
10. **`check --json` reports `"side": null` for a native package** even with `side = "both"`;
    that is the checker's report shape (`mekanismgenerators` trap 9), not a manifest error.
11. **The control leg changed meaning.** The old `EMBEDDED=1`/`.horizon-cache` control proved
    the content came from the embedded file; the package *is* the block now, so the control is
    an unmodified game (`PLAIN_GAME`): no block id, no item, no command.
12. **An unused field is a `-D warnings` failure.** `Sight.id` stopped being read when the
    readout became upstream's `Charging Station at …` line; the struct no longer carries it.
13. **Menu data slots are 16-bit on the wire.** `energy/32` and `capacity/32` keep both in
    range (upstream's own `ContainerData` choice); do not "simplify" them to raw FE.

### Gaps after the migration

Behaviour gaps 1–10 below (deviations and gaps, slice 1) are unchanged: the slice boundary is
the same. The migration adds three record-level ones:

14. **No in-game run at 0.2.0.** `horizon-launcher` is not installed here, so a game carrying
    this package cannot be built (an SDK package is compiled **into** the client by the
    launcher; there is no staging path for it). `tools/e2e.sh` was rewritten for `GAME=`
    (launcher build with the package) and `PLAIN_GAME=` (unmodified build, the control) and was
    **not run**; it refuses to start without `GAME`, and the 0.1.0 runs below remain the
    in-game evidence for the behaviour, which the byte-identical `runtime.rs`, the same saved
    keys, the same numbers and the same command lines make the same behaviour.
    **The first thing the in-game run must confirm** is the one claim this migration cannot
    test offline: that the content block gets its block entity from the *declaration*
    (`block_entity_type("charging_station").blocks(&["charging_station"])`) — scenario A's
    `info` line answers the station only if it does — and the second is that the station's
    charge slot is filled through the item provider (`ITEM_ENERGY`), which scenario F measures
    at exactly 2,500 FE a tick with conservation.
15. **The old `content_root` control leg is gone** (traps 11): the new control is the
    unmodified game.
16. **Only this mod's item is chargeable by the tick.** The tick reads the stack's own
    `minecraft:custom_data` key, as API 307 did, instead of asking `ITEM_ENERGY` at the
    out-of-world block entity (trap 1). Upstream asked the item's capability, so another mod's
    battery would have charged there; nothing else in this game answers `ITEM_ENERGY`, so the
    observable behaviour is the same, and the capability path is used everywhere the entity *is*
    in the world (`/charginggadgets item`, and any other mod's machine reaching the item).

## Verification (the 0.1.0 ladder, kept as history)

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6); checkouts
`ports/_shared/pins/horizon-1d63e0a` and `.../horizon-mixins-5a64af9`. Every
build/test/e2e ran through `ports/tools/build-slot.sh` with
`CARGO_INCREMENTAL=0`; no bare cargo, no `--target-dir`. Evidence:
`ports/_shared/evidence/charginggadgets/`.

| Step | Evidence | Result |
|---|---|---|
| runtime TDD (red) | `test-red.log` | **0 passed, 16 failed** — every body `todo!()` |
| runtime TDD (red, second cycle) | `test-green.log` history | two assertions were wrong (a coal loads *and* burns in its first tick): 14 passed / 2 failed, then green |
| runtime tests (green) | `test-green.log` | **16 passed**, 0 failed |
| fmt | `fmt.log`, `fmt-helper.log` | `cargo fmt --all -- --check` and `rustfmt --check mixins/charginggadgets/embedded_content.rs` both exit 0 (empty output) |
| clippy | `clippy.log` | `--all-targets --locked --offline -- -D warnings`, exit 0, `Finished dev profile` (re-run for the embedded slice) |
| stage | `stage.log` | `stage_changed: true`, 2 modules, 13 injections |
| check | `horizon-check.log` | `cargo_exit: 0`, **0 warnings** |
| embedded test | `horizon-test.log` | `test result: ok. **16 passed**; 0 failed; 1240 filtered out` |
| fast build | `horizon-build.log` | `cargo_exit: 0`, 0 warnings |
| e2e | `e2e.log` | **PASS**, `EXIT:0` |
| fail-capability | `e2e-selftest.log` | `SELFTEST=1` → **EXIT:1** (`the station holds 20000 FE, expected 19375`) |
| e2e, one file only (`EMBEDDED=1`, no `--dev-mod`) | `e2e-embedded.log` | **PASS**, `EXIT:0`; the control leg (same binary, `.horizon-cache` unusable) shows the ids unknown |
| embed fail-capability | `e2e-embedded-selftest.log` | `SELFTEST=1 EMBEDDED=1` → **EXIT:1** |
| log checker | `check-e2e-logs.py charginggadgets --strict` | `logs=5 ok=5 missing-exit=0 bad-exit=0` |
| `.hmod` build/check | `package.log` | 20,248 B, `f926468a…6117`, 24 entries, “no code: content only” |
| `.hnmod` (content embedded) | `package.log` | 33,658 B, `b02bd130…6e58`, 18 entries incl. `mixins/charginggadgets/data/charginggadgets-content.hmod` |
| `cargo horizon-mod check --kind native-mod` | `check-native.log` | exit 0, `native-mod … for 26.3` |
| load-check | `hmod-load.log` | **PASS** with `--dev-mod <the .hmod>` on the shared unmodded game; the same ids unknown without it (`block: minecraft:air`) |

### In-game transcript (quoted from `e2e.log`)

```
== A: a station placed from its item is empty ==
  Charging Station at [202, 101, 0]: 0/1000000 FE, burning 0/0 steps, fuel empty, charge empty, t=177
  block charginggadgets:charging_station, 0/1000000 FE
== B: a non-fuel item in the fuel slot makes no FE and is kept ==
  still 0 FE after 40 ticks, fuel minecraft:dirt x1
== C: an item in the charge slot of an empty station takes nothing ==
  the empty station gave nothing: item 0 FE
== D: a full buffer fills a 999,000 FE item to its cap, debited 1,000 ==
  the item stopped at 1000000/1000000, the station at 999000 FE
== E: one coal is 32 burn steps, exactly 20,000 FE ==
  Charging Station at [202, 101, 4]: 20000/1000000 FE, burning 0/0 steps, fuel empty, charge empty, t=527
== E2: three coals beside a discharged item conserve their FE ==
  three coals: station 0 FE + item 60000 FE = 60000 FE
== F: 2,500 FE a tick, and the station loses exactly what the item gains ==
  76 ticks: item 35000 -> 225000 (+190000), station 965000 -> 775000 (-190000)
== G: a non-chargeable item in the charge slot takes nothing ==
  a stone stack took nothing: the station still holds 10000 FE
== H: a charged station item fills the block it is placed as ==
  Charging Station held item: 999000/1000000 FE.
  Charging Station at [202, 101, 12]: 999000/1000000 FE, burning 0/0 steps, fuel empty, charge empty, t=1119
PASS: … an empty station charges nothing, a non-fuel item makes no FE, one coal is exactly 20,000 FE, a full buffer fills a 999,000 FE item to its cap for exactly 1,000 FE, the charge slot takes exactly 2,500 FE a tick with no FE created or lost, a non-chargeable item and an empty buffer do nothing, and a charged item fills the block it is placed as
EXIT:0
```

The run is fail-capable four ways: `SELFTEST=1` flips E's assertion and exits 1;
B, C, G are controls that must not change a number (a non-fuel item, an empty
station, a non-chargeable item); E2 and F assert **conservation** (nothing is
created or lost); and `hmod-load.sh` shows the same ids unknown without the
package.

### Packaged artifacts

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `charginggadgets.hmod` (0.2.0, the delivered package) | 48,087 B | `87c46de5705e5eddd399c1a8c970d863fc3f1996bacca0a1f42cb09a5610ea12` | 32 |
| `content/charginggadgets.hmod` (0.1.0, retired) | 20,248 B | `f926468ab6c22b7b1f19e5d2688e30d4126751c3cef16b19058f5baa90f76117` | 24 |
| `charginggadgets.hnmod` (0.1.0, retired) | 33,658 B | `b02bd130df827a615d843345876ded838d63535b8d82d5af4ae1ea84f02e6e58` | 18 |

The 0.2.0 `.hmod` is the single artifact a registry entry needs at this pin: it carries
`assets/`, `data/` and the Rust, so there is no embedded copy and no second package. The two
0.1.0 files are evidence in
`../_shared/evidence/charginggadgets/retired-hnmod-0.1.0/`.

### Changelog for the resubmission (verbatim)

> Supersedes #871, closed because Horizon retired `.hnmod` (#17992). Changes: flat SDK package
> with `api = "0.2"` and `[native]` pinned to `0b14f8a`/`e59da90`; the twelve injected helpers
> become `impl mod_api::Hooks` methods and Parts (a `BlockEntityPart` for the block entity, its
> tick and its screen, an `ItemPart` for the tooltip and placement), the container, the FE
> buffer and the item's charge become declarations (`block_container`, `block_energy`,
> `item_capability`), and the `content_root` embedded-content workaround is no longer needed
> because one `.hmod` carries the content and the code. `runtime.rs` and its 16 tests are
> unchanged (byte-identical), the content is the same files moved to the package root, and the
> harness commands are the same typed commands. Version `0.1.0` → `0.2.0`.

### Port version (reviewer rule, playbook §1.7)

The registry version is the **port's** version, not the original mod's: the
manifest carried `1.14.1`, which the reviewer flagged on earlier submissions.
`1.14.1 → 0.1.0` was the first change; the SDK migration takes it to **`0.2.0`**
(a resubmission is always a new version), which is the `version` in
`horizon-mod.toml` and in `Cargo.toml`. The 0.1.0 packaging note that follows is
history.

`content/horizon-mod.toml`, `mixins/charginggadgets/horizon-mixin.toml` `[mod]`
and the version path `tools/package.sh` copies; `tools/package.sh` now also
builds the content first, syncs it into `mixins/charginggadgets/data/` and runs
`cargo horizon-mod check --kind native-mod` on the bundle (the order
`disenchanting_table/tools/package.sh` already used). Two packaging runs gave
identical bytes (`f926468a…6117` / `b02bd130…6e58`, both sizes unchanged in the
manifest's byte count since `1.14.1` and `0.1.0` are the same length), both
`cargo horizon-mod check` kinds exit 0, and the `EMBEDDED=1` leg was re-run
because it compares the written cache file with `content/charginggadgets.hmod`:
`PASS` + `EXIT:0`, cache 20,248 B identical. Evidence:
`../_shared/evidence/charginggadgets/{e2e-embedded,check-final}.log`.

### Campaign gates at this touch

- `python3 ports/tools/precheck-mod.py charginggadgets` → exit 0,
  `DISPATCHABLE`, jar licence MIT (rank 0).
- `python3 ports/tools/check-e2e-logs.py charginggadgets --strict` → 2/2.
- `python3 ports/tools/check-skills.py --notes ports/charginggadgets` → the
  `Skills:` line below.
- `tools/test_template.py` and `tools/test_stage.py` are the inherited scaffold
  fixtures. They do not apply to this bundle: `test_template.py` runs the mixin
  CLI against a workspace with no `native-hooks.json` and stops with
  `E_HOOK_CATALOG` (a named hook needs the catalog), and `test_stage.py` drives
  the old `ports/horizon` layout (`git -C ports/horizon status` → 128). Both are
  kept unmodified so the scaffold stays comparable with `ports/_template`; the
  real ladder above (`tools/horizon.py stage → check → test → build`) is the
  evidence. Logs: `test_template.log`, `test_stage.log`. The same limitation is
  recorded by `mininggadgets`, `powah` and `quarryplus`.

## Review repairs (PR #526, 2026-11-11)

Review: `ports/_shared/horizon-mods/reviews/pr-526-charginggadgets.md` (CHANGES_REQUESTED, 1 P1).

| Reviewer's item | This job |
|---|---|
| **[P1] the entry says 0.1.0 / `b02bd130…` but the linked release `charginggadgets-1.14.1` is a different build (`1878b25f…`, version 1.14.1 inside)** | **aligned on the port's version**: the artifact in this tree is 0.1.0 (`horizon-mixin.toml`, `horizon-native.json`), the registry entry now carries its digest, and the 1.14.1 release is an orphan that must not be linked — that release text is the **orchestrator's** to correct when it re-uploads `charginggadgets-0.1.0` |
| the native ZIP carries no LICENSE/notice | **fixed** by the packager |

The packager change also replaced the digest: the entry now names
`fff31eadce875532710fd803233289b908b7626f13462898e39950e24d79c545` (35,305 B), which is the
exact bytes of `charginggadgets.hnmod` in this tree.

Artifact at that time: `charginggadgets.hnmod` **35,305 B**
`fff31eadce875532710fd803233289b908b7626f13462898e39950e24d79c545`.

*Superseded by the 0.2.0 flat package ([§Migration](#migration-to-the-sdk-format-version-020-2026-11-18)):
`charginggadgets.hmod` **48,087 B**, SHA-256
`87c46de5705e5eddd399c1a8c970d863fc3f1996bacca0a1f42cb09a5610ea12`, 32 entries.*

## Skills

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

`cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test` were
run before hand-back. The 0.1.0 `#[allow(private_interfaces, …)]` exception for
the injected module's own `Game`/`ModBlockEntity` parameters is gone with the
injected module: the SDK's hooks hand out borrowed views and traits, so no lint
suppression is needed anywhere in this crate.

## Deviations and gaps (slice 1)

1. **Breaking the station loses its FE.** Upstream's `getDrops` puts the stored
   energy on the dropped item; the pin has no loot-function hook for a mod
   block's drops, so the drop is the plain block item. The two slots still spill
   through the engine's container handling.
2. **The block item's charge is a capability, not a private component.** It
   declares `Capabilities.Energy.ITEM`, so a station item can be charged like a
   battery (upstream's own station refuses it). Deliberate: it is the pin's form
   of `charginggadgets:energy`, it keeps the place-to-restore behaviour, and it
   is what gives this port's game a chargeable item for the e2e.
3. **No GUI texture and no bars.** The carried `textures/gui/charging_station.png`
   is not drawn: the menu uses the pin's plain panel and slot frames, and the
   energy/burn bars become numeric readouts. No hover tooltip inside the menu.
4. **No per-slot insert rules.** Upstream's `RestrictedSlot.mayPlace` and
   `ChargerItemHandler.insertItem` refuse a non-fuel item in slot 0 and a
   non-chargeable one in slot 1; here both slots accept any item and the **tick**
   refuses what it cannot burn or charge (a non-fuel item sits in the fuel slot
   doing nothing, a non-chargeable item sits in the charge slot doing nothing).
   The menu-slot-rules hook is library-side and cannot reach the bundle.
5. **A stack of more than one item in the charge slot is left alone** (the
   engine's item-energy helpers act on one item of a stack; upstream keeps one
   component for the whole stack). The e2e only ever puts one item there.
6. **No config file.** `chargerMaxEnergy` is the constant 1,000,000 (upstream's
   default); `config/` is not created.
7. **No particles, sounds or `lit`-style visuals**, no comparator output, no
   `getDisplayName` title beyond the block's own name, and no redstone/hopper
   automation tests (the container faces are declared, but only `ctl container`
   drove them).
8. **Block-to-block FE transfer is not exercised**: this build has no second
   energy block, so the `Capabilities.Energy.BLOCK` insert path is declared and
   read back through `/charginggadgets energy` but no neighbour filled it. The
   pin has no block-to-block helper (ARCHITECTURE.md §3).
9. **The advancement is carried unchanged**; it is inert unless its 1.21.1
   format matches 26.3 (nothing reads it in-game here).
10. **Not carried:** the `charginggadgets:energy` component registration, the
    item group code, `pack.mcmeta`, and any JEI/Jade/recipe-viewer integration
    (upstream has none of its own).

## Provenance (upstream analysis)

Repository `https://github.com/Direwolf20-MC/ChargingGadgets`, commit
**`5f6d7b63bbc1397d9b7a528b31c431f24da4ec11`** (2024-08-10, "Update for 1.21.1",
`mod_version=1.14.1`); shallow clone deleted after the findings were written.
Files read and behaviour extracted: [`FINDINGS.md`](FINDINGS.md). The jar stays
as licence evidence in `ports/_shared/licence-cache/charginggadgets-1.14.1.jar`
(provenance: `ports/_shared/carry-work/charginggadgets/SOURCE.md`). Clean-room
rewrite: no Java was copied or translated and no jar was decompiled.
