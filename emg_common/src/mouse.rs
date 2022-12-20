/*
 * @Author: Rais
 * @Date: 2022-08-11 22:48:24
 * @LastEditTime: 2022-12-19 13:51:36
 * @LastEditors: Rais
 * @Description:
 */
//! Handle mouse events.
mod button;
mod event;
mod interaction;

pub use button::Button;
pub use event::*;
pub use interaction::Interaction;

#[cfg(test)]
mod test {
    use super::event::*;

    #[test]
    fn xx() {
        let x = CLICK;
        println!("{:?}", x);
    }
}
