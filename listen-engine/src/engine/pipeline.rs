use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::engine::order::SwapOrder;
use crate::engine::payment::PaymentOrder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    PriceAbove { asset: String, value: f64 },
    PriceBelow { asset: String, value: f64 },
    Now { asset: String },
    And(Vec<Condition>),
    Or(Vec<Condition>),
    GTTimer(DateTime<Utc>),
    LTTimer(DateTime<Utc>),
    // // 价格突破和回调条件
    // PriceBreakout { asset: String, value: f64, period: i64 },
    // PriceRetracement { asset: String, high: f64, low: f64 },
    // // 交易量条件
    // VolumeSpike { asset: String, threshold: f64 },
    // // 技术指标条件
    // MovingAverageCross { asset: String, fast_period: i64, slow_period: i64 },
    // RSIThreshold { asset: String, period: i64, threshold: f64 },
    // // 转账相关条件
    // BalanceAbove { asset: String, amount: f64 },
    // GasFeeBelow { value: f64 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub triggered: bool,
    pub last_evaluated: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    Order(SwapOrder),
    Payment(PaymentOrder),
    Notification(Notification),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub id: Uuid,
    pub action: Action,
    pub conditions: Vec<Condition>,
    pub next_steps: Vec<Uuid>,
    pub status: Status,
    pub transaction_hash: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: Uuid,
    pub user_id: String,
    pub wallet_address: String,
    pub pubkey: String,
    pub current_steps: Vec<Uuid>,
    pub steps: HashMap<Uuid, PipelineStep>,
    pub status: Status,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    Pending,   // Not yet started
    Completed, // Successfully finished
    Failed,    // Execution failed
    Cancelled, // Manually cancelled
}
