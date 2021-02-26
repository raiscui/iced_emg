/*
 * @Author: Rais
 * @Date: 2021-02-26 09:57:45
 * @LastEditTime: 2021-02-26 14:50:20
 * @LastEditors: Rais
 * @Description:
 */

use std::collections::HashMap;

use anymap::any::CloneAny;
use slotmap::{DefaultKey, DenseSlotMap, Key, SecondaryMap};

use crate::topo_store::StorageKey;

#[derive(Clone, Debug)]
pub struct GStateStore {
    pub anymap: anymap::Map<dyn CloneAny>,
    pub id_to_key_map: HashMap<StorageKey, DefaultKey>,
    pub primary_slotmap: DenseSlotMap<DefaultKey, StorageKey>,
}
impl Default for GStateStore {
    fn default() -> Self {
        GStateStore {
            anymap: anymap::Map::new(),
            id_to_key_map: HashMap::new(),
            primary_slotmap: DenseSlotMap::new(),
        }
    }
}

impl GStateStore {
    pub(crate) fn state_exists_with_id<T: 'static + Clone>(&self, id: StorageKey) -> bool {
        match (self.id_to_key_map.get(&id), self.get_secondarymap::<T>()) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.contains_key(*existing_key)
            }
            (_, _) => false,
        }
    }
    pub fn get_secondarymap<T: 'static + Clone>(&self) -> Option<&SecondaryMap<DefaultKey, T>> {
        self.anymap.get::<SecondaryMap<DefaultKey, T>>()
    }
    pub fn get_mut_secondarymap<T: 'static + Clone>(
        &mut self,
    ) -> Option<&mut SecondaryMap<DefaultKey, T>> {
        self.anymap.get_mut::<SecondaryMap<DefaultKey, T>>()
    }
    pub fn register_secondarymap<T: 'static + Clone>(&mut self) {
        let sm: SecondaryMap<DefaultKey, T> = SecondaryMap::new();
        self.anymap.insert(sm);
    }

    pub(crate) fn set_state_with_id<T: 'static + Clone>(
        &mut self,
        data: T,
        current_id: &StorageKey,
    ) {
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(current_id)
            .copied()
            .unwrap_or_default();

        if key.is_null() {
            let key = self.primary_slotmap.insert(*current_id);
            self.id_to_key_map.insert(*current_id, key);
            if let Some(sec_map) = self.get_mut_secondarymap::<T>() {
                sec_map.insert(key, data);
            } else {
                self.register_secondarymap::<T>();
                self.get_mut_secondarymap::<T>().unwrap().insert(key, data);
            }
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            existing_secondary_map.insert(key, data);
        } else {
            self.register_secondarymap::<T>();
            self.get_mut_secondarymap::<T>().unwrap().insert(key, data);
        }
    }

    pub fn get_state_with_id<T: 'static + Clone>(&self, current_id: &StorageKey) -> Option<&T> {
        match (
            self.id_to_key_map.get(current_id),
            self.get_secondarymap::<T>(),
        ) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.get(*existing_key)
            }
            (_, _) => None,
        }
    }

    pub(crate) fn remove_state_with_id<T: 'static + Clone>(
        &mut self,
        current_id: &StorageKey,
    ) -> Option<T> {
        // /self.unseen_ids.remove(&current_id);
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(current_id)
            .copied()
            .unwrap_or_default();

        if key.is_null() {
            None
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            existing_secondary_map.remove(key)
        } else {
            None
        }
    }
}
