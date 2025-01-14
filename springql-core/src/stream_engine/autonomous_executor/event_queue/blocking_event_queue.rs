// This file is part of https://github.com/SpringQL/SpringQL which is licensed under MIT OR Apache-2.0. See file LICENSE-MIT or LICENSE-APACHE for full license details.

use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{mpsc, Mutex, MutexGuard},
};

use super::{
    event::{Event, EventTag},
    EventPoll,
};

/// Event queue (message broker) for Choreography-based Saga pattern.
#[derive(Debug, Default)]
pub(in crate::stream_engine::autonomous_executor) struct BlockingEventQueue {
    subscribers_by_tag: Mutex<HashMap<EventTag, Subscribers>>,
}

impl BlockingEventQueue {
    /// Publish an event to queue (blocking). This function returns after all subscribers subscribe the event.
    pub(in crate::stream_engine::autonomous_executor) fn publish_blocking(&self, event: Event) {
        let tag = EventTag::from(&event);

        let subscribers_by_tag = self.lock();
        let opt_subscribers = subscribers_by_tag.get(&tag);

        if let Some(subscribers) = opt_subscribers {
            subscribers.push_all_blocking(event)
        }
    }

    /// Subscribe to an event tag and get event polling target.
    ///
    /// A worker need to call this method just 1 time if it needs an event tag.
    pub(in crate::stream_engine::autonomous_executor) fn subscribe(
        &self,
        tag: EventTag,
    ) -> EventPoll {
        let (sender, receiver) = mpsc::sync_channel(0); // rendezvous channel
        let event_push = EventPush::new(sender);
        let event_poll = EventPoll::new(receiver);

        let mut subscribers_by_tag = self.lock();

        // add new subscriber to self.subscribers
        match subscribers_by_tag.entry(tag) {
            Entry::Occupied(mut sub) => sub.get_mut().add(event_push),
            Entry::Vacant(v) => {
                let mut sub = Subscribers::default();
                sub.add(event_push);
                v.insert(sub);
            }
        }

        event_poll
    }

    fn lock(&self) -> MutexGuard<HashMap<EventTag, Subscribers>> {
        self.subscribers_by_tag
            .lock()
            .expect("BlockingEventQueue lock poisoned")
    }
}

#[derive(Debug, Default)]
struct Subscribers {
    event_push_list: Vec<EventPush>,
}

impl Subscribers {
    fn add(&mut self, event_push: EventPush) {
        self.event_push_list.push(event_push);
    }

    fn push_all_blocking(&self, event: Event) {
        for event_push in self.event_push_list.iter() {
            event_push.push_blocking(event.clone());
        }
    }
}

#[derive(Debug, new)]
struct EventPush {
    sender: mpsc::SyncSender<Event>,
}

impl EventPush {
    fn push_blocking(&self, event: Event) {
        let event_tag = EventTag::from(&event);
        self.sender
            .send(event)
            .unwrap_or_else(|_| panic!("failed to send event to subscriber: {:?}", event_tag));
    }
}
