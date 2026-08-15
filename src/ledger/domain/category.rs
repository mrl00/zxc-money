use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::ids::{CategoryID, TagID};

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

    pub fn with_parent(mut self, parent_id: CategoryID) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: TagID,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Tag {
    pub fn new(id: TagID, name: String) -> Self {
        Self {
            id,
            name,
            created_at: Utc::now(),
        }
    }
}
