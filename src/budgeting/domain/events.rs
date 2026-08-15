use chrono::{DateTime, Utc};

use crate::shared::ids::BudgetID;
use crate::shared::ids::GoalID;
use crate::shared::money::Money;

#[derive(Debug)]
pub struct BudgetDefined {
    pub budget_id: BudgetID,
    pub category_id: crate::shared::ids::CategoryID,
    pub planned_amount: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for BudgetDefined {
    fn event_type(&self) -> &'static str {
        "BudgetDefined"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct BudgetExceeded {
    pub budget_id: BudgetID,
    pub category_id: crate::shared::ids::CategoryID,
    pub planned_amount: Money,
    pub spent_amount: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for BudgetExceeded {
    fn event_type(&self) -> &'static str {
        "BudgetExceeded"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct GoalContributed {
    pub goal_id: GoalID,
    pub amount: Money,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for GoalContributed {
    fn event_type(&self) -> &'static str {
        "GoalContributed"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Debug)]
pub struct GoalAchieved {
    pub goal_id: GoalID,
    pub timestamp: DateTime<Utc>,
}

impl crate::shared::events::DomainEvent for GoalAchieved {
    fn event_type(&self) -> &'static str {
        "GoalAchieved"
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
