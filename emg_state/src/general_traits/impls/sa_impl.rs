/*
 * @Author: Rais
 * @Date: 2023-03-28 17:50:43
 * @LastEditTime: 2023-03-29 11:26:48
 * @LastEditors: Rais
 * @Description:
 */

use tracing::debug_span;

use crate::{
    error::Error,
    general_fns::{
        global_engine_get_anchor_val, global_engine_get_anchor_val_with,
        try_global_engine_get_anchor_val,
    },
    general_struct::LocationEngineGet,
    general_traits::CloneStateAnchor,
    GStateStore, StateAnchor,
};

// ─────────────────────────────────────────────────────────────────────────────

impl<T> CloneStateAnchor<T> for StateAnchor<T>
where
    T: Clone + 'static,
{
    #[track_caller]
    fn get(&self) -> T {
        // let loc = LocationEngineGet::reset_new();
        // illicit::Layer::new().offer(loc).enter(|| {
        let _span = debug_span!("StateAnchor.get",ty = %std::any::type_name::<T>()).entered();
        global_engine_get_anchor_val(&self.0)
        // })
    }

    #[track_caller]
    fn try_get(&self) -> Result<T, Error> {
        let loc = LocationEngineGet::new();

        illicit::Layer::new().offer(loc).enter(|| {
            let _span =
                debug_span!("StateAnchor::try_get",ty = %std::any::type_name::<T>()).entered();
            try_global_engine_get_anchor_val(&self.0)
        })
    }

    fn get_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R {
        global_engine_get_anchor_val_with(&self.0, func)
    }
    fn store_get(&self, store: &GStateStore) -> T {
        store.engine_get(&self.0)
    }
    fn store_get_with<F: FnOnce(&T) -> R, R>(&self, store: &GStateStore, func: F) -> R {
        store.engine_get_with(&self.0, func)
    }
}
