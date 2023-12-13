use std::any::TypeId;

use crate::{mem_utils, Entity, Store, Tag};

pub struct OrderGroupContainer {
    group_ids: Vec<TypeId>,
    group_index_to_forward_links: Vec<*mut u32>,
    group_index_to_backward_links: Vec<*mut u32>,
    group_index_to_head: Vec<u32>,
    group_index_to_tail: Vec<u32>,
    entity_capacity: usize,
}

struct OrderGroupInfoMut<'a> {
    head: &'a mut u32,
    tail: &'a mut u32,
    forward_links: *mut u32,
    backward_links: *mut u32,
}

impl OrderGroupContainer {
    const NULL_ID_BYTE: u8 = Entity::NULL_ID as u8;

    pub fn new(entity_capacity: usize) -> Self {
        Self {
            group_ids: Vec::new(),
            group_index_to_forward_links: Vec::new(),
            group_index_to_backward_links: Vec::new(),
            group_index_to_head: Vec::new(),
            group_index_to_tail: Vec::new(),
            entity_capacity,
        }
    }

    #[inline]
    pub fn add_id_ordered_by<T: Tag>(&mut self, entity_id: u32) {
        let group_index = self.reserve_group::<T>();

        let OrderGroupInfoMut {
            backward_links,
            forward_links,
            head,
            tail,
        } = unsafe { self.get_order_group_info_unchecked_mut(group_index) };

        if *head == Entity::NULL_ID {
            *head = entity_id;
            *tail = entity_id;
        } else {
            unsafe {
                let fwd_id = *forward_links.add(entity_id as usize);
                let bwd_id = &mut *backward_links.add(entity_id as usize);

                Self::assert_entity_has_no_order(entity_id, fwd_id, *bwd_id);

                *forward_links.add(*tail as usize) = entity_id;
                *bwd_id = *tail;

                *tail = entity_id;
            }
        }
    }

    #[inline]
    pub fn add_id_next_to_ordered_by<T: Tag>(&mut self, entity_id: u32, next_to_id: u32) {
        let group_index = self.reserve_group::<T>();

        let OrderGroupInfoMut {
            backward_links,
            forward_links,
            tail,
            ..
        } = unsafe { self.get_order_group_info_unchecked_mut(group_index) };

        unsafe {
            let fwd_entity_id = &mut *forward_links.add(entity_id as usize);
            let bwd_entity_id = &mut *backward_links.add(entity_id as usize);

            Self::assert_entity_has_no_order(entity_id, *fwd_entity_id, *bwd_entity_id);

            let fwd_after_id = &mut *forward_links.add(next_to_id as usize);
            if *fwd_after_id != Entity::NULL_ID {
                *backward_links.add(*fwd_after_id as usize) = entity_id;
                *fwd_entity_id = *fwd_after_id;
            }

            *fwd_after_id = entity_id;
            *bwd_entity_id = next_to_id;

            if *tail == next_to_id {
                *tail = entity_id
            }
        }
    }

    #[inline]
    pub fn add_id_previous_to_ordered_by<T: Tag>(
        &mut self,
        entity_id: u32,
        previous_to_id: u32,
    ) {
        let group_index = self.reserve_group::<T>();

        let OrderGroupInfoMut {
            backward_links,
            forward_links,
            head,
            ..
        } = unsafe { self.get_order_group_info_unchecked_mut(group_index) };

        unsafe {
            let fwd_entity_id = &mut *forward_links.add(entity_id as usize);
            let bwd_entity_id = &mut *backward_links.add(entity_id as usize);

            Self::assert_entity_has_no_order(entity_id, *fwd_entity_id, *bwd_entity_id);

            let bwd_previous_to_id = &mut *backward_links.add(previous_to_id as usize);
            if *bwd_previous_to_id != Entity::NULL_ID {
                *forward_links.add(*bwd_previous_to_id as usize) = entity_id;
                *bwd_entity_id = *bwd_previous_to_id;
            }

            *bwd_previous_to_id = entity_id;
            *fwd_entity_id = previous_to_id;

            if *head == previous_to_id {
                *head = entity_id
            }
        }
    }

    fn assert_entity_has_no_order(entity_id: u32, fwd_id: u32, bwd_id: u32) {
        debug_assert!(
            fwd_id == Entity::NULL_ID,
            "Entity {entity_id} already reference to the next order of entity {fwd_id}. Added entity must have no orders"
        );

        debug_assert!(
            bwd_id == Entity::NULL_ID,
            "Entity {entity_id} already reference to the previous order of entity {bwd_id}. Added entity must have no orders"
        );
    }

