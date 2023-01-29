use crate::graph::Direction;

/*
 * @Author: Rais
 * @Date: 2020-12-26 12:03:39
 * @LastEditTime: 2020-12-28 17:12:35
 * @LastEditors: Rais
 * @Description:
 */

pub trait HasDir {
    fn dir(&self) -> Direction;
}
