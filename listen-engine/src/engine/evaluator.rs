use super::pipeline::{Condition, ConditionType};
use crate::engine::EngineError;
use std::collections::HashMap;

pub struct Evaluator;

#[derive(Debug, thiserror::Error)]
pub enum EvaluatorError {
    #[error("[Evaluator] Failed to evaluate conditions: {0}")]
    EvaluateConditionsError(String),

    #[error("[Evaluator] Failed to evaluate price condition: {0}")]
    PriceEvaluationError(String),

    #[error("[Evaluator] Missing price data for asset: {0}")]
    MissingPriceData(String),

    #[error("[Evaluator] Invalid condition type: {0}")]
    InvalidConditionType(String),
}

impl From<EvaluatorError> for EngineError {
    fn from(err: EvaluatorError) -> Self {
        EngineError::EvaluatePipelineError(err)
    }
}

impl Evaluator {
    pub fn evaluate_conditions(
        conditions: &[Condition],
        prices: &HashMap<String, f64>,
    ) -> Result<bool, EvaluatorError> {
        conditions.iter().try_fold(true, |acc, c| {
            Ok(acc && Self::evaluate_condition(c, prices)?)
        })
    }

    fn evaluate_condition(
        condition: &Condition,
        prices: &HashMap<String, f64>,
    ) -> Result<bool, EvaluatorError> {
        match &condition.condition_type {
            ConditionType::PriceAbove { asset, value } => {
                let price = prices
                    .get(asset)
                    .ok_or_else(|| EvaluatorError::MissingPriceData(asset.clone()))?;
                Ok(price >= value)
            }
            ConditionType::PriceBelow { asset, value } => {
                let price = prices
                    .get(asset)
                    .ok_or_else(|| EvaluatorError::MissingPriceData(asset.clone()))?;
                Ok(price <= value)
            }
            ConditionType::And(sub) => sub.iter().try_fold(true, |acc, c| {
                Ok(acc && Self::evaluate_condition(c, prices)?)
            }),
            ConditionType::Or(sub) => sub.iter().try_fold(false, |acc, c| {
                Ok(acc || Self::evaluate_condition(c, prices)?)
            }),
            ConditionType::Now { .. } => Ok(true),
            ConditionType::GTTimer(date_time) => Ok(chrono::Utc::now() > *date_time),
            ConditionType::LTTimer(date_time) => Ok(chrono::Utc::now() < *date_time),

            // 价格突破和回调条件
            // ConditionType::PriceBreakout { asset, value, period } => {
            //     let price = prices
            //         .get(asset)
            //         .ok_or_else(|| EvaluatorError::MissingPriceData(asset.clone()))?;
            //     // TODO: 需要实现获取历史价格数据的功能
            //     Ok(price > value)
            // },
            // ConditionType::PriceRetracement { asset, high, low } => {
            //     let price = prices
            //         .get(asset)
            //         .ok_or_else(|| EvaluatorError::MissingPriceData(asset.clone()))?;
            //     Ok(price >= low && price <= high)
            // },

            // // 交易量条件
            // ConditionType::VolumeSpike { asset, threshold } => {
            //     // TODO: 需要实现获取交易量数据的功能
            //     Ok(false)
            // },

            // // 技术指标条件
            // ConditionType::MovingAverageCross { asset, fast_period, slow_period } => {
            //     // TODO: 需要实现计算移动平均线的功能
            //     Ok(false)
            // },
            // ConditionType::RSIThreshold { asset, period, threshold } => {
            //     // TODO: 需要实现计算RSI的功能
            //     Ok(false)
            // },

            // // 转账相关条件
            // ConditionType::BalanceAbove { asset, amount } => {
            //     // TODO: 需要实现获取余额的功能
            //     Ok(false)
            // },
            // ConditionType::GasFeeBelow { value } => {
            //     // TODO: 需要实现获取gas费用的功能
            //     Ok(false)
            // }
        }
    }
}