    #[inline]
    pub fn remove_id_ordered_by<T: Tag>(&mut self, entity_id: u32) {
        let group_id = TypeId::of::<T>();

        let group_index = match self.group_ids.binary_search(&group_id) {
            Err(_) => return,
            Ok(i) => i,
        };

        self.remove_entity_order_by_internal(group_index, entity_id);
    }

    #[inline]
    pub fn get_next_id_ordered_by<T: Tag>(&self, entity_id: u32) -> Option<u32> {
        self.get_link::<T>(&self.group_index_to_forward_links, entity_id)
    }

    #[inline]
    pub fn get_previous_id_ordered_by<T: Tag>(&self, entity_id: u32) -> Option<u32> {
        self.get_link::<T>(&self.group_index_to_backward_links, entity_id)
    }

    pub fn grow(&mut self, new_capacity: usize) {
        let delta_capacity = new_capacity - self.entity_capacity;

        for (bwd_links, fwd_links) in std::iter::zip(
            &mut self.group_index_to_backward_links,
            &mut self.group_index_to_forward_links,
        ) {
            unsafe {
                *bwd_links =
                    mem_utils::realloc(*bwd_links, self.entity_capacity, new_capacity);
                bwd_links
                    .add(self.entity_capacity)
                    .write_bytes(Self::NULL_ID_BYTE, delta_capacity);

                *fwd_links =
                    mem_utils::realloc(*fwd_links, self.entity_capacity, new_capacity);
                fwd_links
                    .add(self.entity_capacity)
                    .write_bytes(Self::NULL_ID_BYTE, delta_capacity);
            }
        }

        self.entity_capacity = new_capacity;
    }

    pub fn remove_all_orders_for_id(&mut self, id: u32) {
        for group_index in 0..self.group_ids.len() {
            self.remove_entity_order_by_internal(group_index, id);
        }
    }

    #[inline]
    fn reserve_group<T: Tag>(&mut self) -> usize {
        let type_id = TypeId::of::<T>();
        match self.group_ids.binary_search(&type_id) {
            Ok(i) => i,
            Err(i) => {
                self.group_ids.insert(i, type_id);
                let forward_links = self.alloc_links_map();
                let backward_links = self.alloc_links_map();

                self.group_index_to_forward_links.insert(i, forward_links);
                self.group_index_to_backward_links.insert(i, backward_links);
                self.group_index_to_head.insert(i, Entity::NULL_ID);
                self.group_index_to_tail.insert(i, Entity::NULL_ID);

                i
            }
        }
    }

    #[inline]
    fn alloc_links_map(&self) -> *mut u32 {
        unsafe {
            let links = mem_utils::alloc::<u32>(self.entity_capacity);
            links.write_bytes(Self::NULL_ID_BYTE, self.entity_capacity);
            links
        }
    }

    #[inline]
    fn get_link<T: Tag>(
        &self,
        link_index_to_links: &Vec<*mut u32>,
        entity_id: u32,
    ) -> Option<u32> {
        let link_index = TypeId::of::<T>();
        match self.group_ids.binary_search(&link_index) {
            Ok(i) => unsafe {
                let links = *link_index_to_links.get_unchecked(i);
                let target_id = *links.add(entity_id as usize);
                if target_id != Entity::NULL_ID {
                    return Some(target_id);
                } else {
                    return None;
                }
            },
            Err(_) => None,
        }
    }

    #[inline]
    fn get_head_id<T: Tag>(&self) -> Option<u32> {
        self.get_group_index::<T>().and_then(|link_index| {
            let head_id = unsafe { *self.group_index_to_head.get_unchecked(link_index) };
            if head_id != Entity::NULL_ID {
                Some(head_id)
            } else {
                None
            }
        })
    }

    #[inline]
    fn get_tail_id<T: Tag>(&self) -> Option<u32> {
        self.get_group_index::<T>().and_then(|link_index| {
            let tail_id = unsafe { *self.group_index_to_tail.get_unchecked(link_index) };
            if tail_id != Entity::NULL_ID {
                Some(tail_id)
            } else {
                None
            }
        })
    }

