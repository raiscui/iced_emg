/*
 * @Author: Rais
 * @Date: 2023-01-30 10:14:43
 * @LastEditTime: 2023-02-20 22:41:35
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2023-01-23 16:44:17
 * @LastEditTime: 2023-01-23 18:28:21
 * @LastEditors: Rais
 * @Description:
 */
/// An error that occurred while creating an application's graphical context.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested backend version is not supported.
    #[error("the edge_index(key) not exist in edges map")]
    CanNotGetEdge,

    #[error("can't get node by {nix}")]
    CanNotGetNode { nix: String },

    // /// An error occurred in the context's internal backend
    // #[error("an error occurred in the context's internal backend")]
    // BackendError(String),
    // #[error("an error occurred")]
    // Error(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("error from emg_state :{0}")]
    EmgState(#[from] emg_state::error::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
