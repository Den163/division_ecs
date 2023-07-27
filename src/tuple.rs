use paste::paste;

use crate::{archetype_data_page::ArchetypeDataPage, type_ids, Archetype};

pub trait EmptyTuple {}
pub trait NonEmptyTuple {}

pub trait ComponentsTuple {
    type OffsetsTuple;
    type RefsTuple<'a>;
    type MutRefsTuple<'a>;
    const CHECK_ARCHETYPE: bool;

    fn get_offsets(
        archetype: &Archetype, 
        layout_offsets: *const usize
    ) -> Self::OffsetsTuple;

    fn get_offsets_checked(
        archetype: &Archetype,
        layout_offsets: *const usize,
    ) -> Self::OffsetsTuple;

    fn is_archetype_include_types(archetype: &Archetype) -> bool;

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::RefsTuple<'a>;

    fn get_refs_mut<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::MutRefsTuple<'a>;
}

macro_rules! components_tuple_impl {
    ($($T:ident),*) => {
        impl<$($T: 'static,)*> NonEmptyTuple for ($($T,)*) {
        }

        impl<$($T: 'static,)*> ComponentsTuple for ($($T,)*) {
            type OffsetsTuple = ($(components_tuple_impl!(@type_to_usize, $T),)*);
            type RefsTuple<'a> = ($(&'a $T,)*);
            type MutRefsTuple<'a> = ($(&'a mut $T,)*);
            const CHECK_ARCHETYPE: bool = true;

            #[inline]
            fn get_offsets(archetype: &Archetype, layout_offsets: *const usize) -> Self::OffsetsTuple {
                unsafe {(
                    $(*layout_offsets.add(archetype.find_component_index(std::any::TypeId::of::<$T>()).unwrap_unchecked()),)*
                )}
            }

            #[inline]
            fn get_offsets_checked(archetype: &Archetype, layout_offsets: *const usize) -> Self::OffsetsTuple {
                unsafe {(
                    $(*layout_offsets.add(archetype.find_component_index(std::any::TypeId::of::<$T>()).unwrap()),)*
                )}
            }

            #[inline]
            fn is_archetype_include_types(archetype: &Archetype) -> bool {
                archetype.is_include_ids(&type_ids!($($T),*))
            }

            #[inline(always)]
            fn get_refs<'a>(
                page: &'a ArchetypeDataPage,
                entity_index: usize,
                ($( paste!([<$T:lower>]) ,) *): &<($($T,)*) as ComponentsTuple>::OffsetsTuple
            ) -> Self::RefsTuple<'a> {
                unsafe {(
                    $(
                        &*(
                            page.get_component_data_ptr(
                                entity_index, *paste!{ [<$T:lower>] } ,
                                std::mem::size_of::<$T>()
                            ) as *const $T
                        ),
                    )*
                )}
            }

            #[inline(always)]
            fn get_refs_mut<'a>(
                page: &'a ArchetypeDataPage,
                entity_index: usize,
                ($( paste!([<$T:lower>]) ,) *): &<($($T,)*) as ComponentsTuple>::OffsetsTuple
            ) -> Self::MutRefsTuple<'a> {
                unsafe {(
                    $(
                        &mut *(
                            page.get_component_data_ptr_mut(
                                entity_index, *paste!{ [<$T:lower>] } ,
                                std::mem::size_of::<$T>()
                            ) as *mut $T
                        ),
                    )*
                )}
            }
        }
    };

    (@type_to_usize, $T: tt) => { usize };
}

components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6);
components_tuple_impl!(T0, T1, T2, T3, T4, T5);
components_tuple_impl!(T0, T1, T2, T3, T4);
components_tuple_impl!(T0, T1, T2, T3);
components_tuple_impl!(T0, T1, T2);
components_tuple_impl!(T0, T1);
components_tuple_impl!(T0);

impl EmptyTuple for () {}

impl ComponentsTuple for () {
    type OffsetsTuple = ();
    type RefsTuple<'a> = ();
    type MutRefsTuple<'a> = ();
    const CHECK_ARCHETYPE: bool = false;

    #[inline(always)]
    fn is_archetype_include_types(_: &Archetype) -> bool {
        true
    }

    #[inline(always)]
    fn get_offsets(_: &Archetype, _: *const usize) -> Self::OffsetsTuple {}

    #[inline(always)]
    fn get_offsets_checked(_: &Archetype, _: *const usize) -> Self::OffsetsTuple {}

    #[inline(always)]
    fn get_refs<'a>(
        _: &'a ArchetypeDataPage,
        _: usize,
        _: &Self::OffsetsTuple,
    ) -> Self::RefsTuple<'a> {
    }

    #[inline(always)]
    fn get_refs_mut<'a>(
        _: &'a ArchetypeDataPage,
        _: usize,
        _: &Self::OffsetsTuple,
    ) -> Self::MutRefsTuple<'a> {
    }
}
