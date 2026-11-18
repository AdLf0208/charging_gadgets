//! The Charging Station's glue: the block entity Part, the block's container
//! and energy declarations, the station item's own charge, the saved state and
//! the `/charginggadgets` command bodies.
//!
//! Every rule lives in [`crate::runtime`] (the retired bundle's `runtime.rs`,
//! byte-identical); this file reads the game, calls it and writes the result
//! back. Its tests pin what it *declares* (two slots and the menu, an
//! insert-only buffer, the item's own buffer), that the saved compound reads
//! back, the item's charge key, and the exact line `/charginggadgets info`
//! prints.
use mod_api::{
    self, BlockEntityEvent, BlockEntityPart, BlockEntityTickEvent, BlockEntityTickReply,
    BlockEntityTicksEvent, BlockEntityTypeKey, BlockPart, ClientHost, ComponentId, Compound,
    EnergyHandler, Hand, Host, HostTransaction, ITEM_ENERGY, ItemComponentHandler, ItemPart,
    ItemPlace, ItemResource, ItemStack, LevelSide, ModContainer, ModEnergyBuffer,
    OnBlockPlacedByEvent, Registrar, Screen, ScreenDrawEvent, ScreenEvent, ServerHost, Tag, Text,
    TooltipEvent, Transaction,
};

use crate::runtime::{CAPACITY, FuelSlot, ItemBuffer, MAX_TRANSFER, Station};

/// The station block and its item (`BlockRegistry.CHARGING_STATION`).
pub(crate) const STATION: &str = "charginggadgets:charging_station";

/// The block entity type's path, in this mod's namespace.
const STATION_TYPE: &str = "charging_station";

/// The menu type the content declares (`data/charginggadgets/horizon/menu/`).
pub(crate) const MENU: &str = "charginggadgets:charging_station_menu";

/// Upstream's slot order (`ChargingStationTile.Slots`).
const FUEL_SLOT: usize = 0;
const CHARGE_SLOT: usize = 1;
const SLOT_COUNT: usize = 2;

/// Upstream's saved keys (`saveAdditional`).
const KEY_ENERGY: &str = "energy";
const KEY_COUNTER: &str = "counter";
const KEY_MAX_BURN: &str = "maxburn";

/// Where the station item keeps its charge: the retired `.hnmod`'s shape, the
/// int `horizon_energy` under `minecraft:custom_data` (`native_item_energy`,
/// API 307), so a charged item made before the migration keeps its charge.
const CUSTOM_DATA: &str = "minecraft:custom_data";
const ITEM_ENERGY_KEY: &str = "horizon_energy";

/// `minecraft:custom_data` as the component the item's charge lives in.
fn custom_data() -> ComponentId {
    ComponentId::new(CUSTOM_DATA).expect("`minecraft:custom_data` is a namespaced component id")
}

/// The block entity type declared for the station, built in `Hooks::register`.
pub(crate) fn declare(registrar: &mut dyn Registrar) -> BlockEntityTypeKey {
    registrar
        .block_entity_type(STATION_TYPE)
        .blocks(&[STATION_TYPE])
        .build()
}

/// The station's container: two slots every face reaches — fuel 0, charge 1 —
/// and the station menu. Upstream's `ChargerItemHandler` is a plain
/// `ItemStackHandler`, so automation reaches both slots from every side; the
/// per-item rules live in `insertItem`, which a face list cannot express (the
/// tick refuses anything it cannot burn or charge).
pub(crate) fn container() -> ModContainer {
    ModContainer {
        slots: SLOT_COUNT,
        faces: std::array::from_fn(|_| vec![FUEL_SLOT, CHARGE_SLOT]),
        inputs: vec![FUEL_SLOT, CHARGE_SLOT],
        outputs: vec![FUEL_SLOT, CHARGE_SLOT],
        menu: Some(MENU.to_owned()),
    }
}

/// The station's FE buffer (`native_block_entity_energy`, API 142): 1,000,000
/// FE, any side may fill it up to a whole buffer a call, and nothing may drain
/// it (`ChargerEnergyStorage.canExtract` is false and `extractEnergy` answers 0).
pub(crate) fn battery() -> ModEnergyBuffer {
    ModEnergyBuffer {
        capacity: CAPACITY,
        max_insert: MAX_TRANSFER,
        max_extract: 0,
        insert_faces: [true; 6],
        extract_faces: [false; 6],
    }
}

