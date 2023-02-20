/*
 * @Author: Rais
 * @Date: 2023-01-23 16:44:17
 * @LastEditTime: 2023-02-21 00:06:10
 * @LastEditors: Rais
 * @Description:
 */
/// An error that occurred while creating an application's graphical context.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested backend version is not supported.
    // #[error("the edge_index not exist for this node")]
    // EdgeIndexNotExistInNode,
    #[error("GraphError:{0}")]
    GraphError(#[from] emg::Error),
    // /// An error occurred in the context's internal backend
    // #[error("an error occurred in the context's internal backend")]
    // BackendError(String),
    // #[error("an error occurred")]
    // Error(#[from] Box<dyn std::error::Error + Send + Sync>),
}
