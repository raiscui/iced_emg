use std::{rc::Rc, time::Duration};

use emg_core::GenericSize;
use emg_state::StateAnchor;
use im::Vector;

/*
 * @Author: Rais
 * @Date: 2021-05-30 22:02:12
 * @LastEditTime: 2021-06-02 11:53:55
 * @LastEditors: Rais
 * @Description:
 */
#[derive(Copy, Clone, Debug)]
pub(crate) enum PropertyType {
    SA,
}
#[derive(Clone, Debug)]
pub struct Property {
    name: Rc<String>,
    t: PropertyType,
    value: StateAnchor<GenericSize>,
}

impl Property {
    /// Get a reference to the property's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get a reference to the property's t.
    pub fn value_type(&self) -> &PropertyType {
        &self.t
    }
}

// enum StepType {
//     _Step,
//     To,
//     ToWith,
//     Set,
//     Wait,
//     Send,
//     Repeat,
//     Loop,
// }
pub type StepTimeVector<Message> = Vector<(Duration, Vector<Step<Message>>)>;

#[allow(clippy::pub_enum_variant_names)]
#[derive(Clone, Debug)]
pub enum Step<Message>
where
    Message: Clone,
{
    _Step,
    To(Vector<Property>),
    ToWith(Vector<Property>),
    Set(Vector<Property>),
    Wait(Duration),
    Send(Message),
    Repeat(u32, Vector<Step<Message>>),
    Loop(Vector<Step<Message>>),
}