/// The station item's own buffer (`native_item_energy_buffer`, API 307): the
/// same capacity, filled only — upstream fills the block from the item when it
/// is placed and never drains it.
pub(crate) fn item_buffer() -> (u32, u32) {
    (CAPACITY as u32, MAX_TRANSFER as u32)
}

/// The station's menu (`native_screen`, API 19): upstream's 176 x 166 panel
/// with its two slots at `ChargingStationContainer`'s pixels.
pub(crate) fn menu() -> Screen {
    Screen {
        width: 176.0,
        height: 166.0,
        slots: vec![Some([65.0, 43.0]), Some([119.0, 43.0])],
        inventory_y: 84.0,
        inventory_x: 8.0,
        own_labels: false,
        own_background: false,
        ..Screen::default()
    }
}

/// The station saved in a block entity's compound; a fresh one is empty.
pub(crate) fn read(saved: &Tag) -> Station {
    Station {
        energy: int(saved, KEY_ENERGY).clamp(0, CAPACITY),
        counter: int(saved, KEY_COUNTER),
        max_burn: int(saved, KEY_MAX_BURN),
    }
}

/// Write the tick's result back, under upstream's keys: the buffer's amount
/// (`energy`, the int the energy capability reads), the burning counter and the
/// steps the last fuel item gave.
pub(crate) fn write(saved: &mut Tag, state: &Station) {
    let Tag::Compound(nbt) = saved else {
        return;
    };
    nbt.insert(KEY_ENERGY.into(), Tag::Int(state.energy));
    nbt.insert(KEY_COUNTER.into(), Tag::Int(state.counter));
    nbt.insert(KEY_MAX_BURN.into(), Tag::Int(state.max_burn));
}

/// A saved int, 0 when absent or of another type.
fn int(saved: &Tag, key: &str) -> i32 {
    match saved.get(key) {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    }
}

/// The charge the item at `data` keeps, 0 when it holds none.
pub(crate) fn item_energy_of(data: &Compound) -> i32 {
    match data.get(&ITEM_ENERGY_KEY.into()) {
        Some(Tag::Int(energy)) => (*energy).max(0),
        _ => 0,
    }
}

/// Keep `energy` as the item's charge; 0 removes it, so an empty item stacks
/// with a new one.
pub(crate) fn with_item_energy(data: &mut Compound, energy: i32) {
    if energy > 0 {
        data.insert(ITEM_ENERGY_KEY.into(), Tag::Int(energy));
    } else {
        data.remove(&ITEM_ENERGY_KEY.into());
    }
}

/// What `/charginggadgets info` prints about one block entity.
pub(crate) struct Sight {
    pub(crate) pos: [i32; 3],
    pub(crate) energy: i32,
    pub(crate) counter: i32,
    pub(crate) max_burn: i32,
    pub(crate) fuel: Option<(String, u32)>,
    pub(crate) charge: Option<Charge>,
}

/// The item in the charge slot, as the readout prints it.
pub(crate) struct Charge {
    pub(crate) item: String,
    pub(crate) count: u32,
    pub(crate) buffer: Option<(i32, i32)>,
}

impl Sight {
    /// The report line, with the world time it was taken at: `t=` is what the
    /// in-game check uses to prove a reply is fresh.
    pub(crate) fn line(&self, t: i64) -> String {
        let fuel = match &self.fuel {
            Some((item, count)) => format!("{item} x{count}"),
            None => "empty".to_owned(),
        };
        let charge = match &self.charge {
            Some(Charge {
                item,
                count,
                buffer: Some((amount, capacity)),
            }) => format!("{item} x{count} {amount}/{capacity} FE"),
            Some(Charge {
                item,
                count,
                buffer: None,
            }) => format!("{item} x{count} no buffer"),
            None => "empty".to_owned(),
        };
        format!(
            "Charging Station at {:?}: {}/{} FE, burning {}/{} steps, fuel {fuel}, charge {charge}, \
             t={t}",
            self.pos, self.energy, CAPACITY, self.counter, self.max_burn,
        )
    }
}

