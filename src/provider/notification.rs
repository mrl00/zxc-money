use async_trait::async_trait;
use thiserror::Error;

/// Errors from the notification infrastructure.
#[derive(Debug, Error)]
pub enum NotificationError {
    /// The notification delivery failed.
    #[error("notification failed: {0}")]
    DeliveryFailed(String),
}

/// Port for sending notifications to the user.
///
/// Frontends must implement this trait to deliver push notifications,
/// emails, or any other alert mechanism.
///
/// # Example
///
/// A frontend might implement this with Firebase Cloud Messaging:
/// ```ignore
/// struct FirebaseNotification;
///
/// #[async_trait]
/// impl NotificationProvider for FirebaseNotification {
///     async fn notify(&self, title: &str, body: &str) -> Result<(), NotificationError> {
///         // send via FCM
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait NotificationProvider: Send + Sync {
    /// Sends a notification with the given title and body.
    async fn notify(&self, title: &str, body: &str) -> Result<(), NotificationError>;
}

/// Mock notification provider for testing.
///
/// Records all notifications sent for assertion in tests.
pub struct MockNotificationProvider {
    notifications: std::sync::Mutex<Vec<(String, String)>>,
}

impl MockNotificationProvider {
    /// Creates a new empty mock.
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Returns all notifications sent through this mock.
    pub fn notifications(&self) -> Vec<(String, String)> {
        self.notifications.lock().unwrap().clone()
    }
}

impl Default for MockNotificationProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationProvider for MockNotificationProvider {
    async fn notify(&self, title: &str, body: &str) -> Result<(), NotificationError> {
        self.notifications
            .lock()
            .unwrap()
            .push((title.to_string(), body.to_string()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_notification_records() {
        let mock = MockNotificationProvider::new();
        mock.notify("Alerta", "Fatura vence amanhã").await.unwrap();
        mock.notify("Meta", "Parabéns! Meta atingida")
            .await
            .unwrap();

        let notifs = mock.notifications();
        assert_eq!(notifs.len(), 2);
        assert_eq!(notifs[0].0, "Alerta");
        assert_eq!(notifs[0].1, "Fatura vence amanhã");
        assert_eq!(notifs[1].0, "Meta");
    }
}
