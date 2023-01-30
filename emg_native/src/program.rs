/*
 * @Author: Rais
 * @Date: 2022-08-12 14:43:52
 * @LastEditTime: 2023-01-30 18:40:53
 * @LastEditors: Rais
 * @Description:
 */
//! Build interactive programs using The Elm Architecture.

// use crate::renderer::SceneCtx;
use crate::Command;
/// The core of a user interface application following The Elm Architecture.
pub trait Program: Sized {
    /// The graphics backend to use to draw the [`Program`].
    // type WhoImplSceneCtx: SceneCtx + Clone + PartialEq;

    /// The type of __messages__ your [`Program`] will produce.
    type Message: std::fmt::Debug + Send;
    type GraphType;
    type Orders;
    type RcRefCellGraphType;

    // type GElement: Widget<Self::Message, Self::ImplRenderContext>;

    /// Handles a __message__ and updates the state of the [`Program`].
    ///
    /// This is where you define your __update logic__. All the __messages__,
    /// produced by either user interactions or commands, will be handled by
    /// this method.
    ///
    /// Any [`Command`] returned will be executed immediately in the
    /// background by shells.
    fn update(
        &mut self,
        graph: Self::RcRefCellGraphType,
        orders: &Self::Orders,
        message: Self::Message,
    ) -> Command<Self::Message>;

    // /// Returns the widgets to display in the [`Program`].
    // ///
    // /// These widgets can produce __messages__ based on user interaction.
    // fn view(&mut self) -> Self::GElement;
}
