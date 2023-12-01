use crate::{
    archetype::Archetype,
    archetype_data_page::{ArchetypeDataPage, SwapRemoveInfo},
    archetype_data_page_view::ArchetypeDataPageView,
    entity_in_archetype::EntityInArchetype,
};

#[derive(Debug)]
pub(crate) struct ArchetypesContainer {
    archetypes: Vec<Archetype>,
    archetype_to_pages: Vec<ArchetypePages>,

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
        let mut archetypes = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);
        archetypes.push(Archetype::empty());

        let mut archetype_to_pages = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);
        archetype_to_pages.push(ArchetypePages { pages: Vec::new() });

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

        let page_index = self.reserve_page_for_archetype(archetype_index);
        let index_in_page = self.pages[page_index].add_entity_id(entity_id) as u32;
        let page_index = page_index as u32;

        EntityInArchetype {
            page_index,
            index_in_page,
        }
    }

    pub fn swap_remove_entity(
        &mut self,
        entity_in_archetype: EntityInArchetype,
    ) -> Option<SwapRemoveInfo> {
        let page_index = entity_in_archetype.page_index as usize;
        let page = &mut self.pages[page_index];
        let page_will_empty = page.entities_count() == 1;

        let arch_index = self.page_to_archetype[page_index] as usize;
        let arch = &self.archetypes[arch_index];
        let swap_remove = page.swap_remove_entity_at_index(
            entity_in_archetype.index_in_page as usize,
            &arch,
        );

        if page_will_empty {
            self.free_page(page_index);
        }

        swap_remove
    }

    #[inline(always)]
    pub unsafe fn get_page_view_unchecked(
        &self,
        page_index: usize,
    ) -> ArchetypeDataPageView {
        let arch_idx = *self.page_to_archetype.get_unchecked(page_index) as usize;
        ArchetypeDataPageView {
            archetype: self.archetypes.get_unchecked(arch_idx),
            page: self.pages.get_unchecked(page_index),
        }
    }

    #[inline]
    pub fn get_archetypes(&self) -> &[Archetype] {
        &self.archetypes
    }

    #[inline]
    pub fn get_pages(&self) -> &[ArchetypeDataPage] {
        &self.pages
    }

    #[inline]
    pub fn get_archetype_page_indices(&self, archetype_index: usize) -> &[usize] {
        &self.archetype_to_pages[archetype_index].pages
    }

    fn free_page(&mut self, page_index: usize) {
        let archetype_index = self.page_to_archetype[page_index];
        if archetype_index < 0 {
            return;
        }
        let archetype_index = archetype_index as usize;
        let arch_pages = &mut self.archetype_to_pages[archetype_index];

        for i in 0..arch_pages.pages.len() {
            if arch_pages.pages[i] == page_index {
                arch_pages.pages.remove(i);
                if arch_pages.pages.len() == 0 {
                    self.free_archetypes.push(archetype_index);
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

        match self.free_archetypes.pop() {
            Some(free_idx) => {
                self.archetypes[free_idx] = archetype.clone();

                free_idx
            }
            None => {
                self.archetypes.push(archetype.clone());
                self.archetype_to_pages
                    .push(ArchetypePages { pages: Vec::new() });

                self.archetypes.len() - 1
            }
        }
    }

    fn reserve_page_for_archetype(&mut self, archetype_index: usize) -> usize {
        let page_index = match self.free_pages.pop() {
            Some(page_index) => page_index,
            None => {
                let page_index = self.pages.len();
                self.pages.insert(page_index, ArchetypeDataPage::new());
                self.page_to_archetype
                    .insert(page_index, archetype_index as isize);
                page_index
            }
        };

        let arch = &self.archetypes[archetype_index];
        self.pages[page_index].set_archetype(arch);

        self.archetype_to_pages[archetype_index]
            .pages
            .push(page_index);
        self.page_to_archetype[page_index] = archetype_index as isize;

        page_index
    }
}
