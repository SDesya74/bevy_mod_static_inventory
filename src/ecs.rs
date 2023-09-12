use std::{any::TypeId, cell::UnsafeCell, marker::PhantomData};

use bevy::{
    ecs::{
        archetype::{Archetype, ArchetypeComponentId},
        component::{ComponentId, Tick},
        query::{Access, FilteredAccess, ReadOnlyWorldQuery, WorldQuery},
        storage::{Table, TableRow},
        world::unsafe_world_cell::UnsafeWorldCell,
    },
    prelude::{Entity, World},
    ptr::ThinSlicePtr,
};

use super::{item::Item, key::Key, Inventory};

pub struct AnySlot; // TODO: Add AnyNumberedSlot, AnyCustomSlot

pub struct HasInInventory<TItem: Item, TSlot = AnySlot> {
    _pd: PhantomData<(TItem, TSlot)>,
}

#[derive(Clone, Copy)]
pub struct HasInInventoryFetch<'w> {
    table_components: Option<ThinSlicePtr<'w, UnsafeCell<Inventory>>>,
    slot: Option<Key>,
}

pub struct HasInInventoryState {
    inventory_component_id: ComponentId,
    slot: Option<Key>,
}

/// SAFETY: `Self` is the same as `Self::ReadOnly`
unsafe impl<TItem: Item, TSlot: 'static> WorldQuery for HasInInventory<TItem, TSlot> {
    type Fetch<'w> = HasInInventoryFetch<'w>;
    type Item<'w> = bool;
    type ReadOnly = Self;
    type State = HasInInventoryState;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wshort>) -> Self::Item<'wshort> {
        item
    }

    const IS_DENSE: bool = true;
    const IS_ARCHETYPAL: bool = true;

    #[inline]
    unsafe fn init_fetch<'w>(
        _world: UnsafeWorldCell<'w>,
        state: &Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Fetch<'w> {
        Self::Fetch {
            table_components: None,
            slot: state.slot,
        }
    }

    #[inline]
    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        _archetype: &'w Archetype,
        table: &'w Table,
    ) {
        if Self::IS_DENSE {
            Self::set_table(fetch, state, table);
        }
    }

    #[inline]
    unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w Table) {
        let components = table
            .get_column(state.inventory_component_id)
            .unwrap()
            .get_data_slice()
            .into();
        fetch.table_components = Some(components);
    }

    #[inline(always)]
    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        _entity: Entity,
        table_row: TableRow,
    ) -> Self::Item<'w> {
        let components = fetch.table_components.unwrap();
        let inv = components.get(table_row.index()).get().as_ref().unwrap();

        fetch
            .slot
            .and_then(|slot| inv.get_slot(slot).ok())
            .map(|slot| slot.contains::<TItem>())
            .unwrap_or_else(|| inv.has::<TItem>())
    }

    #[inline(always)]
    unsafe fn filter_fetch(
        fetch: &mut Self::Fetch<'_>,
        _entity: Entity,
        table_row: TableRow,
    ) -> bool {
        let components = fetch.table_components.unwrap();
        let inv = components.get(table_row.index()).get().as_ref().unwrap();

        fetch
            .slot
            .and_then(|slot| inv.get_slot(slot).ok())
            .map(|slot| slot.contains::<TItem>())
            .unwrap_or_else(|| inv.has::<TItem>())
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        assert!(
            !access.access().has_write(state.inventory_component_id),
            "&{} conflicts with a previous access in this query. Shared access cannot coincide with exclusive access.",
                std::any::type_name::<Inventory>(),
        );
        access.add_read(state.inventory_component_id);
    }

    fn update_archetype_component_access(
        state: &Self::State,
        archetype: &Archetype,
        access: &mut Access<ArchetypeComponentId>,
    ) {
        if let Some(archetype_component_id) =
            archetype.get_archetype_component_id(state.inventory_component_id)
        {
            access.add_read(archetype_component_id);
        }
    }

    fn init_state(world: &mut World) -> Self::State {
        Self::State {
            inventory_component_id: world.init_component::<Inventory>(),
            slot: (TypeId::of::<AnySlot>() != TypeId::of::<TSlot>())
                .then_some(Key::Type(TypeId::of::<TSlot>())),
        }
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        set_contains_id(state.inventory_component_id)
    }

    unsafe fn clone_fetch<'w>(fetch: &Self::Fetch<'w>) -> Self::Fetch<'w> {
        *fetch
    }
}

/// SAFETY: access is read only
unsafe impl<TItem: Item, TSlot: 'static> ReadOnlyWorldQuery for HasInInventory<TItem, TSlot> {}
