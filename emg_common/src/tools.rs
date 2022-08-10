use im_rc::Vector;

/*
 * @Author: Rais
 * @Date: 2022-07-19 22:11:46
 * @LastEditTime: 2022-07-19 22:42:16
 * @LastEditors: Rais
 * @Description:
 */
pub struct VectorDisp<T>(pub Vector<T>);
impl<T> std::fmt::Display for VectorDisp<T>
where
    T: std::fmt::Display + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for one in self.0.iter() {
            writeln!(f, "{},", one)?;
        }
        writeln!(f, "]")
    }
}
pub struct VecDisp<T>(pub Vec<T>);
impl<T> std::fmt::Display for VecDisp<T>
where
    T: std::fmt::Display + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[")?;
        for one in self.0.iter() {
            writeln!(f, "{},", one)?;
        }
        writeln!(f, "]")
    }
}
