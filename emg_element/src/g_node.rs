/*
 * @Author: Rais
 * @Date: 2022-08-18 18:01:09
 * @LastEditTime: 2023-01-25 22:27:16
 * @LastEditors: Rais
 * @Description:
 */

// mod index;
use std::fmt::Write;
mod node_item_rc_sv;
use indented::{indented, indented_with};
pub use node_item_rc_sv::GraphMethods;
pub use node_item_rc_sv::{EventMatchsSa, GelType, GraphType, NItem, E, N};
use std::fmt::Display;

use emg_common::{display::DictDisplay, IdStr};
use emg_layout::EPath;
use emg_state::{CloneStateAnchor, Dict, StateAnchor};

const POOL_SIZE: usize = 1;

// pub type GelType<Message> = GElement<Message>;
// pub type NItem<Message> = StateAnchor<GelType<Message>>;
// pub type N<Message, Ix> = EmgNodeItem<NItem<Message>, GelType<Message>, Ix>;
// pub type E<Ix> = EmgEdgeItem<Ix>;
// pub type GraphType<Message, Ix = IdStr> = Graph<N<Message, Ix>, E<Ix>, Ix>;

type PathDict<Ix> = Dict<EPath<Ix>, ()>;

// type CurrentPathChildrenEixGElSA<Message> =
// StateAnchor<(EdgeIndex<IdStr>, Either<GelType<Message>, GelType<Message>>)>;

// type GElement<Message> = Either<GelType<Message>, GelType<Message>>;

pub struct EmgNodeItem<NItem, GelType, Ix = IdStr>
where
    // Message: 'static + Clone + std::cmp::PartialEq,
    // Ix: std::clone::Clone + std::hash::Hash + std::cmp::Eq + std::default::Default,
    Ix: std::clone::Clone,
    // Dict<EPath<Ix>, EmgNodeItem<Message, Ix>>: PartialEq,
{
    gel_sa: NItem,
    //TODO maybe indexSet
    // paths_sa: StateAnchor<Vector<EPath<Ix>>>, //NOTE: has self
    paths_sa: StateAnchor<PathDict<Ix>>, //NOTE: has self
    // incoming_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
    // outgoing_eix_sa: StateAnchor<NodeEdgeCollect<Ix>>,
    paths_view_gel: StateAnchor<Dict<EPath<Ix>, GelType>>,
}

impl<NItem: Display, GelType: Display, Ix: Display> Display for EmgNodeItem<NItem, GelType, Ix>
where
    Ix: std::clone::Clone + Ord + 'static,
    GelType: 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut members = String::new();
        // ─────────────────────────────────────────────────────────────
        writeln!(members, "gel: {} ,", &self.gel_sa)?;
        // ─────────────────────────────────────────────────────────────

        let mut paths = String::new();
        self.paths_sa.get().iter().for_each(|(k, _)| {
            write!(paths, "{k},").unwrap();
        });

        writeln!(members, "paths_sa: {}", &paths)?;
        // paths_view_gel ─────────────────────────────────────────────────────────────
        {
            let ep_gels = self.paths_view_gel.get();
            let paths_view_gel = DictDisplay("paths_view_gel", ep_gels);
            writeln!(members, "{}", &paths_view_gel)?;
        }
        // ─────────────────────────────────────────────────────────────

        write!(
            f,
            "EmgNodeItem {{\n{}\n}}",
            indented_with(members, " ".repeat("EmgNodeItem {".len()).as_str())
        )
    }
}

impl<NItem: std::fmt::Debug, GelType: std::fmt::Debug, Ix: std::fmt::Debug> std::fmt::Debug
    for EmgNodeItem<NItem, GelType, Ix>
where
    // Message: 'static + Clone + std::cmp::PartialEq,
    Ix: Ord + Clone + 'static,
    GelType: Clone + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmgNodeItem")
            .field("gel_sa", &self.gel_sa)
            .field("paths_sa", &self.paths_sa)
            .field("paths_view_gel", &self.paths_view_gel)
            // .field("paths_view_gel_sa", &self.paths_view_gel_sa)
            .finish()
    }
}

impl<NItem, GelType, Ix> Clone for EmgNodeItem<NItem, GelType, Ix>
where
    // Message: 'static + Clone + std::cmp::PartialEq,
    Ix: std::clone::Clone,
    NItem: std::clone::Clone,
{
    fn clone(&self) -> Self {
        Self {
            gel_sa: self.gel_sa.clone(),
            paths_sa: self.paths_sa.clone(),
            paths_view_gel: self.paths_view_gel.clone(),
        }
    }
}