/// `/charginggadgets info <pos>`: the report line, or `None` when no charging
/// station stands there. The block id is the test, so a chest at that position
/// is "not a charging station", not a station at 0 FE.
pub(crate) fn info_line(host: &mut dyn ServerHost, pos: [i32; 3]) -> Option<String> {
    let level = host.level(host.dimension())?;
    if level.block(&*host, pos)?.id(&*host) != STATION {
        return None;
    }
    let entity = host.block_entity(level, pos)?;
    let saved = Tag::Compound(entity.data(&*host)?);
    let fuel = entity
        .item_slot(&*host, FUEL_SLOT as u32)
        .ok()
        .flatten()
        .map(|stack| (stack.item().id(&*host), stack.count()));
    let charge = entity
        .item_slot(&*host, CHARGE_SLOT as u32)
        .ok()
        .flatten()
        .map(|stack| Charge {
            item: stack.item().id(&*host),
            count: stack.count(),
            buffer: item_buffer_of(host, level, pos),
        });
    let station = read(&saved);
    let sight = Sight {
        pos,
        energy: station.energy,
        counter: station.counter,
        max_burn: station.max_burn,
        fuel,
        charge,
    };
    Some(sight.line(level.game_time(&*host).unwrap_or(0)))
}

/// The charge slot's item buffer, as the item energy capability answers it:
/// its amount and capacity, or `None` for an item without one.
fn item_buffer_of(
    host: &mut dyn ServerHost,
    level: mod_api::LevelRef,
    pos: [i32; 3],
) -> Option<(i32, i32)> {
    let place = ItemPlace::BlockEntity {
        level,
        pos,
        side: None,
        slot: CHARGE_SLOT,
    };
    let handler = place.capability(host, &ITEM_ENERGY)?;
    Some((
        i32::try_from(handler.amount(host)).unwrap_or(i32::MAX),
        i32::try_from(handler.capacity(host)).unwrap_or(i32::MAX),
    ))
}

/// `/charginggadgets energy <pos> <amount>`: set the buffer's amount, the
/// harness control the in-game check uses. Answers with the amount it stored,
/// or `None` when no charging station stands there (a foreign block entity is
/// left alone: writing `energy` into a chest's saved data would be a foreign key
/// in someone else's compound).
pub(crate) fn set_energy(host: &mut dyn ServerHost, pos: [i32; 3], amount: i32) -> Option<i32> {
    let level = host.level(host.dimension())?;
    if level.block(&*host, pos)?.id(&*host) != STATION {
        return None;
    }
    let entity = host.block_entity(level, pos)?;
    let energy = amount.clamp(0, CAPACITY);
    let mut patch = Compound::new();
    patch.insert(KEY_ENERGY.into(), Tag::Int(energy));
    // `merge_data` is `/data merge block`: it loads the patch and marks the chunk
    // changed, which is the `setChanged` the retired bundle had to fake.
    entity.merge_data(host, patch).ok().map(|_| energy)
}

/// `/charginggadgets item <pos> <amount>`: charge the item in the station's
/// charge slot up to `target` FE, or `None` when the slot holds no item with a
/// buffer. Only a fill is possible: the station item's buffer has no extract
/// limit (`canExtract` is false upstream), so `Some(target)` means the item
/// reached it.
pub(crate) fn set_item_energy(
    host: &mut dyn ServerHost,
    pos: [i32; 3],
    target: i32,
) -> Option<i32> {
    let level = host.level(host.dimension())?;
    if level.block(&*host, pos)?.id(&*host) != STATION {
        return None;
    }
    let place = ItemPlace::BlockEntity {
        level,
        pos,
        side: None,
        slot: CHARGE_SLOT,
    };
    fill_to(host, &place, target)
}

/// `/charginggadgets held <amount>`: the same for the item in the acting
/// player's main hand (the harness's way to make a charged station item, as
/// `/mininggadgets energy` does).
pub(crate) fn set_held_energy(host: &mut dyn ServerHost, player: u128, target: i32) -> Option<i32> {
    let place = ItemPlace::PlayerHand {
        player,
        hand: Hand::Main,
    };
    fill_to(host, &place, target)
}

/// Fill the item at `place` up to `target` FE, or `None` when it has no buffer.
/// Answers what the item holds after the fill, which is `target` unless its own
/// capacity or per-insert limit stopped short.
fn fill_to(host: &mut dyn ServerHost, place: &ItemPlace, target: i32) -> Option<i32> {
    let handler = place.capability(host, &ITEM_ENERGY)?;
    let held = i32::try_from(handler.amount(host)).unwrap_or(i32::MAX);
    if target > held {
        let offer = u32::try_from(target - held).unwrap_or(u32::MAX);
        let _ = host.transaction(|host, tx| {
            Ok::<u32, std::convert::Infallible>(handler.insert(host, tx, offer))
        });
    }
    Some(i32::try_from(handler.amount(host)).unwrap_or(i32::MAX))
}

