use std::iter::Sum;

/*
 * @Author: Rais
 * @Date: 2023-04-21 15:27:05
 * @LastEditTime: 2023-04-21 19:17:19
 * @LastEditors: Rais
 * @Description:
 */
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum RenderLoopCommand {
    // Wait,
    Schedule,
    Immediately,
    #[default]
    Nothing,
}

impl Sum<RenderLoopCommand> for Option<RenderLoopCommand> {
    fn sum<I: Iterator<Item = RenderLoopCommand>>(iter: I) -> Self {
        iter.reduce(|acc, x| acc + x)
    }
}

impl std::ops::Add for RenderLoopCommand {
    type Output = RenderLoopCommand;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (_, RenderLoopCommand::Immediately) | (RenderLoopCommand::Immediately, _) => {
                RenderLoopCommand::Immediately
            }
            (_, RenderLoopCommand::Schedule) | (RenderLoopCommand::Schedule, _) => {
                RenderLoopCommand::Schedule
            }
            (x, RenderLoopCommand::Nothing) | (RenderLoopCommand::Nothing, x) => x,
        }
    }
}
