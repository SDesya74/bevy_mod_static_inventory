pub mod ecs;
pub mod item;
pub mod key;
pub mod stack;

use bevy::{prelude::*, utils::label::DynEq};
use std::{any::TypeId, collections::HashMap};

pub use ecs::{AnySlot, HasInInventory};
pub use item::{BoxedItem, Item};
pub use key::Key;
pub use stack::{BoxedItemStack, ItemStack};

// pub struct InventoryPlugin;

// impl Plugin for InventoryPlugin {
//     fn build(&self, _app: &mut App) {
//         // TODO: Implement Reflect for everything somehow
//     }
// }

#[derive(Debug)]
pub enum InvError {
    NotEnoughSpace { left: BoxedItemStack },
    NoSuchSlot(Key),
}

#[derive(Component, Debug, Default)]
pub struct Inventory {
    slot_count: usize,
    slots: HashMap<Key, Slot>,
}

impl Inventory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn slot_count(&self) -> usize {
        self.slot_count
    }

    pub fn with_slot_count(mut self, count: usize) -> Self {
        for slot in 0..self.slot_count {
            self.slots.remove(&Key::Number(slot));
        }

        self.slot_count = count;

        for slot in 0..self.slot_count {
            self.slots.insert(Key::Number(slot), Slot::Empty);
        }

        self
    }

    pub fn with_custom_slot<T: 'static>(mut self, _slot: T) -> Self {
        self.slots.insert(Key::Type(TypeId::of::<T>()), Slot::Empty);
        self
    }

    pub fn items<T: Item>(&self) -> impl Iterator<Item = &BoxedItemStack> {
        self.items_by_type_id(TypeId::of::<T>())
    }

    pub fn items_by_type_id(&self, tid: TypeId) -> impl Iterator<Item = &BoxedItemStack> {
        self.slots
            .iter()
            .filter_map(|(_, slot)| match slot {
                Slot::Empty => None,
                Slot::Occupied(stack) => Some(stack),
            })
            .filter(move |stack| (*stack.item).as_any().type_id() == tid)
    }

    pub fn items_mut<T: Item>(&mut self) -> impl Iterator<Item = &mut BoxedItemStack> {
        self.items_mut_by_type_id(TypeId::of::<T>())
    }

    pub fn items_mut_by_type_id(
        &mut self,
        tid: TypeId,
    ) -> impl Iterator<Item = &mut BoxedItemStack> {
        self.slots
            .iter_mut()
            .filter_map(|(_, slot)| match slot {
                Slot::Empty => None,
                Slot::Occupied(stack) => Some(stack),
            })
            .filter(move |stack| (*stack.item).as_any().type_id() == tid)
    }

    pub fn custom_slots(&self) -> impl Iterator<Item = &Slot> {
        self.slots
            .iter()
            .filter(|(key, _)| key.is_type())
            .map(|(_, slot)| slot)
    }

    pub fn custom_slots_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        self.slots
            .iter_mut()
            .filter(|(key, _)| key.is_type())
            .map(|(_, slot)| slot)
    }

    pub fn numbered_slots(&self) -> impl Iterator<Item = &Slot> {
        self.slots
            .iter()
            .filter(|(key, _)| key.is_numbered())
            .map(|(_, slot)| slot)
    }

    pub fn numbered_slots_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        self.slots
            .iter_mut()
            .filter(|(key, _)| key.is_numbered())
            .map(|(_, slot)| slot)
    }

    pub fn has<T: Item>(&self) -> bool {
        self.items::<T>().next().is_some()
    }

    pub fn has_item<T: Item + Eq>(&self, item: T) -> bool {
        self.items::<T>()
            .any(|stack| *stack.typed::<T>().unwrap() == item)
    }

    pub fn count<T: Item>(&self) -> usize {
        self.items::<T>().map(|e| e.amount).sum()
    }

    pub fn count_item<T: Item + Eq>(&self, item: T) -> usize {
        self.items::<T>()
            .filter(|stack| *stack.typed::<T>().unwrap() == item)
            .map(|e| e.amount)
            .sum()
    }

    pub fn add(&mut self, stack: impl Into<BoxedItemStack>) -> Result<(), InvError> {
        let mut stack: BoxedItemStack = stack.into();
        let stack_item_type_id = (*stack.item).as_any().type_id();

        for existing_stack in self
            .items_mut_by_type_id(stack_item_type_id)
            .collect::<Vec<_>>()
        {
            if stack.is_empty() {
                return Ok(());
            }

            if !existing_stack.item.dyn_eq(&stack.item) {
                continue;
            }

            let last = existing_stack.amount;
            let max = existing_stack.item.max_in_stack();
            if existing_stack.amount < max {
                existing_stack.amount = (existing_stack.amount + stack.amount).min(max);
                stack.amount -= existing_stack.amount - last;
            }
        }

        self.add_item_to_first_empty_slot(stack)
    }

    pub fn add_item_to_first_empty_slot(
        &mut self,
        stack: impl Into<BoxedItemStack>,
    ) -> Result<(), InvError> {
        let stack = stack.into();

        if stack.is_empty() {
            return Ok(());
        }

        if let Some(empty_slot) = self.numbered_slots_mut().find(|slot| slot.is_empty()) {
            *empty_slot = Slot::Occupied(stack);
            Ok(())
        } else {
            Err(InvError::NotEnoughSpace { left: stack })
        }
    }

    pub fn insert(
        &mut self,
        slot: impl Into<Key>,
        stack: impl Into<BoxedItemStack>,
    ) -> Result<(), InvError> {
        let slot = self.get_slot_mut(slot)?;

        *slot = Slot::Occupied(stack.into());
        Ok(())
    }

    pub fn get_slot(&self, slot: impl Into<Key>) -> Result<&Slot, InvError> {
        let key = slot.into();
        self.slots.get(&key).ok_or(InvError::NoSuchSlot(key))
    }

    pub fn get_slot_mut(&mut self, slot: impl Into<Key>) -> Result<&mut Slot, InvError> {
        let key = slot.into();
        self.slots.get_mut(&key).ok_or(InvError::NoSuchSlot(key))
    }
}

