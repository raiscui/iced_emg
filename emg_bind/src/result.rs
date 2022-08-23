/*
 * @Author: Rais
 * @Date: 2022-08-09 20:48:53
 * @LastEditTime: 2022-08-10 15:19:53
 * @LastEditors: Rais
 * @Description:
 */
use crate::Error;

/// The result of running an [`Application`].
///
/// [`Application`]: crate::Application
pub type Result = std::result::Result<(), Error>;
