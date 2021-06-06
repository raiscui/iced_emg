mod props;
use im::Vector;
use tracing::error;

use super::define::Property;

/*
 * @Author: Rais
 * @Date: 2021-05-30 22:13:24
 * @LastEditTime: 2021-06-02 11:44:32
 * @LastEditors: Rais
 * @Description:
 */

fn list_find_dup<T: Eq>(eq_fn: impl Fn(&T, &T) -> bool, list: &[T]) -> Vec<&T> {
    list.iter()
        .fold((None, Vec::new()), |mut acc, t| match acc.0 {
            Some(t0) if eq_fn(t0, t) => {
                acc.1.push(t);
                acc
            }

            _ => {
                acc.0 = Some(t);
                acc
            }
        })
        .1
}
