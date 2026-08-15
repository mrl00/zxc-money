use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::investment::domain::position::Position;
use crate::shared::ids::PortfolioID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    pub id: PortfolioID,
    pub positions: Vec<Position>,
    pub created_at: DateTime<Utc>,
}
