# Static Inventory

Proof of concept of static inventory for [Bevy](https://bevyengine.org/). A static inventory means that each item type is a `struct` (or `enum`) with a specific set of traits. The advantages of this approach are excellent editor support and validity checking of some actions at compile time, e.g. you can't put sticks and apples in the same item stack.

It also opens up possibilities for partial support for inventory requests in ECS. For example, it is possible to call the system only for entities that have an apple in their inventory using Bevy's [query filters](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.Query.html#query-filtering).

This also allows you to add parameters to each individual item, such as color or durability for tools.

The current (not for long, I hope) drawbacks of this method, however, is the inability to create items in runtime.

# Installation

```bash
cargo add bevy_mod_static_inventory
```

# Usage

```rust
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


```

# Progress

- [x] Typed and boxed ItemStacks
- [x] Numbered and custom slots
- [ ] Derive macros for defining `Item` and `Key` for custom slots
- [ ] Advanced query filters (added/removed item etc)
- [ ] Full documentation
- [ ] Support for dynamic item registering (for modding)
- [ ] Remove dependency on `TypeId` in all places as it is unstable between builds
- [ ] Support for enum slots (e.g. `ArmorSlot` containing `Head`, `Body`, `Legs` variants)
- [ ] Support downcasting for more concrete traits, that have `Item` as supertrait (now it is possible only in runtime with `intertrait` and `linkme`)
