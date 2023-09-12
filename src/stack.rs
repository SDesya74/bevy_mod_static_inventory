use std::any::TypeId;

use super::item::{BoxedItem, Item};

#[derive(Debug)]
pub struct ItemStack<T: Item> {
    pub item: T,
    pub amount: usize,
}

impl<T: Item> ItemStack<T> {
    pub fn new(item: T, amount: usize) -> Self {
        Self { item, amount }
    }

    pub fn is_empty(&self) -> bool {
        self.amount < 1
    }
}

impl<T: Item> From<T> for ItemStack<T> {
    fn from(item: T) -> Self {
        Self::new(item, 1)
    }
}

impl<T: Item> From<(usize, T)> for ItemStack<T> {
    fn from((amount, item): (usize, T)) -> Self {
        Self::new(item, amount)
    }
}

impl<T: Item> From<(T, usize)> for ItemStack<T> {
    fn from((item, amount): (T, usize)) -> Self {
        Self::new(item, amount)
    }
}

#[derive(Debug, Clone)]
pub struct BoxedItemStack {
    pub item: BoxedItem,
    pub amount: usize,
}

impl BoxedItemStack {
    pub fn new<T: Item>(item: T, amount: usize) -> Self {
        Self {
            item: Box::new(item),
            amount,
        }
    }

    pub fn typed<T: Item>(&self) -> Option<&T> {
        self.item.as_any().downcast_ref::<T>()
    }

    pub fn is<T: Item>(&self) -> bool {
        self.item.as_any().is::<T>()
    }

    pub fn is_type(&self, type_id: TypeId) -> bool {
        self.item.as_any().type_id() == type_id
    }

    pub fn is_empty(&self) -> bool {
        self.amount < 1
    }
}

impl<T: Item> From<T> for BoxedItemStack {
    fn from(item: T) -> Self {
        Self::new(item, 1)
    }
}

impl<T: Item> From<ItemStack<T>> for BoxedItemStack {
    fn from(stack: ItemStack<T>) -> Self {
        Self::new(stack.item, stack.amount)
    }
}

impl<T: Item> From<(usize, T)> for BoxedItemStack {
    fn from((amount, item): (usize, T)) -> Self {
        Self::new(item, amount)
    }
}

impl<T: Item> From<(T, usize)> for BoxedItemStack {
    fn from((item, amount): (T, usize)) -> Self {
        Self::new(item, amount)
    }
}