// TODO: Implement proper debug
// impl std::fmt::Debug for Inventory {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut numbered = self.numbered_slots().enumerate().collect::<Vec<_>>();
//         numbered.sort_by_key(|e| e.0);
//         f.debug_struct("Inventory")
//             .field(
//                 "numbered_slots",
//                 numbered.into_iter().collect::<HashMap<_, _>>(),
//             )
//             .field(
//                 "custom_slots",
//                 &self.custom_slots().enumerate().collect::<Vec<_>>(),
//             )
//             .finish()
//     }
// }

#[derive(Debug)]
pub enum Slot {
    Empty,
    Occupied(BoxedItemStack),
}

impl Slot {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn contains_type(&self, type_id: TypeId) -> bool {
        match self {
            Slot::Empty => false,
            Slot::Occupied(stack) => stack.is_type(type_id),
        }
    }

    pub fn contains<TItem: Item>(&self) -> bool {
        match self {
            Slot::Empty => false,
            Slot::Occupied(stack) => stack.is::<TItem>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::stack::ItemStack;

    use super::*;

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct Stick;
    impl Item for Stick {}

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct Apple;
    impl Item for Apple {}

    #[test]
    fn test_inventory() {
        struct CustomSlot;
        impl From<CustomSlot> for Key {
            fn from(_: CustomSlot) -> Self {
                Key::Type(TypeId::of::<CustomSlot>())
            }
        }

        let mut inv = Inventory::new()
            .with_slot_count(1)
            .with_custom_slot(CustomSlot);

        assert!(inv.add(ItemStack::new(Stick, 5)).is_ok()); // TODO: Bound stack size
        assert!(inv.has::<Stick>());
        assert_eq!(inv.count::<Stick>(), 5);

        let apples = ItemStack::new(Apple, 4);

        assert!(matches!(
            dbg!(inv.add(apples)),
            Err(InvError::NotEnoughSpace { .. }) // TODO: Check that left is equal to added
        ));

        inv.insert(CustomSlot, Apple).unwrap();
        assert!(inv.has::<Apple>());
        assert_eq!(inv.count::<Apple>(), 1);

        // inv.pop_all::<Stick>(); // ItemStack<Stick>(5);

        // inv.first_entry::<Stick>();

        // inv.remove::<Stick>(20); // Ok(ItemStack<Stick>(20))

        // inv.slot(0); // -> Slot::Empty / Slot::Occupied

        // inv.slot(HelmetSlot); // Slot

        // inv.slot(ArmorSlot::Head);

        // inv.slots(0..=3) // impl Iterator<Item = Slot>
    }

    #[test]
    fn test_enum_item() {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub enum Crystal {
            Red,
            Green,
        }
        impl Item for Crystal {}

        let mut inv = Inventory::new().with_slot_count(2);

        inv.add(Crystal::Red).unwrap();
        inv.add(Crystal::Red).unwrap();
        inv.add(Crystal::Green).unwrap();

        assert!(inv.has::<Crystal>());
        assert!(inv.has_item(Crystal::Red));
        assert!(inv.has_item(Crystal::Green));

        assert_eq!(inv.count::<Crystal>(), 3);
        assert_eq!(inv.count_item(Crystal::Red), 2);
        assert_eq!(inv.count_item(Crystal::Green), 1);
    }

    #[test]
    fn test_custom_slot() {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub struct Helmet;
        impl Item for Helmet {}

        pub struct HelmetSlot;
        impl From<HelmetSlot> for Key {
            fn from(_value: HelmetSlot) -> Self {
                Key::Type(TypeId::of::<HelmetSlot>())
            }
        }

        let mut inv = Inventory::new().with_custom_slot(HelmetSlot);

        assert!(matches!(
            inv.add(Helmet),
            Err(InvError::NotEnoughSpace { .. })
        ));

        inv.insert(HelmetSlot, Helmet).unwrap();
    }

    #[test]
    fn test_add_boxed_item() {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub enum Crystal {
            Red,
            Green,
        }

        impl Item for Crystal {}

        let boxed_item: BoxedItemStack = ItemStack::new(Crystal::Red, 1).into();
        dbg!(
            TypeId::of::<Crystal>(),
            (*boxed_item.item).as_any().type_id(),
            (*boxed_item.item).as_any().is::<Crystal>()
        );

        let mut inv = Inventory::new().with_slot_count(2);

        inv.add(Crystal::Red).unwrap();
        inv.add(Crystal::Red).unwrap();
        inv.add(Crystal::Green).unwrap();

        assert!(inv.has::<Crystal>());
        assert!(inv.has_item(Crystal::Red));
        assert!(inv.has_item(Crystal::Green));

        assert_eq!(inv.count::<Crystal>(), 3);
        assert_eq!(inv.count_item(Crystal::Red), 2);
        assert_eq!(inv.count_item(Crystal::Green), 1);
    }
}
