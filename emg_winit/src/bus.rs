/*
 * @Author: Rais
 * @Date: 2023-04-19 19:05:58
 * @LastEditTime: 2023-04-19 19:05:59
 * @LastEditors: Rais
 * @Description:
 */

// pub struct Bus<Message> {
//     publish: winit::event_loop::EventLoopProxy<Message>,
// }

// unsafe impl<Message> Send for Bus<Message> {}

// impl<S, Message> Clone for Bus<S, Message> {
//     fn clone(&self) -> Self {
//         Self {
//             publish: self.publish.clone(),
//         }
//     }
// }

// impl<Message> Bus<Message>
// where
//     Message: 'static,
// {
//     pub fn new(publish: impl Fn(Message) + Send + 'static) -> Self {
//         Self {
//             publish: Arc::new(publish),
//         }
//     }

//     /// Publishes a new message for the [`Application`].
//     ///
//     /// [`Application`]: crate::Application
//     pub fn publish(&self, message: Message) {
//         (self.publish)(message);
//     }

//     /// Creates a new [`Bus`] that applies the given function to the messages
//     /// before publishing.
//     pub fn map<B>(&self, mapper: Arc<dyn Fn(B) -> Message + Send + 'static>) -> Bus<B>
//     where
//         B: 'static,
//     {
//         let publish = self.publish.clone();

//         // Bus {
//         //     publish: Arc::new(move |message| publish(mapper(message))),
//         // }
//         todo!()
//     }
// }
