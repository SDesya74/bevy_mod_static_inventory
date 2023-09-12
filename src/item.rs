use std::any::Any;

use dyn_clone::DynClone;
use dyn_eq::DynEq;
use intertrait::CastFrom;

pub type BoxedItem = Box<dyn Item>;

// TODO: Remove Debug
pub trait Item:
    CastFrom + std::fmt::Debug + DynEq + DynClone + Any + Send + Sync + 'static
{
    /// Max amount of items in a single stack, checked only in inventory.add for now
    fn max_in_stack(&self) -> usize {
        4
    }
}

dyn_clone::clone_trait_object!(Item);
dyn_eq::eq_trait_object!(Item);
