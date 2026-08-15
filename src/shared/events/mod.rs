use chrono::{DateTime, Utc};
use std::any::Any;

use super::errors::PublishError;

pub trait DomainEvent: Any + Send + Sync {
    fn event_type(&self) -> &'static str;
    fn timestamp(&self) -> DateTime<Utc>;
    fn as_any(&self) -> &dyn Any;
}

pub type EventHandlerFn = Box<dyn Fn(&dyn DomainEvent) + Send + Sync>;

#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, events: Vec<&dyn DomainEvent>) -> Result<(), PublishError>;
}

pub trait EventHandler<E: DomainEvent>: Send + Sync {
    fn handle(&self, event: &E);
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

#[async_trait::async_trait]
impl EventPublisher for InMemoryEventDispatcher {
    async fn publish(&self, events: Vec<&dyn DomainEvent>) -> Result<(), PublishError> {
        let handlers = self.handlers.read().unwrap();
        for event in &events {
            if let Some(event_handlers) = handlers.get(event.event_type()) {
                for handler in event_handlers {
                    handler(*event);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
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

    #[tokio::test]
    async fn test_dispatcher_calls_handler() {
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

        dispatcher.publish(vec![&event]).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_dispatcher_multiple_handlers() {
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

        dispatcher.publish(vec![&event]).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 11);
    }

    #[tokio::test]
    async fn test_dispatcher_ignores_unregistered() {
        let dispatcher = InMemoryEventDispatcher::new();
        let event = TestEvent {
            timestamp: Utc::now(),
            value: 1,
        };
        dispatcher.publish(vec![&event]).await.unwrap();
    }

    #[tokio::test]
    async fn test_dispatcher_multiple_events() {
        let counter = Arc::new(AtomicUsize::new(0));
        let c = counter.clone();

        let dispatcher = InMemoryEventDispatcher::new();
        dispatcher.register_handler("TestEvent", move |_| {
            c.fetch_add(1, Ordering::SeqCst);
        });

        let e1 = TestEvent {
            timestamp: Utc::now(),
            value: 1,
        };
        let e2 = TestEvent {
            timestamp: Utc::now(),
            value: 2,
        };

        dispatcher.publish(vec![&e1, &e2]).await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
