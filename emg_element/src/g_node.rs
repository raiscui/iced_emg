/*
 * @Author: Rais
 * @Date: 2022-08-18 18:01:09
 * @LastEditTime: 2023-03-01 21:45:28
 * @LastEditors: Rais
 * @Description:
 */

// mod index;
use std::fmt::Write;
mod node_item_rc_sv;
use indented::indented;
pub use node_item_rc_sv::GraphMethods;
pub use node_item_rc_sv::{EventMatchsSa, GelType, GraphType, NItem, E, N};
use std::fmt::Display;

use emg_common::display::DictDisplay;
use emg_layout::EPath;
use emg_state::{CloneStateAnchor, Dict, StateAnchor};

const POOL_SIZE: usize = 1;

type PathDictAsSets = Dict<EPath, ()>;

#[derive(PartialEq, Eq)]
pub struct EmgNodeItem<NItem, GelType> {
    gel_item: NItem,
    //TODO maybe indexSet
    // paths_sa: StateAnchor<Vector<EPath>>, //NOTE: has self
    paths_sa: StateAnchor<PathDictAsSets>, //NOTE: has self
    // incoming_eix_sa: StateAnchor<NodeEdgeCollect>,
    // outgoing_eix_sa: StateAnchor<NodeEdgeCollect>,
    paths_view_gel: StateAnchor<Dict<EPath, GelType>>,
}

impl<NItem: Display, GelType: Display> Display for EmgNodeItem<NItem, GelType>
where
    GelType: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut members = String::new();
        // ─────────────────────────────────────────────────────────────
        writeln!(members, "gel: {} ,", &self.gel_item)?;
        // ─────────────────────────────────────────────────────────────

        let mut paths = String::new();
        self.paths_sa.get().iter().for_each(|(k, _)| {
            write!(paths, "{k},").unwrap();
        });

        writeln!(members, "paths_sa: \u{2693} {}", &paths)?;
        // paths_view_gel ─────────────────────────────────────────────────────────────
        {
            let ep_gels = self.paths_view_gel.get();
            let paths_view_gel = DictDisplay("paths_view_gel \u{2693}", ep_gels);
            writeln!(members, "{}", &paths_view_gel)?;
        }
        // ─────────────────────────────────────────────────────────────

        write!(f, "EmgNodeItem {{\n{}\n}}", indented(members))
    }
}

impl<NItem: std::fmt::Debug, GelType: std::fmt::Debug> std::fmt::Debug
    for EmgNodeItem<NItem, GelType>
where
    GelType: Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmgNodeItem")
            .field("gel_sa", &self.gel_item)
            .field("paths_sa", &self.paths_sa)
            .field("paths_view_gel", &self.paths_view_gel)
            // .field("paths_view_gel_sa", &self.paths_view_gel_sa)
            .finish()
    }
}

impl<NItem, GelType> Clone for EmgNodeItem<NItem, GelType>
where
    NItem: std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self {
            gel_item: self.gel_item.clone(),
            paths_sa: self.paths_sa.clone(),
            paths_view_gel: self.paths_view_gel.clone(),
        }
    }
}