/// The station item's charge as `Capabilities.Energy.ITEM`: the int under
/// `minecraft:custom_data`, filled only, kept in the item where it lives (the
/// stock `ItemComponentHandler` rewrites that item in the caller's transaction,
/// so a stack of more than one splits off the charged item).
pub(crate) struct StationEnergy {
    data: ItemComponentHandler,
    capacity: u32,
    max_insert: u32,
}

impl StationEnergy {
    /// The charge of the item at `place`.
    pub(crate) fn new(place: ItemPlace) -> Self {
        let (capacity, max_insert) = item_buffer();
        Self {
            data: ItemComponentHandler::new(place, custom_data()),
            capacity,
            max_insert,
        }
    }

    /// What the item holds now.
    fn held(&self, host: &mut dyn ServerHost) -> i32 {
        match self.data.read(host) {
            Some(Tag::Compound(data)) => item_energy_of(&data),
            _ => 0,
        }
    }

    /// Rewrites the item as holding `energy`: `moved` when that went through,
    /// else 0.
    fn rewrite(&self, host: &mut dyn ServerHost, tx: &Transaction, energy: i32, moved: i32) -> i32 {
        let mut data = match self.data.read(host) {
            Some(Tag::Compound(data)) => data,
            _ => Compound::new(),
        };
        with_item_energy(&mut data, energy);
        let value = (!data.is_empty()).then_some(Tag::Compound(data));
        if self.data.write(host, tx, value) {
            moved
        } else {
            0
        }
    }
}

impl EnergyHandler for StationEnergy {
    fn amount(&self, host: &mut dyn ServerHost) -> u32 {
        u32::try_from(self.held(host)).unwrap_or(0)
    }

    fn capacity(&self, _: &mut dyn ServerHost) -> u32 {
        self.capacity
    }

    fn insert(&self, host: &mut dyn ServerHost, tx: &Transaction, amount: u32) -> u32 {
        let held = self.held(host);
        let room = i32::try_from(self.capacity)
            .unwrap_or(i32::MAX)
            .saturating_sub(held);
        match (amount, self.max_insert) {
            (0, _) | (_, 0) => 0,
            (amount, max) => {
                let went = i32::try_from(amount)
                    .unwrap_or(i32::MAX)
                    .min(i32::try_from(max).unwrap_or(i32::MAX))
                    .min(room);
                match went {
                    0 => 0,
                    went => u32::try_from(self.rewrite(host, tx, held + went, went)).unwrap_or(0),
                }
            }
        }
    }

    fn extract(&self, _: &mut dyn ServerHost, _: &Transaction, _: u32) -> u32 {
        0
    }
}

/// The station's behaviour: a block entity, ticked on the server, with a menu.
pub(crate) struct StationPart;

#[mod_api::horizon_part]
impl BlockPart for StationPart {}

#[mod_api::horizon_part]
impl BlockEntityPart for StationPart {
    /// `EntityBlock.newBlockEntity`: an empty saved compound; the tick fills it.
    fn block_entity(
        &self,
        _level: LevelSide,
        _event: &mut BlockEntityEvent<'_>,
    ) -> Option<Compound> {
        Some(Compound::new())
    }

    /// `EntityBlock.getTicker`: it ticks. Never clear a value an earlier mod set.
    fn block_entity_ticks(&self, _host: &dyn Host, event: &mut BlockEntityTicksEvent<'_>) {
        event.ticks = true;
    }

    /// One server tick: burn (`tryBurn` + `burn` + `initBurn`), then charge the
    /// item in the charge slot, then the menu's data slots. The payload is left
    /// `Changed` only when something moved, so an idle station does not dirty
    /// its chunk every tick (the retired bundle marked it every tick).
    fn block_entity_tick(&self, host: &mut dyn ServerHost, event: &mut BlockEntityTickEvent<'_>) {
        let resized = event.entity.items.len() != SLOT_COUNT;
        if resized {
            event.entity.items.resize(SLOT_COUNT, None);
        }
        let mut saved = Tag::Compound(std::mem::take(&mut event.entity.nbt));
        let before = read(&saved);
        let mut station = before;
        {
            let mut fuel = SlotFuel {
                host: &*host,
                entity: &mut event.entity,
            };
            station.burn_tick(&mut fuel);
        }
        charge_slot(&mut station, &mut event.entity.items[CHARGE_SLOT]);
        write(&mut saved, &station);
        if let Tag::Compound(nbt) = saved {
            event.entity.nbt = nbt;
        }
        let data = station.data_slots().to_vec();
        let data_changed = event.entity.data != data;
        if data_changed {
            event.entity.data = data;
        }
        if !resized && !data_changed && station == before {
            *event.reply = BlockEntityTickReply::Unchanged;
        }
    }

