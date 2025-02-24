pub mod api;
pub mod bridge;
pub mod collect;
pub mod constants;
pub mod error;
pub mod evaluate;
pub mod evaluator;
pub mod execute;
pub mod order;
pub mod payment;
pub mod pipeline;

use crate::engine::error::EngineError;
use crate::redis::client::{make_redis_client, RedisClient};
use crate::redis::subscriber::{make_redis_subscriber, PriceUpdate, RedisSubscriber};
use anyhow::Result;
use dashmap::DashMap;
use metrics::{counter, histogram};
use privy::config::PrivyConfig;
use privy::Privy;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::sync::Notify;
use tokio::sync::RwLock;

use self::pipeline::{Pipeline, Status};
use crate::server::EngineMessage;

pub struct Engine {
    pub redis: Arc<RedisClient>,
    pub redis_sub: Arc<RedisSubscriber>,
    pub privy: Arc<Privy>,

    // Current market state
    price_cache: Arc<RwLock<HashMap<String, f64>>>,
    processing_pipelines: Arc<Mutex<HashSet<String>>>,
    active_pipelines: Arc<DashMap<String, HashSet<String>>>, // asset -> pipeline ids
    shutdown_signal: Arc<Notify>,                            // Used to signal shutdown
    pending_tasks: Arc<AtomicUsize>, // Track number of running pipeline evaluations
}

impl Clone for Engine {
    fn clone(&self) -> Self {
        Self {
            redis: self.redis.clone(),
            redis_sub: self.redis_sub.clone(),
            privy: self.privy.clone(),
            price_cache: self.price_cache.clone(),
            processing_pipelines: self.processing_pipelines.clone(),
            active_pipelines: self.active_pipelines.clone(),
            shutdown_signal: self.shutdown_signal.clone(),
            pending_tasks: self.pending_tasks.clone(),
        }
    }
}

impl Engine {
    pub async fn from_env() -> Result<(Self, mpsc::Receiver<PriceUpdate>), EngineError> {
        let (tx, rx) = mpsc::channel(1000);

        Ok((
            Self {
                privy: Arc::new(Privy::new(
                    PrivyConfig::from_env().map_err(EngineError::PrivyConfigError)?,
                )),
                redis: make_redis_client()
                    .await
                    .map_err(EngineError::RedisClientError)?,
                redis_sub: make_redis_subscriber(tx).map_err(EngineError::RedisSubscriberError)?,
                price_cache: Arc::new(RwLock::new(HashMap::new())),
                processing_pipelines: Arc::new(Mutex::new(HashSet::new())),
                active_pipelines: Arc::new(DashMap::new()),
                shutdown_signal: Arc::new(Notify::new()),
                pending_tasks: Arc::new(AtomicUsize::new(0)),
            },
            rx,
        ))
    }

    pub async fn run(
        engine: Arc<Self>,
        mut receiver: mpsc::Receiver<PriceUpdate>,
        mut command_rx: mpsc::Receiver<EngineMessage>,
    ) -> Result<()> {
        tracing::info!("Engine starting up");

        let existing_pipelines = match engine.redis.get_all_pipelines().await {
            Ok(p) => {
                tracing::info!("{} pipelines from Redis", p.len());
                p
            }
            Err(e) => {
                tracing::error!("Failed to load pipelines from Redis: {}", e);
                return Err(e.into());
            }
        };

        // load existing pipelines into active pipelines
        for pipeline in existing_pipelines {
            let asset_ids = engine.extract_assets(&pipeline).await;
            for asset_id in asset_ids {
                engine
                    .active_pipelines
                    .entry(asset_id.clone())
                    .or_default()
                    .insert(format!("{}:{}", pipeline.user_id, pipeline.id));
            }
        }

        engine.redis_sub.start_listening().await?;

        loop {
            tokio::select! {
                Some(msg) = command_rx.recv() => {
                    tracing::debug!("Received engine message: {:?}", msg);
                    match msg {
                        EngineMessage::AddPipeline { pipeline, response_tx } => {
                            let asset_ids = engine.extract_assets(&pipeline).await;
                            for asset_id in asset_ids {
                                engine.active_pipelines.entry(asset_id.clone()).or_default().insert(format!("{}:{}", pipeline.user_id, pipeline.id));
                                engine.redis.save_pipeline(&pipeline).await?;
                            }
                            let _ = response_tx.send(Ok(pipeline.id.to_string()));
                        },
                        EngineMessage::DeletePipeline { .. } => {
                            panic!("DeletePipeline not implemented");
                            // let result = self.delete_pipeline(&user_id, pipeline_id).await;
                            // let _ = response_tx.send(result);
                        },
                        EngineMessage::GetPipeline { .. } => {
                            panic!("GetPipeline not implemented");
                        },
                        EngineMessage::GetAllPipelinesByUser { user_id, response_tx } => {
                            let result = engine.get_all_pipelines_by_user(&user_id).await;
                            match &result {
                                Ok(pipelines) => tracing::debug!("Found {} pipelines for user", pipelines.len()),
                                Err(e) => tracing::error!("Error getting pipelines: {}", e),
                            }
                            if response_tx.send(result).is_err() {
                                tracing::error!("Failed to send response - channel closed");
                            }
                        },
                    }
                }
                Some(price_update) = receiver.recv() => {
                    if let Err(e) = engine.handle_price_update(&price_update.pubkey, price_update.price, price_update.slot).await {
                        tracing::error!("Error handling price update: {}", e);
                    }
                }
                else => break
            }
        }

        Ok(())
    }

