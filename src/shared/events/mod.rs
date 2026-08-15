use chrono::{DateTime, Utc};
use std::any::Any;

pub trait DomainEvent: Any + Send + Sync {
    fn event_type(&self) -> &'static str;
    fn timestamp(&self) -> DateTime<Utc>;
    fn as_any(&self) -> &dyn Any;
}

pub type EventHandlerFn = Box<dyn Fn(&dyn DomainEvent) + Send + Sync>;

pub trait EventPublisher: Send + Sync {
    fn publish(&self, event: &dyn DomainEvent);
}

pub struct InMemoryEventDispatcher {
    handlers: std::sync::RwLock<std::collections::HashMap<&'static str, Vec<EventHandlerFn>>>,
}

impl InMemoryEventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub fn register_handler<F>(&self, event_type: &'static str, handler: F)
    where
        F: Fn(&dyn DomainEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().unwrap();
        handlers
            .entry(event_type)
            .or_default()
            .push(Box::new(handler));
    }
}

impl Default for InMemoryEventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventPublisher for InMemoryEventDispatcher {
    fn publish(&self, event: &dyn DomainEvent) {
        let handlers = self.handlers.read().unwrap();
        if let Some(event_handlers) = handlers.get(event.event_type()) {
            for handler in event_handlers {
                handler(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestEvent {
        timestamp: DateTime<Utc>,
        #[allow(dead_code)]
        value: usize,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }

        fn timestamp(&self) -> DateTime<Utc> {
            self.timestamp
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_dispatcher_calls_handler() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let dispatcher = InMemoryEventDispatcher::new();
        dispatcher.register_handler("TestEvent", move |_event| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = TestEvent {
            timestamp: Utc::now(),
            value: 42,
        };

        dispatcher.publish(&event);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_dispatcher_multiple_handlers() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c1 = counter.clone();
        let c2 = counter.clone();

        let dispatcher = InMemoryEventDispatcher::new();
        dispatcher.register_handler("TestEvent", move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        dispatcher.register_handler("TestEvent", move |_| {
            c2.fetch_add(10, Ordering::SeqCst);
        });

        let event = TestEvent {
            timestamp: Utc::now(),
            value: 1,
        };

        dispatcher.publish(&event);
        assert_eq!(counter.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn test_dispatcher_ignores_unregistered() {
        let dispatcher = InMemoryEventDispatcher::new();
        let event = TestEvent {
            timestamp: Utc::now(),
            value: 1,
        };
        dispatcher.publish(&event);
    }

    use std::sync::Arc;
}
