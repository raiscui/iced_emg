/*
 * @Author: Rais
 * @Date: 2023-02-10 00:13:39
 * @LastEditTime: 2023-02-10 00:13:41
 * @LastEditors: Rais
 * @Description:
 */
fn xx() {
    {
        let id = IdStr::new_inline("root").into();
        let edges = vec![];
        let children = vec![];
        Checkbox::new(false, "abcd", |_| Message::IncrementPressed)
            .tree_init(&id, &edges, &children)
            .with_id_edge_children(id, Some(edges), Some(children))
    }
}