    /// The station menu's layout.
    fn screen(&self, _host: &dyn Host, _event: &mut ScreenEvent<'_>) -> Option<Screen> {
        Some(menu())
    }

    /// The readouts (`native_screen_draw`, API 19): the buffer and the burning
    /// steps, from the menu's data slots `[energy/32, capacity/32, counter,
    /// maxburn]`.
    fn screen_draw(&self, _host: &dyn ClientHost, event: &mut ScreenDrawEvent<'_>) {
        let energy = event.data.first().copied().unwrap_or(0) * 32;
        let capacity = event.data.get(1).copied().unwrap_or(0) * 32;
        let counter = event.data.get(2).copied().unwrap_or(0);
        let max_burn = event.data.get(3).copied().unwrap_or(0);
        let energy_text = format!("{energy}/{capacity} FE");
        let burn_text = if max_burn > 0 && counter > 0 {
            format!("{counter}/{max_burn}")
        } else {
            "-".to_owned()
        };
        let [x, y] = event.origin;
        event
            .draw
            .text(&energy_text, [x + 8.0, y + 8.0], 0xFF404040, false);
        event
            .draw
            .text(&burn_text, [x + 66.0, y + 8.0], 0xFF404040, false);
    }
}

/// The charge slot: `min(stored, 2,500)` FE into the item there, debited by what
/// it accepted. Nothing moves for a non-chargeable item, a full item or an empty
/// station.
///
/// The block entity is **out of the world during its own tick** (the hook's
/// own doc says so), so the item is read and rewritten here rather than through
/// `ItemPlace::BlockEntity`'s capability: the retired bundle's
/// `native_item_energy` worked on the stack for the same reason. Only this
/// mod's item carries a buffer, which is the same set API 307 answered for.
fn charge_slot(station: &mut Station, slot: &mut Option<ItemStack>) {
    if station.energy <= 0 {
        return;
    }
    let Some(stack) = slot.as_ref() else {
        return;
    };
    let moved = station.charge_step(Some(ItemBuffer {
        energy: stack_charge(stack),
        capacity: CAPACITY,
        max_insert: MAX_TRANSFER,
    }));
    if moved <= 0 {
        return;
    }
    match charged(stack, moved) {
        Some((took, charged)) => {
            // The charge step already debited `moved`; hand back what the item
            // did not take so no FE is lost.
            station.energy += moved - took;
            *slot = Some(charged);
        }
        // Nothing could be written: the station keeps its FE.
        None => station.energy += moved,
    }
}

/// The charge one item of `stack` keeps, 0 when it has none or the stack holds
/// more than one item (the engine's item helpers act on one item at a time).
fn stack_charge(stack: &ItemStack) -> i32 {
    if stack.count() != 1 {
        return 0;
    }
    match custom_data_of(stack) {
        Some(data) => item_energy_of(&data),
        None => 0,
    }
}

/// `stack`'s `minecraft:custom_data`, if it has one.
fn custom_data_of(stack: &ItemStack) -> Option<Compound> {
    let patch = stack.components()?.as_nbt();
    match patch.get(&CUSTOM_DATA.into()) {
        Some(Tag::Compound(data)) => Some(data.clone()),
        _ => None,
    }
}

/// One item of `stack` holding `amount` more FE: what it took and the item it
/// becomes, or `None` when it holds no buffer or can take nothing.
fn charged(stack: &ItemStack, amount: i32) -> Option<(i32, ItemStack)> {
    if stack.count() != 1 {
        return None;
    }
    let mut data = custom_data_of(stack).unwrap_or_default();
    let held = item_energy_of(&data);
    let room = CAPACITY.saturating_sub(held).max(0);
    let took = amount.clamp(0, MAX_TRANSFER).min(room);
    if took == 0 {
        return None;
    }
    with_item_energy(&mut data, held + took);
    let mut charged = stack.clone();
    let mut patch = Compound::new();
    patch.insert(CUSTOM_DATA.into(), Tag::Compound(data));
    charged.apply_components(&patch).ok()?;
    Some((took, charged))
}

