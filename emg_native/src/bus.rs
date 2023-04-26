/*
 * @Author: Rais
 * @Date: 2022-08-23 00:21:57
 * @LastEditTime: 2023-04-22 00:01:21
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

use std::sync::{Arc, Mutex, RwLock};

/// A publisher of messages.
///
/// It can be used to route messages back to the [`Application`].
///
/// [`Application`]: crate::Application

pub struct Bus<Message> {
    publish: Arc<Mutex<dyn Fn(Message) + Send>>,
}

impl<Message> std::fmt::Debug for Bus<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bus").finish()
    }
}

impl<Message> Clone for Bus<Message> {
    fn clone(&self) -> Self {
        Self {
            publish: self.publish.clone(),
        }
    }
}

impl<Message: 'static> Bus<Message> {
    pub fn new(publish: impl Fn(Message) + Send + 'static) -> Self {
        Bus {
            publish: Arc::new(Mutex::new(publish)) as Arc<Mutex<dyn Fn(Message) + Send>>,
        }
    }

    pub fn publish(&self, message: Message) {
        self.publish.lock().unwrap()(message)
    }

    pub fn map<B>(&self, mapper: Arc<Mutex<dyn Fn(B) -> Message + Send + 'static>>) -> Bus<B>
    where
        B: 'static,
    {
        let self_publish = self.publish.clone();

        let publish_fn = move |message: B| {
            let message = mapper.lock().unwrap()(message);
            let self_publish_fn = self_publish.lock().unwrap();
            self_publish_fn(message);
        };
        let publish = Arc::new(Mutex::new(publish_fn)) as Arc<Mutex<dyn Fn(B) + Send + 'static>>;

        Bus { publish }
    }
}

unsafe impl<Message> Sync for Bus<Message> {}

unsafe impl<Message: Send> Send for Bus<Message> {}
