use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::{Any, Pool, Row};
use serde::{Serialize, Deserialize};

/// Menggunakan macro dan runtime metrics untuk analisis komprehensif

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation_name: String,
    pub duration_ms: f64,
    pub memory_usage_mb: f64,
    pub database_queries: u32,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Default)]
pub struct PerformanceProfiler {
    metrics: Arc<Mutex<Vec<PerformanceMetrics>>>,
    operation_counters: Arc<Mutex<HashMap<String, u32>>>,
}

impl PerformanceProfiler {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            operation_counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Macro-like function untuk automatic profiling
    pub async fn profile_operation<F, R>(&self, operation_name: &str, operation: F) -> Result<R, Box<dyn std::error::Error>>
    where
        F: std::future::Future<Output = Result<R, Box<dyn std::error::Error>>>,
    {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();
        
        // Increment operation counter
        {
            let mut counters = self.operation_counters.lock().await;
            *counters.entry(operation_name.to_string()).or_insert(0) += 1;
        }

        let result = operation.await;
        
        let duration = start_time.elapsed();
        let end_memory = self.get_memory_usage();
        let memory_delta = (end_memory as i64 - start_memory as i64).max(0) as f64;

        let metric = PerformanceMetrics {
            operation_name: operation_name.to_string(),
            duration_ms: duration.as_secs_f64() * 1000.0,
            memory_usage_mb: memory_delta / 1_048_576.0, // Convert bytes to MB
            database_queries: 1,
            timestamp: Utc::now(),
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
        };

        {
            let mut metrics = self.metrics.lock().await;
            metrics.push(metric);
        }

        result
    }

    fn get_memory_usage(&self) -> usize {
        std::mem::size_of::<Self>() * 1024 // Placeholder
    }

    pub async fn get_metrics(&self) -> Vec<PerformanceMetrics> {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }

    pub async fn get_summary(&self) -> PerformanceSummary {
        let metrics = self.metrics.lock().await;
        let mut summary = PerformanceSummary::default();

        for metric in metrics.iter() {
            summary.total_operations += 1;
            summary.total_duration_ms += metric.duration_ms;
            summary.total_memory_usage_mb += metric.memory_usage_mb;
            summary.total_database_queries += metric.database_queries;

            if metric.success {
                summary.successful_operations += 1;
            } else {
                summary.failed_operations += 1;
            }

            // Track per-operation stats
            let op_stats = summary.operation_stats.entry(metric.operation_name.clone()).or_insert(OperationStats::default());
            op_stats.count += 1;
            op_stats.total_duration_ms += metric.duration_ms;
            op_stats.total_memory_mb += metric.memory_usage_mb;
            
            if metric.duration_ms > op_stats.max_duration_ms {
                op_stats.max_duration_ms = metric.duration_ms;
            }
            if op_stats.min_duration_ms == 0.0 || metric.duration_ms < op_stats.min_duration_ms {
                op_stats.min_duration_ms = metric.duration_ms;
            }
        }

        // Calculate averages
        for op_stats in summary.operation_stats.values_mut() {
            op_stats.avg_duration_ms = op_stats.total_duration_ms / op_stats.count as f64;
            op_stats.avg_memory_mb = op_stats.total_memory_mb / op_stats.count as f64;
        }

        if summary.total_operations > 0 {
            summary.average_duration_ms = summary.total_duration_ms / summary.total_operations as f64;
            summary.average_memory_usage_mb = summary.total_memory_usage_mb / summary.total_operations as f64;
        }

        summary
    }

