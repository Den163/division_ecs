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

    entity_to_archetype: *mut EntityInArchetype,
}

impl Store {
    pub fn new() -> Store {
        Self::with_capacity(ENTITIES_DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Store {
        Store {
            entities_container: EntitiesContainer::new(capacity),
            archetypes_container: ArchetypesContainer::new(),
            entity_to_archetype: unsafe { mem_utils::alloc_zeroed(capacity) },
        }
    }

    pub const fn data_page_size() -> usize {
        ArchetypeDataPage::PAGE_SIZE_BYTES
    }

    #[inline(always)]
    pub fn entities_capacity(&self) -> usize {
        self.entities_container.capacity()
    }

    #[inline(always)]
    pub fn create_entity(&mut self, archetype: &Archetype) -> Entity {
        let will_entities_grow = self.entities_container.will_grow_with_entity_create();
        let old_capacity = self.entities_container.capacity();
        let entity = self.entities_container.create_entity();
        let entity_in_arch = self.archetypes_container.add_entity(entity.id, archetype);

        if will_entities_grow {
            self.grow_entities_in_archetype(old_capacity);
        }
        self.set_entity_in_archetype(entity.id, entity_in_arch);

        entity
    }

    #[inline(always)]
    pub fn destroy_entity(&mut self, entity: Entity) {
        let entity_in_arch = self.get_entity_in_archetype(entity.id);
        self.archetypes_container
            .remove_entity(entity.id, entity_in_arch);
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
        assert!(self.is_alive(entity));

        let page_view = self.get_page_view(entity);
        let entity_index = page_view.page.get_entity_index_by_id(entity.id);
        page_view.get_components_refs::<T>(entity_index)
    }

    #[inline(always)]
    pub fn get_components_refs_mut<'a, T>(&'a self, entity: Entity) -> T::MutRefsTuple<'a>
    where
        T: ComponentsTuple,
    {
        assert!(self.is_alive(entity));

        let page_view = self.get_page_view(entity);
        let entity_index = page_view.page.get_entity_index_by_id(entity.id);
        page_view.get_components_refs_mut::<T>(entity_index)
    }

    #[inline(always)]
    fn get_page_view<'a>(&self, entity: Entity) -> ArchetypeDataPageView {
        self.archetypes_container
            .get_page_view(self.get_entity_in_archetype(entity.id).page_index)
    }

    #[inline(always)]
    fn grow_entities_in_archetype(&mut self, old_capacity: usize) {
        self.entity_to_archetype = unsafe {
            mem_utils::realloc_with_uninit_capacity_zeroing(
                self.entity_to_archetype,
                old_capacity,
                self.entities_container.capacity(),
            )
        };
    }

    #[inline(always)]
    fn get_entity_in_archetype(&self, id: u32) -> EntityInArchetype {
        self.entities_container.validate_id_with_panic(id);
        unsafe { *self.entity_to_archetype.add(id as usize) }
    }

    #[inline(always)]
    fn set_entity_in_archetype(&self, id: u32, entity_in_archetype: EntityInArchetype) {
        self.entities_container.validate_id_with_panic(id);
        unsafe {
            *self.entity_to_archetype.add(id as usize) = entity_in_archetype;
        }
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe {
            mem_utils::dealloc(
                self.entity_to_archetype,
                self.entities_container.capacity(),
            )
        }
    }
}
