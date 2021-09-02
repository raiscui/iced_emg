use std::{convert::TryFrom, rc::Rc};

/*
 * @Author: Rais
 * @Date: 2021-05-07 15:15:12
 * @LastEditTime: 2021-09-01 09:23:58
 * @LastEditors: Rais
 * @Description:
 */
use crate::emg_runtime::NodeBuilderWidget;
use crate::GElement;

impl<'a, Message> TryFrom<GElement<'a, Message>> for NodeBuilderWidget<'a, Message>
where
    Message: 'static + Clone,
{
    type Error = GElement<'a, Message>;

    fn try_from(gel: GElement<'a, Message>) -> Result<Self, Self::Error> {
        use match_any::match_any;
        use GElement::{Button_, Layer_, Text_};
        match_any! (gel,
            Layer_( x) |Button_(x)| Text_(x)=> {
                Ok(NodeBuilderWidget::new(Rc::new(x)))
            },
            _=>Err(gel)
        )
    }
}
