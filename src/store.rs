use crate::{
    archetype::Archetype, archetype_data_page::ArchetypeDataPage,
    archetype_data_page_view::ArchetypeDataPageView,
    archetypes_container::ArchetypesContainer, entities_container::EntitiesContainer,
    entity_in_archetype::EntityInArchetype, mem_utils, tuple::ComponentsTuple, Entity,
};

const ENTITIES_DEFAULT_CAPACITY: usize = 10;

#[derive(Debug)]
pub struct Store {
    pub(crate) entities_container: EntitiesContainer,
    pub(crate) archetypes_container: ArchetypesContainer,

    entity_in_archetypes: *mut EntityInArchetype,
}

impl Store {
    pub fn new() -> Store {
        Self::with_capacity(ENTITIES_DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Store {
        Store {
            entities_container: EntitiesContainer::new(capacity),
            archetypes_container: ArchetypesContainer::new(),
            entity_in_archetypes: unsafe { mem_utils::alloc_zeroed(capacity) },
        }
    }

    pub const fn data_page_size() -> usize {
        ArchetypeDataPage::PAGE_SIZE_BYTES
    }

    #[inline(always)]
    pub fn entities_capacity(&self) -> usize {
        self.entities_container.capacity()
    }

    pub fn create_entity_with_archetype(&mut self, archetype: &Archetype) -> Entity {
        let creation = self.entities_container.create_entity();
        let entity = creation.entity;
        let entity_in_arch = self.archetypes_container.add_entity(entity.id, archetype);

        if creation.container_was_grow() {
            self.grow_entities_in_archetype(creation.capacity_before);
        }
        *self.get_entity_in_archetype_ref_mut(entity.id) = entity_in_arch;

        entity
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        let entity_in_arch = *self.get_entity_in_archetype_ref(entity.id);
        let swap_remove = self.archetypes_container.swap_remove_entity(entity_in_arch);

        if let Some(swap_remove) = swap_remove {
            let swapped = self.get_entity_in_archetype_ref_mut(swap_remove.id_to_replace);
            swapped.index_in_page = entity_in_arch.index_in_page;
        }

        self.entities_container.destroy_entity(entity)
    }

    #[inline(always)]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities_container.is_alive(entity)
    }

    #[inline(always)]
    pub fn get_components_refs<'a, T>(&'a self, entity: Entity) -> T::RefsTuple<'a>
    where
        T: ComponentsTuple,
    {
        debug_assert!(self.is_alive(entity));

        let entity_in_archetype = self.get_entity_in_archetype_ref(entity.id);
        let page_view = self.get_page_view(entity_in_archetype.page_index as usize);
        page_view.get_components_refs::<T>(entity_in_archetype.index_in_page as usize)
    }

    #[inline(always)]
    pub fn get_components_refs_mut<'a, T>(&'a self, entity: Entity) -> T::MutRefsTuple<'a>
    where
        T: ComponentsTuple,
    {
        debug_assert!(self.is_alive(entity));

        let entity_in_archetype = self.get_entity_in_archetype_ref(entity.id);
        let page_view = self.get_page_view(entity_in_archetype.page_index as usize);
        page_view.get_components_refs_mut::<T>(entity_in_archetype.index_in_page as usize)
    }

    #[inline(always)]
    fn get_page_view<'a>(&self, page_index: usize) -> ArchetypeDataPageView {
        self.archetypes_container.get_page_view(page_index)
    }

    #[inline(always)]
    fn grow_entities_in_archetype(&mut self, old_capacity: usize) {
        self.entity_in_archetypes = unsafe {
            mem_utils::realloc_with_uninit_capacity_zeroing(
                self.entity_in_archetypes,
                old_capacity,
                self.entities_container.capacity(),
            )
        };
    }

    #[inline(always)]
    fn get_entity_in_archetype_ref(&self, id: u32) -> &EntityInArchetype {
        self.entities_container.debug_validate_id_with_panic(id);
        unsafe { &*self.entity_in_archetypes.add(id as usize) }
    }

    #[inline(always)]
    fn get_entity_in_archetype_ref_mut(&mut self, id: u32) -> &mut EntityInArchetype {
        self.entities_container.debug_validate_id_with_panic(id);
        unsafe { &mut *self.entity_in_archetypes.add(id as usize) }
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe {
            mem_utils::dealloc(
                self.entity_in_archetypes,
                self.entities_container.capacity(),
            )
        }
    }
}