/// The fuel slot, as `ChargingStationTile.initBurn` reads it: the item's
/// smelting burn time and the shrink (or the crafting remainder) that takes it.
struct SlotFuel<'a, 'g> {
    host: &'a dyn ServerHost,
    entity: &'a mut mod_api::ModBlockEntityView<'g>,
}

impl FuelSlot for SlotFuel<'_, '_> {
    fn peek(&self) -> Option<u32> {
        let stack = self.entity.items.first()?.as_ref()?;
        (stack.count() > 0).then(|| burn_time(self.host, stack))
    }

    fn take(&mut self) {
        let Some(Some(stack)) = self.entity.items.first_mut() else {
            return;
        };
        let left = stack.clone().shrink(1);
        self.entity.items[FUEL_SLOT] = left.or_else(|| remaining(self.host, stack));
    }
}

/// The engine's fuel value (`minecraft:cooking_fuel.burn_time`), 0 if none.
fn burn_time(host: &dyn Host, stack: &ItemStack) -> u32 {
    host.item_burn_time(stack.item()).unwrap_or(0).max(0) as u32
}

/// `ItemStack.getCraftingRemainingItem`: the bucket a lava bucket leaves.
fn remaining(host: &dyn Host, stack: &ItemStack) -> Option<ItemStack> {
    host.item_crafting_remainder(stack)
}

/// The station item's behaviour: its charge line in the tooltip and the charge
/// it hands the block it is placed as.
pub(crate) struct StationItemPart;

#[mod_api::horizon_part]
impl ItemPart for StationItemPart {
    /// The item's tooltip (`native_tooltip`, API 12): upstream's
    /// `screen.charginggadgets.energy` line, drawn only when it holds a charge.
    fn tooltip(&self, host: &dyn ClientHost, event: &mut TooltipEvent<'_>) {
        let energy = item_energy(host, event.stack);
        if energy == 0 {
            return;
        }
        event.lines.push((
            Text::translate(
                "screen.charginggadgets.energy",
                vec![
                    Text::literal(energy.to_string()),
                    Text::literal(CAPACITY.to_string()),
                ],
            ),
            0xFF55FF55,
        ));
    }

    /// A charged station item fills the block it becomes
    /// (`ChargingStationItem.updateCustomBlockEntityTag`), API 14's
    /// `place_block` at the retired pin.
    fn on_block_placed_by(&self, host: &mut dyn ServerHost, event: &mut OnBlockPlacedByEvent<'_>) {
        let amount = item_energy(host, event.stack).clamp(0, CAPACITY);
        if amount == 0 {
            return;
        }
        let Some(level) = host.level(host.dimension()) else {
            return;
        };
        if level.block(&*host, event.pos).map(|block| block.id(&*host)) != Some(STATION.to_owned())
        {
            return;
        }
        let Some(entity) = host.block_entity(level, event.pos) else {
            return;
        };
        let mut patch = Compound::new();
        patch.insert(KEY_ENERGY.into(), Tag::Int(amount));
        let _ = entity.merge_data(host, patch);
    }
}

/// The charge the item `stack` keeps, 0 when it has none: its
/// `minecraft:custom_data.horizon_energy`, the shape API 307 wrote.
fn item_energy(host: &dyn Host, stack: &ItemStack) -> i32 {
    match ItemResource::of(stack).value(host, &custom_data()) {
        Some(Tag::Compound(data)) => item_energy_of(&data),
        _ => 0,
    }
}