    pub async fn clear_metrics(&self) {
        let mut metrics = self.metrics.lock().await;
        metrics.clear();
        let mut counters = self.operation_counters.lock().await;
        counters.clear();
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub total_operations: u32,
    pub successful_operations: u32,
    pub failed_operations: u32,
    pub total_duration_ms: f64,
    pub average_duration_ms: f64,
    pub total_memory_usage_mb: f64,
    pub average_memory_usage_mb: f64,
    pub total_database_queries: u32,
    pub operation_stats: HashMap<String, OperationStats>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OperationStats {
    pub count: u32,
    pub total_duration_ms: f64,
    pub avg_duration_ms: f64,
    pub min_duration_ms: f64,
    pub max_duration_ms: f64,
    pub total_memory_mb: f64,
    pub avg_memory_mb: f64,
}

/// Database-specific profiling functions
pub struct DatabaseProfiler {
    profiler: PerformanceProfiler,
}

impl DatabaseProfiler {
    pub fn new() -> Self {
        Self {
            profiler: PerformanceProfiler::new(),
        }
    }

    pub async fn profile_query<T>(&self, query_name: &str, pool: &Pool<Any>, query: &str, params: &[&(dyn sqlx::Encode<'_, Any> + Send + Sync)]) -> Result<Vec<sqlx::any::AnyRow>, sqlx::Error> {
        let operation = async {
            let mut conn = pool.acquire().await?;
            let mut query_builder = sqlx::query(query);
            
            for param in params {
                query_builder = query_builder.bind(*param);
            }
            
            let result = query_builder.fetch_all(&mut *conn).await?;
            Ok::<Vec<sqlx::any::AnyRow>, sqlx::Error>(result)
        };

        let wrapped_operation = async {
            operation.await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        };

        self.profiler.profile_operation(query_name, wrapped_operation).await
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))
    }

    pub async fn get_metrics(&self) -> Vec<PerformanceMetrics> {
        self.profiler.get_metrics().await
    }

    pub async fn get_summary(&self) -> PerformanceSummary {
        self.profiler.get_summary().await
    }
}

/// Benchmark comparison utilities
pub struct BenchmarkComparison {
    pub before_metrics: PerformanceSummary,
    pub after_metrics: PerformanceSummary,
}

impl BenchmarkComparison {
    pub fn new(before: PerformanceSummary, after: PerformanceSummary) -> Self {
        Self {
            before_metrics: before,
            after_metrics: after,
        }
    }

    pub fn calculate_improvements(&self) -> ImprovementReport {
        let duration_improvement = if self.before_metrics.average_duration_ms > 0.0 {
            ((self.before_metrics.average_duration_ms - self.after_metrics.average_duration_ms) / self.before_metrics.average_duration_ms) * 100.0
        } else {
            0.0
        };

        let memory_improvement = if self.before_metrics.average_memory_usage_mb > 0.0 {
            ((self.before_metrics.average_memory_usage_mb - self.after_metrics.average_memory_usage_mb) / self.before_metrics.average_memory_usage_mb) * 100.0
        } else {
            0.0
        };

        let throughput_improvement = if self.before_metrics.total_operations > 0 && self.after_metrics.total_operations > 0 {
            let before_ops_per_sec = self.before_metrics.total_operations as f64 / (self.before_metrics.total_duration_ms / 1000.0);
            let after_ops_per_sec = self.after_metrics.total_operations as f64 / (self.after_metrics.total_duration_ms / 1000.0);
            
            if before_ops_per_sec > 0.0 {
                ((after_ops_per_sec - before_ops_per_sec) / before_ops_per_sec) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        ImprovementReport {
            duration_improvement_percent: duration_improvement,
            memory_improvement_percent: memory_improvement,
            throughput_improvement_percent: throughput_improvement,
            error_rate_before: if self.before_metrics.total_operations > 0 {
                (self.before_metrics.failed_operations as f64 / self.before_metrics.total_operations as f64) * 100.0
            } else {
                0.0
            },
            error_rate_after: if self.after_metrics.total_operations > 0 {
                (self.after_metrics.failed_operations as f64 / self.after_metrics.total_operations as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    pub fn generate_report(&self) -> String {
        let improvements = self.calculate_improvements();
        
        format!(
            r#"
=== PROFILING COMPARISON REPORT ===

BEFORE OPTIMIZATION:
- Average Duration: {:.2}ms
- Average Memory Usage: {:.2}MB  
- Total Operations: {}
- Success Rate: {:.1}%
- Error Rate: {:.1}%

AFTER OPTIMIZATION:
- Average Duration: {:.2}ms
- Average Memory Usage: {:.2}MB
- Total Operations: {}
- Success Rate: {:.1}%
- Error Rate: {:.1}%

IMPROVEMENTS:
- Duration: {:.1}% {}
- Memory Usage: {:.1}% {}
- Throughput: {:.1}% {}
- Error Rate: {:.1}% -> {:.1}%

=== DETAILED OPERATION ANALYSIS ===
{}"#,
            self.before_metrics.average_duration_ms,
            self.before_metrics.average_memory_usage_mb,
            self.before_metrics.total_operations,
            if self.before_metrics.total_operations > 0 {
                (self.before_metrics.successful_operations as f64 / self.before_metrics.total_operations as f64) * 100.0
            } else { 0.0 },
            improvements.error_rate_before,
            
            self.after_metrics.average_duration_ms,
            self.after_metrics.average_memory_usage_mb,
            self.after_metrics.total_operations,
            if self.after_metrics.total_operations > 0 {
                (self.after_metrics.successful_operations as f64 / self.after_metrics.total_operations as f64) * 100.0
            } else { 0.0 },
            improvements.error_rate_after,
            
            improvements.duration_improvement_percent,
            if improvements.duration_improvement_percent > 0.0 { "faster" } else { "slower" },
            improvements.memory_improvement_percent,
            if improvements.memory_improvement_percent > 0.0 { "less memory" } else { "more memory" },
            improvements.throughput_improvement_percent,
            if improvements.throughput_improvement_percent > 0.0 { "higher" } else { "lower" },
            improvements.error_rate_before,
            improvements.error_rate_after,
            
            self.generate_operation_comparison()
        )
    }

    fn generate_operation_comparison(&self) -> String {
        let mut comparison = String::new();
        
        for (op_name, before_stats) in &self.before_metrics.operation_stats {
            if let Some(after_stats) = self.after_metrics.operation_stats.get(op_name) {
                let duration_change = if before_stats.avg_duration_ms > 0.0 {
                    ((before_stats.avg_duration_ms - after_stats.avg_duration_ms) / before_stats.avg_duration_ms) * 100.0
                } else {
                    0.0
                };
                
                comparison.push_str(&format!(
                    "\n{}: {:.2}ms -> {:.2}ms ({:.1}% {})",
                    op_name,
                    before_stats.avg_duration_ms,
                    after_stats.avg_duration_ms,
                    duration_change.abs(),
                    if duration_change > 0.0 { "improvement" } else { "regression" }
                ));
            }
        }
        
        comparison
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImprovementReport {
    pub duration_improvement_percent: f64,
    pub memory_improvement_percent: f64,
    pub throughput_improvement_percent: f64,
    pub error_rate_before: f64,
    pub error_rate_after: f64,
}

/// Test data generator untuk consistent profiling
pub struct PaymentTestDataGenerator;

impl PaymentTestDataGenerator {
    pub fn generate_test_payments(count: usize) -> Vec<crate::manajemen_pembayaran::model::payment::Payment> {
        use crate::manajemen_pembayaran::model::payment::{Payment, PaymentMethod, Installment};
        use crate::manajemen_pembayaran::enums::payment_status::PaymentStatus;
        
        (0..count).map(|i| {
            let has_installments = i % 3 == 0; // Every 3rd payment has installments
            let installments = if has_installments {
                vec![
                    Installment {
                        id: format!("INST-{}-1", Uuid::new_v4()),
                        payment_id: format!("PMT-TEST-{}", i),
                        amount: 250.0,
                        payment_date: Utc::now(),
                    },
                    Installment {
                        id: format!("INST-{}-2", Uuid::new_v4()),
                        payment_id: format!("PMT-TEST-{}", i),
                        amount: 350.0,
                        payment_date: Utc::now(),
                    },
                ]
            } else {
                vec![]
            };

            Payment {
                id: format!("PMT-TEST-{}", i),
                transaction_id: format!("TXN-TEST-{}", i),
                amount: 1000.0 + (i as f64 * 100.0),
                method: match i % 4 {
                    0 => PaymentMethod::Cash,
                    1 => PaymentMethod::CreditCard,
                    2 => PaymentMethod::BankTransfer,
                    _ => PaymentMethod::EWallet,
                },
                status: if has_installments { PaymentStatus::Installment } else { PaymentStatus::Paid },
                payment_date: Utc::now(),
                due_date: if i % 2 == 0 { Some(Utc::now()) } else { None },
                installments,
            }
        }).collect()
    }

    pub fn generate_large_dataset(size: usize) -> Vec<crate::manajemen_pembayaran::model::payment::Payment> {
        Self::generate_test_payments(size)
    }
}
