use emg_shaping::Shaping;
use emg_state::{general_traits::BiState, CloneState, StateVOA};

use crate::GElement;

use super::Video;

/*
 * @Author: Rais
 * @Date: 2023-04-18 17:24:16
 * @LastEditTime: 2023-04-19 14:10:44
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
                who.player.set_source_paused(*x);
            }
            (VideoController::Loop, _) => todo!(),
        }
        true
    }
}
impl Shaping<Video> for (VideoController, StateVOA<bool>) {
    fn shaping(&self, who: &mut Video) -> bool {
        match self {
            (VideoController::Pause, x) => who.player.paused().bi(*x),
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
