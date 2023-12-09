use std::marker::PhantomData;

use crate::{
    archetype_data_page::ArchetypeDataPage, archetype_layout::ArchetypeLayout,
    component_tuple::ComponentTuple, Archetype,
};

pub struct ReadonlyAccess<TRead: ComponentTuple> {
    _read: PhantomData<TRead>,
}

pub struct WriteAccess<TWrite: ComponentTuple> {
    _write: PhantomData<TWrite>,
}

pub struct ReadWriteAccess<TRead: ComponentTuple, TWrite: ComponentTuple> {
    _read: PhantomData<TRead>,
    _write: PhantomData<TWrite>,
}

pub trait ComponentQueryAccess {
    type AccessOutput<'a>;
    type OffsetTuple: Default + Copy;
    type PtrTuple: Copy;

    fn is_archetype_include_types(archetype: &Archetype) -> bool;

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetTuple,
    ) -> Self::AccessOutput<'a>;

    fn get_offsets(archetype: &Archetype, layout: &ArchetypeLayout) -> Self::OffsetTuple;

    fn get_ptrs(
        page: &ArchetypeDataPage,
        offsets: &Self::OffsetTuple,
    ) -> Self::PtrTuple;

    fn add_to_ptrs(
        ptrs: &Self::PtrTuple,
        entity_index: usize,
    ) -> Self::PtrTuple;

    fn null_ptrs() -> Self::PtrTuple;

    fn ptrs_to_refs<'a>(ptrs: Self::PtrTuple) -> Self::AccessOutput<'a>;
}

impl<TRead, TWrite> ComponentQueryAccess for ReadWriteAccess<TRead, TWrite>
where
    TRead: ComponentTuple,
    TWrite: ComponentTuple,
{
    type PtrTuple = (TRead::PtrTuple, TWrite::MutPtrTuple);
    type OffsetTuple = (TRead::OffsetTuple, TWrite::OffsetTuple);
    type AccessOutput<'a> = (TRead::RefTuple<'a>, TWrite::MutRefTuple<'a>);

    #[inline(always)]
    fn get_offsets(archetype: &Archetype, layout: &ArchetypeLayout) -> Self::OffsetTuple {
        (
            TRead::get_offsets_unchecked(archetype, layout),
            TWrite::get_offsets_unchecked(archetype, layout),
        )
    }

    #[inline(always)]
    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TRead::is_archetype_include_types(archetype)
            && TWrite::is_archetype_include_types(archetype)
    }

    #[inline(always)]
    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        (read_offsets, write_offsets): &Self::OffsetTuple,
    ) -> Self::AccessOutput<'a> {
        (
            TRead::get_refs(page, entity_index, read_offsets),
            TWrite::get_refs_mut(page, entity_index, write_offsets),
        )
    }

    #[inline(always)]
    fn get_ptrs(
        page: &ArchetypeDataPage,
        (read_offsets, write_offsets): &Self::OffsetTuple,
    ) -> Self::PtrTuple {
        (
            TRead::get_ptrs(page, read_offsets),
            TWrite::get_ptrs_mut(page, write_offsets),
        )
    }

    #[inline(always)]
    fn add_to_ptrs(
        (read_ptrs, write_ptrs): &Self::PtrTuple,
        entity_index: usize,
    ) -> Self::PtrTuple {
        (
            TRead::add_to_ptrs(read_ptrs, entity_index),
            TWrite::add_to_ptrs_mut(write_ptrs, entity_index),
        )
    }

    #[inline(always)]
    fn null_ptrs() -> Self::PtrTuple {
        (TRead::null_ptrs(), TWrite::null_ptrs_mut())
    }

    #[inline(always)]
    fn ptrs_to_refs<'a>((read, write): Self::PtrTuple) -> Self::AccessOutput<'a> {
        (TRead::ptrs_to_refs(read), TWrite::ptrs_to_refs_mut(write))
    }
}

impl<TRead: ComponentTuple> ComponentQueryAccess for ReadonlyAccess<TRead> {
    type OffsetTuple = TRead::OffsetTuple;
    type AccessOutput<'a> = TRead::RefTuple<'a>;
    type PtrTuple = TRead::PtrTuple;

    #[inline(always)]
    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TRead::is_archetype_include_types(archetype)
    }

    #[inline(always)]
    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetTuple,
    ) -> Self::AccessOutput<'a> {
        TRead::get_refs(page, entity_index, offsets)
    }

    #[inline(always)]
    fn get_offsets(archetype: &Archetype, layout: &ArchetypeLayout) -> Self::OffsetTuple {
        TRead::get_offsets_unchecked(archetype, layout)
    }

    #[inline(always)]
    fn get_ptrs(page: &ArchetypeDataPage, offsets: &Self::OffsetTuple) -> Self::PtrTuple {
        TRead::get_ptrs(page, offsets)
    }

    #[inline(always)]
    fn add_to_ptrs(ptrs: &Self::PtrTuple, entity_index: usize) -> Self::PtrTuple {
        TRead::add_to_ptrs(ptrs, entity_index)
    }

    #[inline(always)]
    fn null_ptrs() -> Self::PtrTuple {
        TRead::null_ptrs()
    }

    #[inline(always)]
    fn ptrs_to_refs<'a>(ptrs: Self::PtrTuple) -> Self::AccessOutput<'a> {
        TRead::ptrs_to_refs(ptrs)
    }
}

impl<TWrite: ComponentTuple> ComponentQueryAccess for WriteAccess<TWrite> {
    type OffsetTuple = TWrite::OffsetTuple;
    type AccessOutput<'a> = TWrite::MutRefTuple<'a>;
    type PtrTuple = TWrite::MutPtrTuple;

    #[inline(always)]
    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TWrite::is_archetype_include_types(archetype)
    }

    #[inline(always)]
    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetTuple,
    ) -> Self::AccessOutput<'a> {
        TWrite::get_refs_mut(page, entity_index, offsets)
    }

    #[inline(always)]
    fn get_offsets(archetype: &Archetype, layout: &ArchetypeLayout) -> Self::OffsetTuple {
        TWrite::get_offsets_unchecked(archetype, layout)
    }

    #[inline(always)]
    fn get_ptrs(page: &ArchetypeDataPage, offsets: &Self::OffsetTuple) -> Self::PtrTuple {
        TWrite::get_ptrs_mut(page, offsets)
    }

    #[inline(always)]
    fn add_to_ptrs(ptrs: &Self::PtrTuple, entity_index: usize) -> Self::PtrTuple {
        TWrite::add_to_ptrs_mut(ptrs, entity_index)
    }

    #[inline(always)]
    fn null_ptrs() -> Self::PtrTuple {
        TWrite::null_ptrs_mut()
    }

    #[inline(always)]
    fn ptrs_to_refs<'a>(ptrs: Self::PtrTuple) -> Self::AccessOutput<'a> {
        TWrite::ptrs_to_refs_mut(ptrs)
    }
}
