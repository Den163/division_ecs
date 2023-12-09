use crate::{
    archetype::Archetype, archetype_data_page::ArchetypeDataPage,
    archetype_data_page_view::ArchetypeDataPageView,
    archetypes_container::ArchetypesContainer, bitvec_utils,
    component_tuple::ComponentTuple, entities_container::EntitiesContainer,
    entity_in_archetype::EntityInArchetype, mem_utils, ArchetypeBuilder, Entity,
};

const ENTITIES_DEFAULT_CAPACITY: usize = 10;

#[derive(Debug)]
pub struct Store {
    pub(crate) entities_container: EntitiesContainer,
    pub(crate) archetypes_container: ArchetypesContainer,

    entity_to_page: *mut u32,
    entity_to_index_in_page: *mut u32,
    entity_has_archetype_bit_vec: *mut u32,
}

impl Store {
    pub fn new() -> Store {
        Self::with_capacity(ENTITIES_DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Store {
        Store {
            entities_container: EntitiesContainer::new(capacity),
            archetypes_container: ArchetypesContainer::new(),

            entity_has_archetype_bit_vec: unsafe { bitvec_utils::alloc(capacity) },
            entity_to_index_in_page: unsafe { mem_utils::alloc_zeroed(capacity) },
            entity_to_page: unsafe { mem_utils::alloc_zeroed(capacity) },
        }
    }

    pub const fn data_page_size() -> usize {
        ArchetypeDataPage::PAGE_SIZE_BYTES
    }

    #[inline(always)]
    pub fn entities_capacity(&self) -> usize {
        self.entities_container.capacity()
    }

    pub fn create_entity(&mut self) -> Entity {
        let entity = self.register_new_entity();

        unsafe {
            self.disable_archetype_unchecked(entity.id);
        }

        return entity;
    }

    pub fn create_entity_with_archetype(&mut self, archetype: &Archetype) -> Entity {
        let entity = self.register_new_entity();
        let entity_in_arch = self.archetypes_container.add_entity(entity.id, archetype);

        unsafe {
            self.set_page_index_unchecked(entity.id, entity_in_arch.page_index);
            self.set_index_in_page_unchecked(entity.id, entity_in_arch.index_in_page);
            self.enable_archetype_unchecked(entity.id)
        }

        entity
    }

    fn register_new_entity(&mut self) -> Entity {
        let creation = self.entities_container.create_entity();
        if creation.container_was_grow() {
            self.grow_entities_internal(creation.capacity_before);
        }

        creation.entity
    }

    pub fn destroy_entity(&mut self, entity: Entity) {
        // TODO: add error
        let entity_in_arch = unsafe {
            EntityInArchetype {
                page_index: self.get_page_index_unchecked(entity.id),
                index_in_page: self.get_index_in_page_unchecked(entity.id),
            }
        };

        self.swap_remove_internal(entity_in_arch);

        self.entities_container.destroy_entity(entity)
    }

    pub fn add_components<T: ComponentTuple>(&mut self, entity: Entity, components: T) {
        if self.is_alive(entity) == false {
            return;
        }

        // TODO: Optimize, refactor. Avoid using archetype/builder allocations

        let has_archetype = unsafe { self.has_archetype_unchecked(entity.id) };

        let entity_in_archetype = if has_archetype {
            let entity_in_arch = unsafe { self.get_entity_in_archetype(entity.id) };
            let arch_index = self
                .archetypes_container
                .get_archetype_index_by_page(entity_in_arch.page_index as usize);

            let arch = unsafe {
                self.archetypes_container
                    .get_archetypes()
                    .get_unchecked(arch_index)
            };

            if T::is_archetype_include_types(arch) {
                entity_in_arch
            } else {
                let new_arch = ArchetypeBuilder::new()
                    .include_archetype(&arch)
                    .include_components::<T>()
                    .build();

                self.move_entity_to_other_archetype(entity, &new_arch)
            }
        } else {
            let archetype = Archetype::with_components::<T>();
            let entity_in_arch =
                self.archetypes_container.add_entity(entity.id, &archetype);

            unsafe {
                self.set_page_index_unchecked(entity.id, entity_in_arch.page_index);
                self.set_index_in_page_unchecked(entity.id, entity_in_arch.index_in_page);
                self.enable_archetype_unchecked(entity.id);
            }
            entity_in_arch
        };

        unsafe {
            let page_view = self
                .archetypes_container
                .get_page_view_unchecked(entity_in_archetype.page_index as usize);

            let refs = page_view.get_components_refs_mut_unchecked::<T>(
                entity_in_archetype.index_in_page as usize,
            );
            T::assign_to_refs(refs, components);
        };
    }

    pub fn remove_components<T: ComponentTuple + 'static>(&mut self, entity: Entity) {
        if !self.is_valid_entity_with_archetype(entity) {
            return;
        }

        let prev_entity_in_arch = unsafe { self.get_entity_in_archetype(entity.id) };
        let prev_arch_index = self
            .archetypes_container
            .get_archetype_index_by_page(prev_entity_in_arch.page_index as usize);

        let prev_arch = unsafe {
            self.archetypes_container
                .get_archetypes()
                .get_unchecked(prev_arch_index)
        };
        let new_arch = ArchetypeBuilder::new()
            .include_archetype(&prev_arch)
            .exclude_components::<T>()
            .build();

        self.move_entity_to_other_archetype(entity, &new_arch);
    }

    fn move_entity_to_other_archetype(
        &mut self,
        entity: Entity,
        new_arch: &Archetype,
    ) -> EntityInArchetype {
        let prev_entity_in_arch = unsafe { self.get_entity_in_archetype(entity.id) };
        let prev_arch_index = self
            .archetypes_container
            .get_archetype_index_by_page(prev_entity_in_arch.page_index as usize);

        let entity_in_arch = self.archetypes_container.move_entity_to_other_archetype(
            entity.id,
            prev_entity_in_arch,
            prev_arch_index,
            &new_arch,
        );
        unsafe {
            self.set_page_index_unchecked(entity.id, entity_in_arch.page_index);
            self.set_index_in_page_unchecked(entity.id, entity_in_arch.index_in_page);
        }

        self.swap_remove_internal(prev_entity_in_arch);

        entity_in_arch
    }

    fn swap_remove_internal(&mut self, entity_in_archetype: EntityInArchetype) {
        let swap_remove = self
            .archetypes_container
            .swap_remove_entity(entity_in_archetype);

        if let Some(swap_remove) = swap_remove {
            unsafe {
                self.set_index_in_page_unchecked(
                    swap_remove.id_to_replace,
                    entity_in_archetype.index_in_page,
                );
            }
        }
    }

    #[inline(always)]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entities_container.is_alive(entity)
    }

