/*
 * @Author: Rais
 * @Date: 2022-09-05 20:56:05
 * @LastEditTime: 2022-09-05 20:56:08
 * @LastEditors: Rais
 * @Description:
 */

use super::EventNode;

use emg_common::Vector;

use super::EventNameString;

use emg_state::Dict;

pub struct EventBuilder<Message> {
    pub(crate) event_callbacks: Dict<EventNameString, Vector<EventNode<Message>>>,
}

impl<Message> std::fmt::Debug for EventBuilder<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBuilder")
            .field("event_callbacks", &self.event_callbacks)
            .finish()
    }
}

impl<Message> Clone for EventBuilder<Message> {
    fn clone(&self) -> Self {
        Self {
            event_callbacks: self.event_callbacks.clone(),
        }
    }
}

impl<Message> PartialEq for EventBuilder<Message> {
    fn eq(&self, other: &Self) -> bool {
        self.event_callbacks == other.event_callbacks
    }
}

impl<Message> EventBuilder<Message> {
    pub fn new() -> Self {
        Self {
            event_callbacks: Dict::new(),
        }
    }
}

impl<Message> EventBuilder<Message> {
    pub(crate) fn register_listener(
        &mut self,
        event_name: EventNameString,
        event_node: EventNode<Message>,
    ) {
        let entry = self.event_callbacks.entry(event_name);
        let v = entry.or_insert_with(|| Vector::new());
        v.push_back(event_node);
    }
    // fn register_event(
    //     mut self,
    //     event_name: EventNameString,
    //     event_node: EventNode<Message>,
    // ) -> Self {
    //     self.event_callbacks = self.event_callbacks.update_with(
    //         event_name,
    //         vector![event_node],
    //         |mut old_v, new_v| {
    //             old_v.append(new_v);
    //             old_v
    //         },
    //     );
    //     self
    // }
}