/// The command bodies' argument reader: the block position `/charginggadgets`
/// names.
pub(crate) fn block_pos(argument: Option<&mod_api::CommandArgumentValue>) -> Option<[i32; 3]> {
    match argument? {
        mod_api::CommandArgumentValue::BlockPos(pos) => Some(*pos),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_station_declares_two_slots_and_its_menu() {
        let container = container();
        assert_eq!(container.slots, SLOT_COUNT, "fuel and charge");
        assert_eq!(container.inputs, vec![FUEL_SLOT, CHARGE_SLOT]);
        assert_eq!(container.outputs, vec![FUEL_SLOT, CHARGE_SLOT]);
        assert_eq!(
            container.faces,
            std::array::from_fn(|_| vec![FUEL_SLOT, CHARGE_SLOT]),
            "automation reaches both slots from every face"
        );
        assert_eq!(container.menu.as_deref(), Some(MENU));
    }

    #[test]
    fn the_station_declares_an_insert_only_buffer() {
        let buffer = battery();
        assert_eq!(buffer.capacity, CAPACITY, "the config's chargerMaxPower");
        assert_eq!(
            buffer.max_insert, MAX_TRANSFER,
            "a whole buffer a call, as ChargerEnergyStorage's maxInOut"
        );
        assert_eq!(buffer.max_extract, 0, "nothing may drain it");
        assert_eq!(buffer.insert_faces, [true; 6]);
        assert_eq!(buffer.extract_faces, [false; 6]);
    }

    #[test]
    fn the_station_item_declares_the_block_s_buffer_filled_only() {
        assert_eq!(
            item_buffer(),
            (CAPACITY as u32, MAX_TRANSFER as u32),
            "the item keeps the block's buffer and is filled, never drained"
        );
    }

    #[test]
    fn a_fresh_compound_is_an_empty_station() {
        assert_eq!(read(&Tag::Compound(Compound::new())), Station::new());
    }

    #[test]
    fn the_tick_s_state_reads_back_from_the_compound_it_wrote() {
        let mut saved = Tag::Compound(Compound::new());
        let state = Station {
            energy: 20_000,
            counter: 12,
            max_burn: 32,
        };
        write(&mut saved, &state);
        assert_eq!(read(&saved), state);
        assert_eq!(
            saved.get(KEY_ENERGY),
            Some(&Tag::Int(20_000)),
            "the buffer is the int `energy` the energy capability reads"
        );
        assert_eq!(saved.get(KEY_COUNTER), Some(&Tag::Int(12)));
        assert_eq!(saved.get(KEY_MAX_BURN), Some(&Tag::Int(32)));
    }

    #[test]
    fn the_item_s_charge_is_the_zero_point_one_key() {
        let mut data = Compound::new();
        assert_eq!(item_energy_of(&data), 0, "an uncharged item reads empty");
        with_item_energy(&mut data, 999_000);
        assert_eq!(item_energy_of(&data), 999_000);
        assert_eq!(
            Tag::Compound(data.clone()).get(ITEM_ENERGY_KEY),
            Some(&Tag::Int(999_000)),
            "the retired bundle's `minecraft:custom_data.horizon_energy` int"
        );
        with_item_energy(&mut data, 0);
        assert_eq!(
            item_energy_of(&data),
            0,
            "an emptied item carries nothing, so it stacks with a new one"
        );
        assert!(
            Tag::Compound(data.clone()).get(ITEM_ENERGY_KEY).is_none(),
            "the key is removed, not written as 0"
        );
    }

    #[test]
    fn the_info_line_is_the_one_the_e2e_asserts() {
        let sight = Sight {
            pos: [202, 101, 0],
            energy: 20_000,
            counter: 0,
            max_burn: 0,
            fuel: None,
            charge: Some(Charge {
                item: STATION.to_owned(),
                count: 1,
                buffer: Some((35_000, CAPACITY)),
            }),
        };
        assert_eq!(
            sight.line(527),
            "Charging Station at [202, 101, 0]: 20000/1000000 FE, burning 0/0 steps, fuel empty, \
             charge charginggadgets:charging_station x1 35000/1000000 FE, t=527"
        );
        let with_fuel = Sight {
            fuel: Some(("minecraft:coal".to_owned(), 3)),
            ..sight
        };
        assert!(
            with_fuel
                .line(1119)
                .starts_with("Charging Station at [202, 101, 0]: 20000/1000000 FE, burning 0/0 steps, fuel minecraft:coal x3, charge "),
            "a loaded fuel slot prints its item and count"
        );
    }

    #[test]
    fn the_menu_is_upstream_s_panel_with_its_two_slots() {
        let screen = menu();
        assert_eq!([screen.width, screen.height], [176.0, 166.0]);
        assert_eq!(
            screen.slots,
            vec![Some([65.0, 43.0]), Some([119.0, 43.0])],
            "ChargingStationContainer's fuel and charge slots"
        );
        assert_eq!([screen.inventory_x, screen.inventory_y], [8.0, 84.0]);
        assert!(!screen.own_labels && !screen.own_background);
    }
}