    #[inline]
    pub(crate) fn get_group_index<T: Tag>(&self) -> Option<usize> {
        let link_index = TypeId::of::<T>();

        match self.group_ids.binary_search(&link_index) {
            Ok(i) => Some(i),
            Err(_) => None,
        }
    }

    #[inline]
    pub(crate) unsafe fn get_first_id_in_group_unchecked(&self, group_index: usize) -> u32 {
        *self.group_index_to_head.get_unchecked(group_index)
    }

    #[inline]
    pub(crate) unsafe fn get_id_to_next_in_group_map_unchecked(&self, group_index: usize) -> *const u32 {
        *self.group_index_to_forward_links.get_unchecked(group_index)
    }

    #[inline]
    unsafe fn get_order_group_info_unchecked_mut(
        &mut self,
        group_index: usize,
    ) -> OrderGroupInfoMut {
        OrderGroupInfoMut {
            head: self.group_index_to_head.get_unchecked_mut(group_index),
            tail: self.group_index_to_tail.get_unchecked_mut(group_index),
            backward_links: *self
                .group_index_to_backward_links
                .get_unchecked_mut(group_index),
            forward_links: *self
                .group_index_to_forward_links
                .get_unchecked_mut(group_index),
        }
    }

    fn remove_entity_order_by_internal(&mut self, group_index: usize, entity_id: u32) {
        unsafe {
            let OrderGroupInfoMut {
                forward_links,
                backward_links,
                head,
                tail,
            } = self.get_order_group_info_unchecked_mut(group_index);

            let fwd = &mut *forward_links.add(entity_id as usize);
            let bwd = &mut *backward_links.add(entity_id as usize);

            if *fwd != Entity::NULL_ID {
                *backward_links.add(*fwd as usize) = *bwd;
            }

            if *bwd != Entity::NULL_ID {
                *forward_links.add(*bwd as usize) = *fwd;
            }

            if *head == entity_id {
                *head = *fwd;
            }

            if *tail == entity_id {
                *tail = *bwd;
            }

            *bwd = Entity::NULL_ID;
            *fwd = Entity::NULL_ID;
        }
    }
}

impl Store {
    pub fn add_entity_order_by<T: Tag>(&mut self, entity: Entity) {
        self.order_group_container.add_id_ordered_by::<T>(entity.id);
    }

    pub fn add_entity_next_to_order_by<T: Tag>(
        &mut self,
        entity: Entity,
        next_to_entity: Entity,
    ) {
        self.order_group_container
            .add_id_next_to_ordered_by::<T>(entity.id, next_to_entity.id)
    }

    pub fn add_entity_previous_to_order_by<T: Tag>(
        &mut self,
        entity: Entity,
        previous_to_entity: Entity,
    ) {
        self.order_group_container
            .add_id_previous_to_ordered_by::<T>(entity.id, previous_to_entity.id);
    }

    pub fn remove_entity_order_by<T: Tag>(&mut self, entity: Entity) {
        self.order_group_container
            .remove_id_ordered_by::<T>(entity.id);
    }

    pub fn get_next_entity_ordered_by<T: Tag>(&self, entity: Entity) -> Option<Entity> {
        self.order_group_container
            .get_next_id_ordered_by::<T>(entity.id)
            .map(|id| unsafe { self.get_entity_by_id_unchecked(id) })
    }

    pub fn get_previous_entity_ordered_by<T: Tag>(
        &self,
        entity: Entity,
    ) -> Option<Entity> {
        self.order_group_container
            .get_previous_id_ordered_by::<T>(entity.id)
            .map(|id| unsafe { self.get_entity_by_id_unchecked(id) })
    }

    pub fn get_first_entity_ordered_by<T: Tag>(&self) -> Option<Entity> {
        self.order_group_container
            .get_head_id::<T>()
            .map(|id| unsafe { self.get_entity_by_id_unchecked(id) })
    }

    pub fn get_last_entity_ordered_by<T: Tag>(&self) -> Option<Entity> {
        self.order_group_container
            .get_tail_id::<T>()
            .map(|id| unsafe { self.get_entity_by_id_unchecked(id) })
    }
}

impl Drop for OrderGroupContainer {
    fn drop(&mut self) {
        for (&bwd_links, &fwd_links) in std::iter::zip(
            &self.group_index_to_backward_links,
            &self.group_index_to_forward_links,
        ) {
            unsafe {
                mem_utils::dealloc(bwd_links, self.entity_capacity);
                mem_utils::dealloc(fwd_links, self.entity_capacity);
            }
        }
    }
}