    pub async fn handle_price_update(&self, asset: &str, price: f64, slot: u64) -> Result<()> {
        let start = Instant::now();
        counter!("price_updates_processed", 1);

        let pipeline_ids = {
            let mut res = Vec::new();
            if let Some(now_pipeline_ids) = self.active_pipelines.get(&"NOW".to_string()) {
                res.extend(now_pipeline_ids.iter().cloned());
            }
            if let Some(active_pipelines) = self.active_pipelines.get(&asset.to_string()) {
                res.extend(active_pipelines.iter().cloned());
            }
            res
        };

        // Update price cache after getting pipeline IDs
        {
            let mut cache = self.price_cache.write().await;
            cache.insert(asset.to_string(), price);
        }

        // Process in chunks to limit concurrent Redis connections
        for chunk in pipeline_ids.chunks(10) {
            let asset = asset.to_string();

            // Batch fetch pipelines from Redis
            let mut pipe = bb8_redis::redis::pipe();
            for id in chunk {
                pipe.get(format!("pipeline:{}", id));
            }

            let pipelines: Vec<Option<Pipeline>> = self.redis.execute_redis_pipe(pipe).await?;
            tracing::debug!("Fetched {} from redis", pipelines.len());

            // Process the fetched pipelines concurrently
            for (pipeline_id, maybe_pipeline) in chunk.iter().zip(pipelines) {
                if let Some(mut pipeline) = maybe_pipeline {
                    let self_clone = self.clone();
                    let pipeline_id = pipeline_id.clone();
                    let asset = asset.clone();

                    if !matches!(pipeline.status, Status::Pending) {
                        continue;
                    }

                    let can_process = {
                        let mut processing = self_clone.processing_pipelines.lock().await;
                        if processing.contains(&pipeline_id) {
                            tracing::warn!("Pipeline {} already being processed", pipeline_id);
                            false
                        } else {
                            processing.insert(pipeline_id.clone());
                            true
                        }
                    };

                    if can_process && !matches!(pipeline.status, Status::Failed | Status::Cancelled)
                    {
                        // Increment pending tasks counter
                        self_clone.pending_tasks.fetch_add(1, Ordering::SeqCst);

                        // Spawn a detached task for pipeline evaluation
                        let shutdown = self_clone.shutdown_signal.clone();
                        tokio::spawn(async move {
                            let result = async {
                                tokio::select! {
                                    r = self_clone.evaluate_pipeline(&mut pipeline) => r,
                                    _ = shutdown.notified() => {
                                        tracing::info!("Gracefully stopping pipeline evaluation for {}", pipeline_id);
                                        Ok(false) // Don't mark as complete if interrupted
                                    }
                                }
                            }.await;

                            match result {
                                Ok(is_complete) => {
                                    if is_complete {
                                        if let Some(mut pipelines) =
                                            self_clone.active_pipelines.get_mut(&asset)
                                        {
                                            pipelines.remove(&pipeline_id);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Pipeline evaluation error: {}", e);
                                }
                            }

                            // Always release the processing lock
                            let mut processing = self_clone.processing_pipelines.lock().await;
                            processing.remove(&pipeline_id);

                            // Decrement pending tasks counter
                            self_clone.pending_tasks.fetch_sub(1, Ordering::SeqCst);
                        });
                    }
                }
            }
        }

        histogram!("price_update_duration", start.elapsed());
        tracing::debug!("{}: {} {} took {:?}", asset, price, slot, start.elapsed());
        Ok(())
    }

    pub async fn shutdown(&self) {
        // Signal all pipeline evaluations to stop
        self.shutdown_signal.notify_waiters();

        // Wait for all pending tasks to complete
        while self.pending_tasks.load(Ordering::SeqCst) > 0 {
            tracing::info!(
                "Waiting for {} pipeline evaluations to complete...",
                self.pending_tasks.load(Ordering::SeqCst)
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        tracing::info!("All pipeline evaluations completed");
    }
}
