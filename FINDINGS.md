# Upstream analysis — Charging Gadgets 1.14.1

**Repository:** <https://github.com/Direwolf20-MC/ChargingGadgets>, shallow clone at
commit `5f6d7b63bbc1397d9b7a528b31c431f24da4ec11` (2024-08-10, "Update for 1.21.1"),
`gradle.properties` `mod_version=1.14.1`, `minecraft_version=1.21.1`,
`neo_version=21.1.4` — the version of the ATM10 jar. **MIT** (`LICENSE.md`,
"Copyright © 2018 Direwolf20"; the jar's `neoforge.mods.toml` `license="MIT
(https://github.com/Direwolf20-MC/ChargingGadgets/blob/master/LICENSE.md)"`).

Analysis only: the clone was read for behaviour and deleted afterwards; the jar
stays as licence evidence (`ports/_shared/licence-cache/charginggadgets-1.14.1.jar`,
SHA-1 `3208ea2ca10d786895d03c2cd3a124d14393b357`, SHA-256
`411bae6c95c1c4ab19e0431785341dabe39b6eaf5f132abd00514264243ba2ac`). No jar was
decompiled and no Java was copied: the Rust is a clean-room rewrite.

## Files read

| File | What it says |
|---|---|
| `blocks/chargingstation/ChargingStationTile.java` | the tick: `tryBurn`, `burn`, `initBurn`, `chargeItem`; slots `FUEL(0)`, `CHARGE(1)`; saved keys `inv`, `energy`, `counter`, `maxburn`; the menu data `[energy/32, max/32, counter, maxburn]` |
| `blocks/chargingstation/ChargingStationBlock.java` | `Properties.of().strength(2f)`, horizontal `facing` placed opposite the player's, a server-only ticker, the BE, `useWithoutItem` opens the menu, `onRemove` drops the slots, `getDrops` puts the stored FE on the dropped item |
| `blocks/chargingstation/ChargingStationContainer.java` | two `RestrictedSlot`s at (65, 43) and (119, 43) plus the player inventory; `quickMoveStack` moves between the two slots and the inventory |
| `blocks/chargingstation/ChargingStationItem.java` | the block item's tooltip (the `screen.charginggadgets.energy` line, only when charged) and `updateCustomBlockEntityTag` (a charged item fills the block it is placed as) |
| `blocks/chargingstation/ChargingStationScreen.java` | 176 x 166 GUI, the energy bar (x 8, height 70) and the burn bar (x 66), the tooltip on hover |
| `capabilities/ChargerEnergyStorage.java` | capacity = `Config.GENERAL.chargerMaxPower` (default **1,000,000**), `maxInOut` = **1,000,000**, `receiveEnergy` yes, `extractEnergy`/`canExtract` **no** |
| `capabilities/ChargerItemHandler.java` | slot rules: fuel takes anything with `getBurnTime(SMELTING) > 0` (or a bucket), the charge slot takes an `EnergyStorage.ITEM` item and only when empty |
| `Config.java` | `general.chargerMaxEnergy`, default 1000000, range 0..`Integer.MAX_VALUE` |
| `CGDataComponents.java` | the `charginggadgets:energy` int component the block item keeps the charge in |
| `assets/…`, `data/…` | one blockstate (4 horizontal facings), an `orientable_with_bottom` model, 5 block textures + 1 GUI texture, `en_us.json` (4 keys), the shaped recipe (lapis/iron/coal block/redstone), the loot table, the pickaxe tag, 1 advancement |

## Behaviour extracted (and what the port does)

| Area | Upstream 1.14.1 | This port |
|---|---|---|
| capacity | 1,000,000 FE (`chargerMaxEnergy`) | same constant; API 142 buffer |
| insert / extract | insert up to 1,000,000 a call from any side; **no** extract | `ModEnergyBuffer { max_insert 1,000,000, max_extract 0 }`, insert faces all, extract none |
| burn tick | one step per server tick while `receiveEnergy(625, true) > 0`, adding 625 FE | `runtime::Station::burn_tick`, same rule (`room() < 625` stops it) |
| fuel | any item with a smelting burn time; `counter = floor(burnTime)/50`; the item is shrunk (or replaced by its crafting remainder); nothing is consumed while the buffer cannot take a full step | `FuelSlot` over the engine's `minecraft:cooking_fuel` value (`Providers::burn_time`), same counter, same remainder, same condition |
| one coal | 1600 / 50 = 32 steps x 625 = **20,000 FE** | verified in game (`e2e.log` phase E) |
| reload | `burn()` calls `initBurn()` when the counter reaches 0, so the next fuel loads in the same tick (no gap) and two items can be taken in one tick | reproduced (`load_fuel` inside `burn_step`) |
| quirk | a fuel under 50 smelting ticks gives `counter = 0`, and `counter--` makes it -1, so one item is consumed per tick | reproduced and unit-tested (`a_fuel_under_fifty_smelting_ticks_burns_one_item_a_step`) |
| charge | `energy.receiveEnergy(min(stored, 2500), false)`, then `energyStorage.consumeEnergy(took)` | `runtime::Station::charge_step`, same `min(stored, 2500, room, item limit)` |
| what charges | only an item with `Capabilities.Energy.ITEM`; a non-chargeable item is left alone | the engine's `native_item_energy`/`native_item_energy_insert` (API 307); an item without a buffer takes nothing |
| slots | 0 fuel, 1 charge; hoppers reach both (plain `ItemStackHandler`); the charge slot takes one item at a time | `ModContainer` 2 slots, all faces in and out; a stack of more than one in the charge slot is left alone (the engine's helpers act on one item) |
| menu | 2 slots + 4 data slots (`energy/32`, `max/32`, `counter`, `maxburn`) | `charginggadgets:charging_station_menu` with the same 4 data slots; the screen readout draws the FE and the burn |
| breaking | the drop carries the stored FE in the `charginggadgets:energy` component, and the two slots spill | **gap**: the drop does not carry the charge (no loot-function hook); the slots spill through the engine's container handling |
| placing | `updateCustomBlockEntityTag` fills the new block from the item's component | the block item declares an **item energy buffer** (the pin's form of that component) and `place_block` (API 14) fills the block |
| tooltip | the energy line when charged | same, from the jar's own lang key |
| config | `chargerMaxEnergy` | not ported (the default is the constant) |

## Where the rewrite differs, and why

1. **The block item's charge is a `Capabilities.Energy.ITEM` buffer** (API 307),
   not a private component. Upstream's `charginggadgets:energy` component has no
   capability, so upstream's own station refuses to charge a station item; here
   the item answers the capability, so a station item can hold and receive FE
   like a battery, and placing it restores the block. This is the same promotion
   `mininggadgets` made for its `EnergyStorageItemstack`, and it is what makes a
   chargeable item exist in this port's own game for the e2e.
2. **No `inv` compound**: the engine saves the two slots as `Items` beside the
   block entity's compound, so only `energy`, `counter` and `maxburn` live in
   `nbt`.
3. **No `getDrops` override**: nothing puts the station's FE back on the dropped
   item (recorded gap). Upstream also spills both slots on removal; the engine's
   container handling does that for a mod block.
4. **No screen background texture**: the carried `textures/gui/charging_station.png`
   is not drawn (the pin's plain panel and slot frames are used, as `botanypots`
   does), so the energy/burn bars are numeric readouts instead of blitted bars.
5. **No per-slot `mayPlace`**: the pin's `menu_slot_rules` hook lives in the
   library crate (`horizon_client::menu`), which cannot name the gameplay-side
   bundle module, so the tick refuses what it cannot burn or charge instead of
   the slot refusing the insert. Recorded as a gap.
6. **The `charginggadgets` item group** is carried as a content creative tab
   (`data/charginggadgets/horizon/creative_tab/`), since upstream's is code.
