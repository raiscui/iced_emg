/*
 * @Author: Rais
 * @Date: 2022-08-22 22:42:54
 * @LastEditTime: 2023-01-31 21:21:35
 * @LastEditors: Rais
 * @Description:
 */
#![allow(clippy::module_name_repetitions)]

pub use emg_element::{graph_edit, node_ref, widget::*, EventCallback, EventMessage};
// ────────────────────────────────────────────────────────────────────────────────

pub type GelType<Message> = emg_element::GelType<Message>;

pub type GraphType<Message> = emg_element::GraphType<Message>;

pub type GTreeBuilderElement<Message> = emg_element::GTreeBuilderElement<Message>;

pub type GElement<Message> = emg_element::GElement<Message>;
