fn xx() {
    let children = vec![];

    let edges = vec![];
    let g_tree_builder_element =
        Checkbox::new(false, "abcd", |_| Message::IncrementPressed).tree_build_in_topo(&children);
    g_tree_builder_element.with_id_edge_children(IdStr::new_inline("root"), edges, children)
}
