/*
 * @Author: Rais
 * @Date: 2022-08-23 00:21:57
 * @LastEditTime: 2023-01-15 14:18:13
 * @LastEditors: Rais
 * @Description:
 */
/*
 * @Author: Rais
 * @Date: 2021-09-01 09:25:55
 * @LastEditTime: 2021-09-02 12:51:29
 * @LastEditors: Rais
 * @Description:
 */
use emg_futures::futures::channel::mpsc;
use std::rc::Rc;

/// A publisher of messages.
///
/// It can be used to route messages back to the [`Application`].
///
/// [`Application`]: crate::Application
#[allow(missing_debug_implementations)]

pub struct Bus<Message> {
    publish: Rc<dyn Fn(Message)>,
}

impl<Message> Clone for Bus<Message> {
    fn clone(&self) -> Self {
        Self {
            publish: self.publish.clone(),
        }
    }
}

impl<Message> Bus<Message>
where
    Message: 'static,
{
    pub fn new(publish: impl Fn(Message) + 'static) -> Self {
        Self {
            publish: Rc::new(publish),
        }
    }

    /// Publishes a new message for the [`Application`].
    ///
    /// [`Application`]: crate::Application
    pub fn publish(&self, message: Message) {
        (self.publish)(message);
    }

    /// Creates a new [`Bus`] that applies the given function to the messages
    /// before publishing.
    pub fn map<B>(&self, mapper: Rc<dyn Fn(B) -> Message>) -> Bus<B>
    where
        B: 'static,
    {
        let publish = self.publish.clone();

        Bus {
            publish: Rc::new(move |message| publish(mapper(message))),
        }
    }
}
