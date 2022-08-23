/*
 * @Author: Rais
 * @Date: 2022-08-23 15:57:49
 * @LastEditTime: 2022-08-23 15:57:55
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2022-08-09 20:48:53
 * @LastEditTime: 2022-08-10 15:19:53
 * @LastEditors: Rais
 * @Description:
 */

use std::error::Error;

/// The result of running an [`Application`].
///
/// [`Application`]: crate::Application
pub type Result = std::result::Result<(), Error>;
