/*
 * @Author: Rais
 * @Date: 2021-06-02 12:49:55
 * @LastEditTime: 2022-01-25 21:52:12
 * @LastEditors: Rais
 * @Description:
 */
pub fn list_find_dup<T: Eq>(eq_fn: impl Fn(&T, &T) -> bool, list: &[T]) -> Vec<&T> {
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
