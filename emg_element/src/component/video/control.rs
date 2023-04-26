use emg_shaping::Shaping;
use emg_state::{general_traits::BiState, topo, CloneState, StateVOA};

use crate::GElement;

use super::Video;

/*
 * @Author: Rais
 * @Date: 2023-04-18 17:24:16
 * @LastEditTime: 2023-04-26 11:00:24
 * @LastEditors: Rais
 * @Description:
 */
#[derive(Debug, PartialEq, Eq)]
pub enum VideoController {
    Pause,
    Loop,
}

impl Shaping<Video> for (VideoController, bool) {
    fn shaping(&self, who: &mut Video) -> bool {
        match self {
            (VideoController::Pause, x) => {
                who.player.paused().set(*x);
                who.player.set_paused(*x).ok();
            }
            (VideoController::Loop, _) => todo!(),
        }
        true
    }
}
impl Shaping<Video> for (VideoController, StateVOA<bool>) {
    #[topo::nested]
    fn shaping(&self, who: &mut Video) -> bool {
        match self {
            // (VideoController::Pause, x) => who.player.paused().bi_in_topo(*x),
            (VideoController::Pause, x) => {
                x.bi_in_topo(who.player.paused());
                who.player.set_paused(x.get_out_val()).ok();
            }
            (VideoController::Loop, _) => todo!(),
        };
        true
    }
}
impl Shaping<Video> for (VideoController, StateVOA<i32>) {
    fn shaping(&self, _who: &mut Video) -> bool {
        todo!()
    }
}

impl<Message> Shaping<GElement<Message>> for (VideoController, bool) {
    fn shaping(&self, who: &mut GElement<Message>) -> bool {
        println!("controller shaping video");
        match (self, who) {
            ((VideoController::Pause, x), GElement::Video_(video)) => {
                // video.player.paused().set(*x);
                // video.player.set_source_paused(*x);
                self.shaping(video);
            }
            _ => (),
        }
        true
    }
}
