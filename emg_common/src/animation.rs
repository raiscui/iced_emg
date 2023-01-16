use std::time::Duration;

use num_traits::{NumCast, ToPrimitive};

/*
 * @Author: Rais
 * @Date: 2023-01-13 17:02:38
 * @LastEditTime: 2023-01-13 22:16:58
 * @LastEditors: Rais
 * @Description:
 */

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Tick(pub Duration);

impl Tick {
    #[must_use]
    pub const fn subsec_millis(&self) -> u32 {
        self.0.subsec_millis()
    }
    #[must_use]
    pub fn new(millisecond: impl NumCast) -> Self {
        Self(Duration::from_micros(
            (millisecond.to_f64().unwrap() * 1000.0)
                .trunc()
                .to_u64()
                .unwrap(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::Tick;

    #[test]
    fn t() {
        let x = Tick::new(22);
        println!("{:?}", x)
    }
}
