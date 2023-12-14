use crate::{
    archetype::Archetype,
    archetype_data_page::{ArchetypeDataPage, SwapRemoveInfo},
    archetype_data_page_view::ArchetypeDataPageView,
    archetype_layout::ArchetypeLayout,
    entity_in_archetype::EntityInArchetype,
};

#[derive(Debug)]
pub(crate) struct ArchetypesContainer {
    archetypes: Vec<Archetype>,
    layouts: Vec<ArchetypeLayout>,
    archetype_to_pages: Vec<ArchetypePages>,

    pages: Vec<ArchetypeDataPage>,
    page_to_archetype: Vec<usize>,

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
        let layouts = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);

        let archetype_to_pages = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);

        let pages = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY)
            .map(|_| ArchetypeDataPage::new())
            .collect();

        let page_to_archetype = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY)
            .map(|_| 0)
            .collect();

        let free_archetypes = Vec::new();
        let free_pages = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY).collect();

        ArchetypesContainer {
            archetypes,
            layouts,
            archetype_to_pages,

            pages,
            page_to_archetype,

            free_archetypes,
            free_pages,
        }
    }

    pub fn add_entity(
        &mut self,
        entity_id: u32,
        archetype: &Archetype,
    ) -> EntityInArchetype {
        let archetype_index = self.reserve_archetype(archetype);
        self.reserve_page(entity_id, archetype_index)
    }

    pub fn swap_remove_entity(
        &mut self,
        entity_in_archetype: EntityInArchetype,
    ) -> Option<SwapRemoveInfo> {
        let page_index = entity_in_archetype.page_index as usize;
        let arch_index = self.page_to_archetype[page_index];

        let page = &mut self.pages[page_index];
        let page_will_empty = page.entity_count() == 1;

        let arch = &self.archetypes[arch_index];
        let layout = &self.layouts[arch_index];
        let swap_remove = page.swap_remove_entity_at_index(
            entity_in_archetype.index_in_page as usize,
            &arch,
            &layout,
        );

        if page_will_empty {
            self.free_page(page_index);
        }

        swap_remove
    }

    pub fn move_entity_to_other_archetype(
        &mut self,
        entity_id: u32,
        previous_entity_in_archetype: EntityInArchetype,
        previous_archetype_index: usize,
        new_archetype: &Archetype,
    ) -> EntityInArchetype {
        let new_archetype_index = self.reserve_archetype(new_archetype);
        let new_entity_in_arch = self.reserve_page(entity_id, new_archetype_index);

        let prev_page = &self.pages[previous_entity_in_archetype.page_index as usize];
        let new_page = &self.pages[new_entity_in_arch.page_index as usize];
        let prev_archetype = &self.archetypes[previous_archetype_index];

        let prev_layout = &self.layouts[previous_archetype_index];
        let new_layout = &self.layouts[new_archetype_index];

        unsafe {
            ArchetypeDataPage::copy_component_data_to_page_with_new_archetype(
                prev_page,
                new_page,
                previous_entity_in_archetype.index_in_page as usize,
                new_entity_in_arch.index_in_page as usize,
                prev_archetype,
                new_archetype,
                prev_layout,
                new_layout,
            );
        }

        new_entity_in_arch
    }

    #[inline(always)]
    pub unsafe fn get_page_view_unchecked(
        &self,
        page_index: usize,
    ) -> ArchetypeDataPageView {
        let arch_idx = *self.page_to_archetype.get_unchecked(page_index) as usize;
        ArchetypeDataPageView {
            archetype: self.archetypes.get_unchecked(arch_idx),
            layout: self.layouts.get_unchecked(arch_idx),
            page: self.pages.get_unchecked(page_index),
        }
    }

    #[inline]
    pub fn get_archetypes(&self) -> &[Archetype] {
        &self.archetypes
    }

    #[inline]
    pub unsafe fn get_archetype_with_layout_unchecked(
        &self,
        archetype_index: usize,
    ) -> (&Archetype, &ArchetypeLayout) {
        (
            self.archetypes.get_unchecked(archetype_index),
            self.layouts.get_unchecked(archetype_index),
        )
    }

    #[inline]
    pub fn get_pages(&self) -> &[ArchetypeDataPage] {
        &self.pages
    }

    #[inline]
    pub unsafe fn get_page_by_index_unchecked(
        &self,
        page_index: u32,
    ) -> &ArchetypeDataPage {
        self.pages.get_unchecked(page_index as usize)
    }

    #[inline]
    pub fn get_archetype_page_indices(&self, archetype_index: usize) -> &[usize] {
        &self.archetype_to_pages[archetype_index].pages
    }

    #[inline]
    pub fn get_archetype_index_by_page(&self, page_index: usize) -> usize {
        self.page_to_archetype[page_index] as usize
    }

    #[inline]
    pub fn get_archetype_by_page(&self, page_index: usize) -> &Archetype {
        let arch_index = self.page_to_archetype[page_index] as usize;
        &self.archetypes[arch_index]
    }

    fn free_page(&mut self, page_index: usize) {
        let archetype_index = self.page_to_archetype[page_index];
        if archetype_index == 0 {
            return;
        }
        let archetype_index = archetype_index as usize;
        let arch_pages = &mut self.archetype_to_pages[archetype_index];

        for i in 0..arch_pages.pages.len() {
            if arch_pages.pages[i] == page_index {
                arch_pages.pages.remove(i);
                if arch_pages.pages.len() == 0 {
                    match self.free_archetypes.binary_search(&archetype_index) {
                        Err(i) => self.free_archetypes.insert(i, archetype_index),
                        _ => {} 
                    }
                }
                break;
            }
        }

        self.free_pages.push(page_index);
    }

    fn reserve_archetype(&mut self, archetype: &Archetype) -> usize {
        for (i, arch) in self.archetypes.iter().enumerate() {
            if arch.is_same_as(&archetype) {
                return i;
            }
        }

        let layout = ArchetypeLayout::new(archetype);
        match self.free_archetypes.pop() {
            Some(free_idx) => {
                self.archetypes[free_idx] = archetype.clone();
                self.layouts[free_idx] = layout;

                free_idx
            }
            None => {
                self.archetypes.push(archetype.clone());
                self.layouts.push(layout);
                self.archetype_to_pages
                    .push(ArchetypePages { pages: Vec::new() });

                self.archetypes.len() - 1
            }
        }
    }

    fn reserve_page(
        &mut self,
        entity_id: u32,
        archetype_index: usize,
    ) -> EntityInArchetype {
        let pages = &mut self.archetype_to_pages[archetype_index].pages;

        // Looking in reverse direction, because there is more probability, that
        // page with free slots will be located in the end
        for page_index in pages.into_iter().rev() {
            let page_index = *page_index;
            let page = &mut self.pages[page_index];
            let page_index = page_index as u32;
            if page.has_free_space() {
                let index_in_page = page.add_entity_id(entity_id) as u32;

                return EntityInArchetype {
                    page_index,
                    index_in_page,
                };
            }
        }

        let page_index = match self.free_pages.pop() {
            Some(page_index) => page_index,
            None => {
                let page_index = self.pages.len();
                self.pages.insert(page_index, ArchetypeDataPage::new());
                self.page_to_archetype.insert(page_index, archetype_index);
                page_index
            }
        };

        let layout = &self.layouts[archetype_index];
        self.pages[page_index].set_layout(layout);

        self.archetype_to_pages[archetype_index]
            .pages
            .push(page_index);
        self.page_to_archetype[page_index] = archetype_index;

        let index_in_page = self.pages[page_index].add_entity_id(entity_id) as u32;
        let page_index = page_index as u32;

        EntityInArchetype {
            page_index,
            index_in_page,
        }
    }
}
