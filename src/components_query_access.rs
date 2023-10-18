use crate::{archetype_data_page::ArchetypeDataPage, tuple::ComponentsTuple, Archetype};

pub trait ComponentsQueryAccess {
    type OffsetsTuple;
    type AccessOutput<'a>;

    fn is_archetype_include_types(archetype: &Archetype) -> bool;

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a>;

    fn get_offsets(
        archetype: &Archetype,
        layout_offsets: *const usize,
    ) -> Self::OffsetsTuple;
}

impl<TRead, TWrite> ComponentsQueryAccess for (TRead, TWrite)
where
    TRead: ComponentsTuple,
    TWrite: ComponentsTuple,
{
    type OffsetsTuple = (TRead::OffsetsTuple, TWrite::OffsetsTuple);
    type AccessOutput<'a> = (TRead::RefsTuple<'a>, TWrite::MutRefsTuple<'a>);

    fn get_offsets(
        archetype: &Archetype,
        layout_offsets: *const usize,
    ) -> Self::OffsetsTuple {
        (
            TRead::get_offsets(archetype, layout_offsets),
            TWrite::get_offsets(archetype, layout_offsets),
        )
    }

    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TRead::is_archetype_include_types(archetype)
            && TWrite::is_archetype_include_types(archetype)
    }

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        (read_offsets, write_offsets): &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a> {
        (
            TRead::get_refs(page, entity_index, read_offsets),
            TWrite::get_refs_mut(page, entity_index, write_offsets),
        )
    }
}

impl<TRead> ComponentsQueryAccess for (TRead, ())
where
    TRead: ComponentsTuple,
{
    type OffsetsTuple = TRead::OffsetsTuple;
    type AccessOutput<'a> = TRead::RefsTuple<'a>;

    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TRead::is_archetype_include_types(archetype)
    }

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a> {
        TRead::get_refs(page, entity_index, offsets)
    }

    fn get_offsets(
        archetype: &Archetype,
        layout_offsets: *const usize,
    ) -> Self::OffsetsTuple {
        TRead::get_offsets(archetype, layout_offsets)
    }
}

impl<TWrite> ComponentsQueryAccess for ((), TWrite)
where
    TWrite: ComponentsTuple,
{
    type OffsetsTuple = TWrite::OffsetsTuple;
    type AccessOutput<'a> = TWrite::MutRefsTuple<'a>;

    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TWrite::is_archetype_include_types(archetype)
    }

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a> {
        TWrite::get_refs_mut(page, entity_index, offsets)
    }

    fn get_offsets(
        archetype: &Archetype,
        layout_offsets: *const usize,
    ) -> Self::OffsetsTuple {
        TWrite::get_offsets(archetype, layout_offsets)
    }
}
