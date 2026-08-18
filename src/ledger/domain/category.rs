use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{CategoryID, TagID};

/// A category for classifying transactions (e.g. "Food", "Salary").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: CategoryID,
    pub name: String,
    pub parent_id: Option<CategoryID>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Category {
    /// Creates a new category with the given `id` and `name`.
    pub fn new(id: CategoryID, name: String) -> Self {
        Self {
            id,
            name,
            parent_id: None,
            icon: None,
            color: None,
            created_at: Utc::now(),
        }
    }

    /// Sets the parent category, creating a hierarchy.
    pub fn with_parent(mut self, parent_id: CategoryID) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Sets an icon identifier for the category.
    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets a display color for the category.
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

/// A label that can be applied to transactions for custom grouping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagID,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Tag {
    /// Creates a new tag with the given `id` and `name`.
    pub fn new(id: TagID, name: String) -> Self {
        Self {
            id,
            name,
            created_at: Utc::now(),
        }
    }
}
