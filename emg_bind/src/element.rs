/*
 * @Author: Rais
 * @Date: 2022-08-22 22:42:54
 * @LastEditTime: 2022-09-08 16:08:56
 * @LastEditors: Rais
 * @Description:
 */
#![allow(clippy::module_name_repetitions)]

pub use emg_element::{node_ref, EventCallback, EventMessage};
// ────────────────────────────────────────────────────────────────────────────────

pub type GelType<Message, RenderContext = crate::renderer::RenderCtx> =
    emg_element::GelType<Message, RenderContext>;

pub type GraphType<Message, RenderContext = crate::renderer::RenderCtx> =
    emg_element::GraphType<Message, RenderContext>;

pub type GTreeBuilderElement<Message, RenderContext = crate::renderer::RenderCtx> =
    emg_element::GTreeBuilderElement<Message, RenderContext>;

pub type GElement<Message, RenderContext = crate::renderer::RenderCtx> =
    emg_element::GElement<Message, RenderContext>;