    #[inline(always)]
    pub fn get_components_refs<'a, T: ComponentTuple>(
        &'a self,
        entity: Entity,
    ) -> Option<T::RefTuple<'a>> {
        if self.is_valid_entity_with_archetype(entity) {
            let (page_view, index_in_page) = unsafe { self.get_page_info(entity.id) };
            page_view.get_components_refs::<T>(index_in_page)
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn get_components_refs_mut<'a, T: ComponentTuple>(
        &'a mut self,
        entity: Entity,
    ) -> Option<T::MutRefTuple<'a>> {
        if self.is_valid_entity_with_archetype(entity) {
            let (page_view, index_in_page) = unsafe { self.get_page_info(entity.id) };
            page_view.get_components_refs_mut::<T>(index_in_page)
        } else {
            None
        }
    }

    unsafe fn get_page_info(&self, entity_id: u32) -> (ArchetypeDataPageView, usize) {
        (
            self.get_page_view_unchecked(entity_id),
            self.get_index_in_page_unchecked(entity_id) as usize,
        )
    }

    unsafe fn get_entity_in_archetype(&self, entity_id: u32) -> EntityInArchetype {
        EntityInArchetype {
            page_index: self.get_page_index_unchecked(entity_id),
            index_in_page: self.get_index_in_page_unchecked(entity_id),
        }
    }

    #[inline(always)]
    unsafe fn get_page_view_unchecked(&self, entity_id: u32) -> ArchetypeDataPageView {
        let page_index = self.get_page_index_unchecked(entity_id) as usize;
        self.archetypes_container
            .get_page_view_unchecked(page_index)
    }

    #[inline(always)]
    pub fn get_entity_archetype(&self, entity: Entity) -> Option<&Archetype> {
        if self.is_valid_entity_with_archetype(entity) == false {
            return None;
        }

        let page_index = unsafe { *self.entity_to_page.add(entity.id as usize) as usize };

        Some(self.archetypes_container.get_archetype_by_page(page_index))
    }

    #[inline]
    pub fn is_valid_entity_with_archetype(&self, entity: Entity) -> bool {
        self.is_alive(entity) && unsafe { self.has_archetype_unchecked(entity.id) }
    }

    #[inline(always)]
    fn grow_entities_internal(&mut self, old_capacity: usize) {
        let new_capacity = self.entities_capacity();
        unsafe {
            self.entity_to_page =
                mem_utils::realloc(self.entity_to_page, old_capacity, new_capacity);

            self.entity_to_index_in_page = mem_utils::realloc(
                self.entity_to_index_in_page,
                old_capacity,
                new_capacity,
            );

            self.entity_has_archetype_bit_vec = bitvec_utils::realloc(
                self.entity_has_archetype_bit_vec,
                old_capacity,
                new_capacity,
            );
        };
    }

    #[inline(always)]
    pub(crate) unsafe fn get_page_index_unchecked(&self, entity_id: u32) -> u32 {
        *self.entity_to_page.add(entity_id as usize)
    }

    #[inline(always)]
    pub(crate) unsafe fn get_index_in_page_unchecked(&self, entity_id: u32) -> u32 {
        *self.entity_to_index_in_page.add(entity_id as usize)
    }

    #[inline(always)]
    pub(crate) unsafe fn entity_to_index_in_page_ptr(&self) -> *const u32 {
        self.entity_to_index_in_page
    }

    #[inline(always)]
    pub(crate) unsafe fn has_archetype_unchecked(&self, entity_id: u32) -> bool {
        bitvec_utils::is_bit_on(self.entity_has_archetype_bit_vec, entity_id as usize)
    }

    #[inline(always)]
    unsafe fn set_page_index_unchecked(&mut self, entity_id: u32, page_index: u32) {
        *self.entity_to_page.add(entity_id as usize) = page_index
    }

    #[inline(always)]
    unsafe fn set_index_in_page_unchecked(&mut self, entity_id: u32, index_in_page: u32) {
        *self.entity_to_index_in_page.add(entity_id as usize) = index_in_page
    }

    #[inline(always)]
    unsafe fn enable_archetype_unchecked(&mut self, entity_id: u32) {
        bitvec_utils::set_bit_on(self.entity_has_archetype_bit_vec, entity_id as usize)
    }

    #[inline(always)]
    unsafe fn disable_archetype_unchecked(&mut self, entity_id: u32) {
        bitvec_utils::set_bit_off(self.entity_has_archetype_bit_vec, entity_id as usize)
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe {
            let capacity = self.entities_container.capacity();

            mem_utils::dealloc(self.entity_to_index_in_page, capacity);
            mem_utils::dealloc(self.entity_to_page, capacity);
            bitvec_utils::dealloc(self.entity_has_archetype_bit_vec, capacity);
        }
    }
}
