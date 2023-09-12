use std::{any::TypeId, time::Duration};

use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use bevy_mod_static_inventory::{HasInInventory, Inventory, Item, Key};

fn main() {
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0))),
        )
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (log_when_apple_in_inventory, log_when_stick_in_hand_slot),
        )
        .run();
}

pub struct HandSlot;

// TODO: Create derive macro
impl From<HandSlot> for Key {
    fn from(_: HandSlot) -> Self {
        Key::Type(TypeId::of::<HandSlot>())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Apple;

// TODO: Create derive macro
impl Item for Apple {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stick;

impl Item for Stick {}

fn setup(mut commands: Commands) {
    let mut inv = Inventory::new().with_slot_count(4);

    inv.add(Apple).unwrap(); // add 1 apple
    inv.add((Apple, 2)).unwrap(); // add 2 apples
    inv.add((2, Apple)).unwrap(); // you also can swap count and item

    commands.spawn(inv);
    println!("Spawned entity with apples in inventory");

    let mut inv = Inventory::new().with_custom_slot(HandSlot);

    inv.insert(HandSlot, Stick).unwrap();

    commands.spawn(inv);
    println!("Spawned entity with apples in inventory");
}

fn log_when_apple_in_inventory(entities_with_apples: Query<DebugName, HasInInventory<Apple>>) {
    for e in entities_with_apples.iter() {
        println!("{:?} has Apple in inventory", e);
    }
}

fn log_when_stick_in_hand_slot(
    entities_with_apples: Query<DebugName, HasInInventory<Stick, HandSlot>>,
) {
    for e in entities_with_apples.iter() {
        println!("{:?} has Stick in hand slot", e);
    }
}
