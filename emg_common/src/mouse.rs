/*
 * @Author: Rais
 * @Date: 2022-08-11 22:48:24
 * @LastEditTime: 2023-01-13 12:36:30
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
