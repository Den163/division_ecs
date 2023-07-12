use crate::{
    archetype::Archetype, archetype_data_page::ArchetypeDataPage,
    archetypes_container::ArchetypesContainer, EntitiesContainer, Entity,
};

const ENTITIES_DEFAULT_CAPACITY: usize = 10;

#[derive(Debug)]
pub struct Registry {
    pub(crate) entities_container: EntitiesContainer,
    pub(crate) archetypes_container: ArchetypesContainer,
}

impl Registry {
    pub fn new() -> Registry {
        Self::with_capacity(ENTITIES_DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Registry {
        Registry {
            entities_container: EntitiesContainer::new(capacity),
            archetypes_container: ArchetypesContainer::new(),
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
        let entity = self.entities_container.create_entity();
        let entity_in_arch = self.archetypes_container.add_entity(entity.id, archetype);
        self.entities_container
            .set_entity_in_archetype(entity.id, entity_in_arch);

        entity
    }

    #[inline(always)]
    pub fn destroy_entity(&mut self, entity: Entity) {
        let entity_in_arch = self.entities_container.get_entity_in_archetype(entity.id);
        self.archetypes_container
            .remove_entity(entity.id, entity_in_arch);
        self.entities_container.destroy_entity(entity)
    }

    #[inline(always)]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities_container.is_alive(entity)
    }

    #[inline(always)]
    pub fn get_component_ref<T: 'static>(&self, entity: Entity) -> &T {
        assert!(self.is_alive(entity));

        self.archetypes_container.get_component_ref_by_entity_id(
            entity.id,
            self.entities_container.get_entity_in_archetype(entity.id),
        )
    }

    #[inline(always)]
    pub fn get_component_ref_mut<T: 'static>(&mut self, entity: Entity) -> &mut T {
        assert!(self.is_alive(entity));

        self.archetypes_container
            .get_component_ref_mut_by_entity_id(
                entity.id,
                self.entities_container.get_entity_in_archetype(entity.id),
            )
    }
}
