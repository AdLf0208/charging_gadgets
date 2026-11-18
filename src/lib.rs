//! Charging Gadgets for Horizon — slice 1, the Charging Station.
//!
//! One flat SDK package (mod API 0.2): the carried block, its item, models,
//! textures, lang, loot table, recipe, advancement and tags are the `assets/`
//! and `data/` here, and the behaviour is the one `impl Hooks` below. The rules
//! themselves live in [`runtime`], byte-identical to the retired `.hnmod`
//! bundle's `runtime.rs`; [`station`] is the glue.
//!
//! What the retired bundle expressed as twelve injected helpers is here three
//! shapes: the block's block entity, its tick and its screen are a **Part**
//! (`station::StationPart`, attached to the type declared for the block), its
//! two slots, its FE buffer, the station item's own buffer and its tooltip and
//! placement are **declarations** and a second Part on the item key, and the
//! command is the typed `register_commands`/`run_mod_command` pair. Its
//! embedded-content helper has no successor: one `.hmod` carries the content
//! and the code, which is what that helper existed to work around.
use mod_api::{
    self, Command, CommandArgumentValue, CommandSource, Flow, Hooks, RegisterCommandsEvent,
    Registrar, RegistrationContext, RunModCommandEvent, ServerHost, Text,
};

use crate::runtime::CAPACITY;

mod station;

pub mod runtime;

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Declare the station: one block with a block entity, two slots and its
    /// menu, an insert-only FE buffer, and one item carrying its own charge.
    ///
    /// The retired bundle wrote the buffer into the `block_entity_energy` hook's
    /// shared output and had to leave a foreign block's entry alone; a
    /// declaration is keyed by the block, so that hazard cannot recur.
    fn register(&self, registrar: &mut dyn Registrar, context: &RegistrationContext) {
        let block = registrar.block_key(station::STATION);
        registrar.block_container(&block, station::container());
        registrar.block_energy(&block, station::battery());
        let kind = station::declare(registrar);
        if let Err(error) = registrar.block_entity_part(&kind, station::StationPart) {
            context.refuse(&error.to_string());
        }
        let item = registrar.item_key(station::STATION);
        registrar.item_part(item, station::StationItemPart);
        registrar.item_capability(&mod_api::ITEM_ENERGY, station::STATION, |_, place| {
            Some(Box::new(station::StationEnergy::new(place.clone())))
        });
    }

    /// `/charginggadgets info|energy|item|held`: the harness controls the
    /// in-game check uses. Upstream has none of its own (its config and GUI are
    /// the only controls); precedent `/powah info|set`, `/mininggadgets energy`.
    fn register_commands(&self, _host: &mut dyn ServerHost, event: &mut RegisterCommandsEvent<'_>) {
        event
            .commands
            .push(Command::new("charginggadgets info <pos:block_pos>"));
        event.commands.push(Command::op(
            "charginggadgets energy <pos:block_pos> <amount:integer>",
            2,
        ));
        event.commands.push(Command::op(
            "charginggadgets item <pos:block_pos> <amount:integer>",
            2,
        ));
        event
            .commands
            .push(Command::op("charginggadgets held <amount:integer>", 2));
    }

    /// The command bodies: the arguments are parsed and resolved by the
    /// server's dispatcher, so this only reads the station and answers.
    fn run_mod_command(
        &self,
        host: &mut dyn ServerHost,
        event: &mut RunModCommandEvent<'_>,
    ) -> Flow {
        let literals: Vec<&str> = event.command.literals.iter().map(String::as_str).collect();
        let pos = station::block_pos(event.command.argument("pos"));
        let amount = match event.command.argument("amount") {
            Some(CommandArgumentValue::Integer(amount)) => Some(*amount),
            _ => None,
        };
        let line = match (literals.as_slice(), pos, amount) {
            (["charginggadgets", "info"], Some(pos), _) => station::info_line(host, pos)
                .unwrap_or_else(|| format!("{pos:?} is not a charging station.")),
            (["charginggadgets", "energy"], Some(pos), Some(amount)) => {
                let amount = amount.clamp(0, CAPACITY);
                match station::set_energy(host, pos, amount) {
                    Some(after) => {
                        format!("Charging Station at {pos:?}: {after}/{} FE.", CAPACITY)
                    }
                    None => format!("{pos:?} is not a charging station."),
                }
            }
            (["charginggadgets", "item"], Some(pos), Some(amount)) => {
                let target = amount.clamp(0, CAPACITY);
                match station::set_item_energy(host, pos, target) {
                    Some(energy) => format!(
                        "Charging Station at {pos:?}: the charge slot holds {energy}/{} FE.",
                        CAPACITY
                    ),
                    None => format!("{pos:?} has no chargeable item in its charge slot."),
                }
            }
            (["charginggadgets", "held"], _, Some(amount)) => {
                let target = amount.clamp(0, CAPACITY);
                let player = match event.source {
                    CommandSource::Player { player, .. } => Some(player.uuid()),
                    _ => None,
                };
                match player.and_then(|player| station::set_held_energy(host, player, target)) {
                    Some(energy) => {
                        format!("Charging Station held item: {energy}/{} FE.", CAPACITY)
                    }
                    None => "Hold a Charging Station item first.".to_owned(),
                }
            }
            _ => return Flow::Continue,
        };
        let _ = host.commands().feedback(Text::literal(line));
        Flow::Cancel
    }
}
