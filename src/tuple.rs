use paste::paste;

use crate::{archetype_data_page::ArchetypeDataPage, type_ids, Archetype, Component};

pub trait ComponentsTuple {
    type OffsetsTuple;
    type RefsTuple<'a>;
    type MutRefsTuple<'a>;

    fn get_offsets(archetype: &Archetype) -> Self::OffsetsTuple;

    fn get_offsets_checked(archetype: &Archetype) -> Self::OffsetsTuple;

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
        #[allow(unused_parens)]
        impl<$($T: 'static + Component),*> ComponentsTuple for ($($T),*) {
            type OffsetsTuple = ($(components_tuple_impl!(@type_to_usize, $T)),*);
            type RefsTuple<'a> = ($(&'a $T,)*);
            type MutRefsTuple<'a> = ($(&'a mut $T,)*);

            #[inline]
            fn get_offsets(archetype: &Archetype) -> Self::OffsetsTuple {
                unsafe {(
                    $(
                        *(
                            archetype.component_offsets().add(
                                archetype.find_component_index(std::any::TypeId::of::<$T>())
                                        .unwrap_unchecked()
                            )
                        )
                    ),*
                )}
            }

            #[inline]
            fn get_offsets_checked(archetype: &Archetype) -> Self::OffsetsTuple {
                unsafe {(
                    $(
                        *(archetype.component_offsets().add(
                            archetype.find_component_index_typed_checked::<$T>()
                        ))
                    ),*
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
                ($( paste!([<$T:lower>]) ),*): &<($($T),*) as ComponentsTuple>::OffsetsTuple
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
                ($( paste!([<$T:lower>]) ),*): &<($($T),*) as ComponentsTuple>::OffsetsTuple
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

        $crate::archetype_builder_tuple_impl!($($T),*);
        $crate::tuple_into_archetype_impl!($($T),*);
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
