/*
 * @Author: Rais
 * @Date: 2021-03-08 16:50:04
 * @LastEditTime: 2021-03-08 16:52:47
 * @LastEditors: Rais
 * @Description:
 */

use crate::{
    runtime::{Element, Text},
    Layer, RefreshFor,
};
use match_any::match_any;
use std::{convert::TryFrom, rc::Rc};
use strum_macros::Display;

pub use GElement::*;

#[derive(Clone, Display)]
pub enum GElement<'a, Message> {
    Element_(Element<'a, Message>),
    Layer_(Layer<'a, Message>),
    Text_(Text),
    Refresher_(Rc<dyn RefreshFor<GElement<'a, Message>> + 'a>),
}

impl<'a, Message: std::fmt::Debug> std::fmt::Debug for GElement<'a, Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layer_(l) => f.debug_tuple("GElement::GContainer").field(l).finish(),
            Text_(t) => f.debug_tuple("GElement::Text").field(t).finish(),
            Refresher_(_) => f
                .debug_tuple("GElement::GUpdater(Rc<dyn RtUpdateFor<GElement<'a, Message>>>)")
                .finish(),
            Element_(_) => f
                .debug_tuple("GElement::Element_(Element<'a, Message>)")
                .finish(),
        }
    }
}

impl<'a, Message> TryFrom<GElement<'a, Message>> for Element<'a, Message>
where
    Message: 'static + Clone,
{
    type Error = ();

    #[allow(clippy::useless_conversion)]
    fn try_from(ge: GElement<'a, Message>) -> Result<Self, Self::Error> {
        // match ge {
        //     Layer_(l) => Ok(l.into()),
        //     Text_(t) => Ok(t.into()),
        //     Refresher_(_) => Err(()),
        // }
        match_any! (ge,
            Layer_(x)|Text_(x)|Element_(x) => Ok(x.into()),
            Refresher_(_)=>Err(())
        )
    }
}
