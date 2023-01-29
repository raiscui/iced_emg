mod graph;
mod macros;
pub use graph::edge_index;
pub use graph::edge_index_no_source;
pub use graph::node_index;
pub use graph::Direction;
pub use graph::Edge;
pub use graph::EdgeCollect;
pub use graph::EdgeIndex;
pub use graph::Graph;
pub use graph::Incoming;
pub use graph::Node;
pub use graph::NodeCollect;
pub use graph::NodeEdgesIter;
pub use graph::NodeIndex;
pub use graph::Outgoing;

use im_rc as im;
//CHECK graph 因为含有 StateVar 所以没有实现 eq ,
//CHECK 因为只有clone才会eq,任意两个graph对比,虽然值一样但是不会eq,除非使用 deep_eq_use_format_str, deep_eq_use_format_str是对比StateVar的get获得的Debug字符串,
//CHECK 因为简单的eq 会对比StateVar,而StateVar 的 eq原则是 虽然值不同但是只要不是同一个,就不eq(但是clone 的话是 eq的)
