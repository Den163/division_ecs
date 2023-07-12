use std::{mem::size_of, any::TypeId};

use crate::{
    archetype::Archetype, archetype_data_layout::ArchetypeDataLayout,
    archetype_data_page::ArchetypeDataPage, entity_in_archetype::EntityInArchetype,
};

#[derive(Debug)]
pub(crate) struct ArchetypesContainer {
    archetypes: Vec<Archetype>,
    archetype_to_pages: Vec<ArchetypePages>,
    archetype_layouts: Vec<ArchetypeDataLayout>,

    pages: Vec<ArchetypeDataPage>,
    page_to_archetype: Vec<isize>,

    free_archetypes: Vec<usize>,
    free_pages: Vec<usize>,
}

#[derive(Debug)]
struct ArchetypePages {
    pages: Vec<usize>,
}

impl ArchetypesContainer {
    const ARCHETYPE_DEFAULT_CAPACITY: usize = 5;
    const ARCHETYPE_PAGE_DEFAULT_CAPACITY: usize = 5;

    pub fn new() -> ArchetypesContainer {
        let archetypes = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);
        let archetype_to_pages = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);
        let archetype_layouts = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);

        let pages = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY)
            .map(|_| ArchetypeDataPage::new())
            .collect();

        let page_to_archetype = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY)
            .map(|_| -1)
            .collect();

        let free_archetypes = Vec::new();
        let free_pages = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY).collect();

        ArchetypesContainer {
            archetypes,
            archetype_to_pages,
            archetype_layouts,

            pages,
            page_to_archetype,

            free_archetypes,
            free_pages,
        }
    }

    pub fn add_entity(&mut self, entity_id: u32, archetype: &Archetype) -> EntityInArchetype {
        let archetype_index = self.reserve_archetype(archetype);
        let pages = &mut self.archetype_to_pages[archetype_index].pages;

        for page_index in pages {
            let page_index = *page_index;
            let page = &mut self.pages[page_index];
            if page.has_free_space() {
                page.push_entity_id(entity_id);

                return EntityInArchetype {
                    archetype_index,
                    page_index,
                };
            }
        }

        let page_index = self.reserve_page_for_archetype(archetype_index);
        self.pages[page_index].push_entity_id(entity_id);

        EntityInArchetype {
            archetype_index,
            page_index,
        }
    }

    pub fn remove_entity(&mut self, entity_id: u32, entity_in_archetype: EntityInArchetype) {
        let page = &mut self.pages[entity_in_archetype.page_index];

        page.remove_entity_id(entity_id);
    }

    pub fn get_component_ref_by_entity_id<'a, T: 'static>(
        &'a self,
        entity_id: u32,
        entity_in_archetype: EntityInArchetype,
    ) -> &T {
        let component_offset =
            self.get_valid_component_offset::<T>(entity_in_archetype.archetype_index);
        let page = &self.pages[entity_in_archetype.page_index];

        unsafe {
            &*get_component_data_ptr_mut(page, page.get_entity_index_by_id(entity_id), component_offset)
        }
    }

    pub fn get_component_ref_mut_by_entity_id<'a, T: 'static>(
        &'a mut self,
        entity_id: u32,
        entity_in_archetype: EntityInArchetype,
    ) -> &'a mut T {
        let component_offset =
            self.get_valid_component_offset::<T>(entity_in_archetype.archetype_index);
        let page = &self.pages[entity_in_archetype.page_index];

        unsafe {
            &mut *get_component_data_ptr_mut(page, page.get_entity_index_by_id(entity_id), component_offset)
        }
    }

    pub(crate) fn get_valid_component_offset<T: 'static>(&self, archetype_index: usize) -> usize {
        let archetype = &self.archetypes[archetype_index];
        let layout = &self.archetype_layouts[archetype_index];

        let component_index = archetype.find_component_index(std::any::TypeId::of::<T>());
        assert!(
            component_index.is_some(),
            "There is no component {} in the given archetype",
            std::any::type_name::<T>()
        );

        unsafe {
            let component_index = component_index.unwrap_unchecked();
            *layout.component_offsets().add(component_index)
        }
    }

    pub(crate) fn get_suitable_page_indices<'a>(
        &'a self, 
        include_types: &'a [TypeId]
    ) -> impl Iterator<Item=usize> + 'a {
        self.archetypes.iter().enumerate()
            .filter(|(_, arch)| { arch.is_include_ids(include_types) })
            .flat_map(|(idx, _)| { 
                (&self.archetype_to_pages[idx].pages).into_iter().map(|v| *v)
            })
    }

    pub(crate) fn get_page_entity_ids_slice<'a>(&'a self, page_index: usize) -> &'a [u32] {
        &self.pages[page_index].entities_ids()
    }

    pub(crate) fn get_components_slices<'a, T0: 'static, T1: 'static>(
        &'a self,
        page_index: usize
    ) -> (&'a [T0], &'a [T1]) {
        let archetype_index = self.page_to_archetype[page_index];
        assert!(archetype_index >= 0, "There is no pages for the given archetype");

        let archetype_index = archetype_index as usize;
        let page = &self.pages[page_index];
        let entities_count = page.entities_count();

        let offsets = (
            self.get_valid_component_offset::<T0>(archetype_index),
            self.get_valid_component_offset::<T1>(archetype_index),
        );

        unsafe{(
            &*std::ptr::slice_from_raw_parts(get_component_data_ptr(page, 0, offsets.0), entities_count),
            &*std::ptr::slice_from_raw_parts(get_component_data_ptr(page, 0, offsets.1), entities_count),
        )}
    }

    fn reserve_archetype(&mut self, archetype: &Archetype) -> usize {
        for (i, arch) in self.archetypes.iter().enumerate() {
            if arch.is_same_as(&archetype) {
                return i;
            }
        }

        match self.free_archetypes.pop() {
            Some(free_idx) => {
                self.archetype_layouts[free_idx] = ArchetypeDataLayout::new(&archetype);
                self.archetypes[free_idx] = archetype.clone();

                free_idx
            }
            None => {
                let new_size = self.archetypes.len() + 1;
                self.archetype_layouts
                    .resize_with(new_size, || ArchetypeDataLayout::new(&archetype));
                self.archetypes.resize_with(new_size, || archetype.clone());
                self.archetype_to_pages
                    .resize_with(new_size, || ArchetypePages { pages: Vec::new() });

                new_size - 1
            }
        }
    }

    fn reserve_page_for_archetype(&mut self, archetype_index: usize) -> usize {
        let page_index = match self.free_pages.pop() {
            Some(page_index) => page_index,
            None => {
                let page_index = self.pages.len();
                self.pages.insert(page_index, ArchetypeDataPage::new());
                page_index
            }
        };

        let layout = &self.archetype_layouts[archetype_index];
        self.pages[page_index].set_layout(layout);

        self.archetype_to_pages[archetype_index].pages.push(page_index);
        self.page_to_archetype[page_index] = archetype_index as isize;

        page_index
    }
}

#[inline(always)]
fn get_component_data_ptr<'a, T: 'static>(
    page: &ArchetypeDataPage,
    entity_index: usize,
    component_offset: usize
) -> *const T {
    page.get_component_data_ptr(entity_index, component_offset, size_of::<T>()) as *const T
}

#[inline(always)]
fn get_component_data_ptr_mut<'a, T: 'static>(
    page: &ArchetypeDataPage,
    entity_index: usize,
    component_offset: usize,
) -> *mut T {
    page.get_component_data_ptr_mut(entity_index, component_offset, size_of::<T>()) as *mut T
}
