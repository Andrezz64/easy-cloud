use serde::Serialize;
use aws_config::{Region, BehaviorVersion};
use aws_credential_types::Credentials;
use aws_sdk_costexplorer::Client as CeClient;
use aws_sdk_costexplorer::types::{
    DateInterval, Granularity, GroupDefinition, GroupDefinitionType, MetricValue,
};

// ============================================================
// Types
// ============================================================

#[derive(Serialize)]
pub struct ServiceCost {
    pub service: String,
    pub amount: String,
    pub unit: String,
}

#[derive(Serialize)]
pub struct DailyCost {
    pub date: String,
    pub amount: String,
    pub unit: String,
}

#[derive(Serialize)]
pub struct MonthlyCostResponse {
    pub success: bool,
    pub total_amount: Option<String>,
    pub total_unit: Option<String>,
    pub services: Option<Vec<ServiceCost>>,
    pub error_message: Option<String>,
}

#[derive(Serialize)]
pub struct DailyCostResponse {
    pub success: bool,
    pub costs: Option<Vec<DailyCost>>,
    pub error_message: Option<String>,
}

#[derive(Serialize)]
pub struct CostForecastResponse {
    pub success: bool,
    pub forecasted_amount: Option<String>,
    pub forecasted_unit: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Serialize)]
pub struct CostSummaryResponse {
    pub success: bool,
    pub current_month_total: Option<String>,
    pub last_month_total: Option<String>,
    pub forecast_month_total: Option<String>,
    pub unit: Option<String>,
    pub services: Option<Vec<ServiceCost>>,
    pub daily_costs: Option<Vec<DailyCost>>,
    pub error_message: Option<String>,
}

// ============================================================
// Helper
// ============================================================

async fn build_ce_client(access_key_id: &str, secret_access_key: &str, region: &str) -> CeClient {
    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "easy-cloud-app");
    // Cost Explorer API is global, but the SDK requires a region (us-east-1 is standard)
    let effective_region = if region.is_empty() { "us-east-1" } else { region };
    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(Region::new(effective_region.to_string()))
        .load()
        .await;
    CeClient::new(&config)
}

fn extract_amount(metrics: &std::collections::HashMap<String, MetricValue>, key: &str) -> (String, String) {
    if let Some(mv) = metrics.get(key) {
        let amount = mv.amount().unwrap_or("0").to_string();
        let unit = mv.unit().unwrap_or("USD").to_string();
        (amount, unit)
    } else {
        ("0".to_string(), "USD".to_string())
    }
}

fn extract_amount_opt(metrics: Option<&std::collections::HashMap<String, MetricValue>>, key: &str) -> (String, String) {
    match metrics {
        Some(m) => extract_amount(m, key),
        None => ("0".to_string(), "USD".to_string()),
    }
}

fn get_current_month_range() -> (String, String) {
    let now = chrono_lite_now();
    let start = format!("{}-{:02}-01", now.0, now.1);
    let end = format!("{}-{:02}-{:02}", now.0, now.1, now.2);
    (start, end)
}

fn get_last_month_range() -> (String, String) {
    let now = chrono_lite_now();
    let (year, month) = if now.1 == 1 { (now.0 - 1, 12) } else { (now.0, now.1 - 1) };
    let start = format!("{}-{:02}-01", year, month);
    // End is first day of current month (exclusive in AWS API)
    let end = format!("{}-{:02}-01", now.0, now.1);
    (start, end)
}

fn get_forecast_range() -> (String, String) {
    let now = chrono_lite_now();
    let start = format!("{}-{:02}-{:02}", now.0, now.1, now.2);
    // End of current month
    let end_month = if now.1 == 12 {
        format!("{}-01-01", now.0 + 1)
    } else {
        format!("{}-{:02}-01", now.0, now.1 + 1)
    };
    (start, end_month)
}

/// Simple date helper to avoid pulling in chrono crate.
/// Returns (year, month, day) in UTC.
fn chrono_lite_now() -> (i32, u32, u32) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    // Simple calculation for current UTC date
    let days = (secs / 86400) as i64;
    // Days since 1970-01-01
    let (year, month, day) = days_to_ymd(days);
    (year, month, day)
}

fn days_to_ymd(days_since_epoch: i64) -> (i32, u32, u32) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ============================================================
// Commands
// ============================================================

/// Get cost breakdown by service for the current month
#[tauri::command]
pub async fn get_monthly_cost_by_service(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<MonthlyCostResponse, String> {
    let client = build_ce_client(&access_key_id, &secret_access_key, &region).await;
    let (start, end) = get_current_month_range();

    let time_period = DateInterval::builder()
        .start(&start)
        .end(&end)
        .build()
        .map_err(|e| e.to_string())?;

    let group = GroupDefinition::builder()
        .r#type(GroupDefinitionType::Dimension)
        .key("SERVICE")
        .build();

    let result = client
        .get_cost_and_usage()
        .time_period(time_period)
        .granularity(Granularity::Monthly)
        .metrics("UnblendedCost")
        .group_by(group)
        .send()
        .await;

    match result {
        Ok(response) => {
            let mut services = Vec::new();
            let mut total: f64 = 0.0;
            let mut unit = "USD".to_string();

            for result_by_time in response.results_by_time() {
                for group in result_by_time.groups() {
                    let service_name = group.keys().first().map(|s| s.to_string()).unwrap_or_default();
                    let (amount, u) = extract_amount_opt(group.metrics(), "UnblendedCost");
                    unit = u.clone();
                    let amt: f64 = amount.parse().unwrap_or(0.0);
                    if amt > 0.001 {
                        total += amt;
                        services.push(ServiceCost { service: service_name, amount, unit: u });
                    }
                }
            }

            // Sort by amount descending
            services.sort_by(|a, b| {
                let a_val: f64 = a.amount.parse().unwrap_or(0.0);
                let b_val: f64 = b.amount.parse().unwrap_or(0.0);
                b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
            });

            Ok(MonthlyCostResponse {
                success: true,
                total_amount: Some(format!("{:.2}", total)),
                total_unit: Some(unit),
                services: Some(services),
                error_message: None,
            })
        }
        Err(e) => Ok(MonthlyCostResponse {
            success: false,
            total_amount: None,
            total_unit: None,
            services: None,
            error_message: Some(e.to_string()),
        }),
    }
}

/// Get daily cost for the current month
#[tauri::command]
pub async fn get_daily_costs(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<DailyCostResponse, String> {
    let client = build_ce_client(&access_key_id, &secret_access_key, &region).await;
    let (start, end) = get_current_month_range();

    let time_period = DateInterval::builder()
        .start(&start)
        .end(&end)
        .build()
        .map_err(|e| e.to_string())?;

    let result = client
        .get_cost_and_usage()
        .time_period(time_period)
        .granularity(Granularity::Daily)
        .metrics("UnblendedCost")
        .send()
        .await;

    match result {
        Ok(response) => {
            let mut costs = Vec::new();

            for result_by_time in response.results_by_time() {
                let date = result_by_time.time_period()
                    .map(|tp| tp.start().to_string())
                    .unwrap_or_default();
                let (amount, unit) = if let Some(total) = result_by_time.total() {
                    extract_amount(total, "UnblendedCost")
                } else {
                    ("0".to_string(), "USD".to_string())
                };
                costs.push(DailyCost { date, amount, unit });
            }

            Ok(DailyCostResponse { success: true, costs: Some(costs), error_message: None })
        }
        Err(e) => Ok(DailyCostResponse { success: false, costs: None, error_message: Some(e.to_string()) }),
    }
}

/// Get forecast for the rest of the current month
#[tauri::command]
pub async fn get_cost_forecast(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<CostForecastResponse, String> {
    let client = build_ce_client(&access_key_id, &secret_access_key, &region).await;
    let (start, end) = get_forecast_range();

    // Forecast requires start to be tomorrow or later
    if start >= end {
        return Ok(CostForecastResponse {
            success: true,
            forecasted_amount: Some("0.00".to_string()),
            forecasted_unit: Some("USD".to_string()),
            error_message: Some("Last day of the month — no forecast available".to_string()),
        });
    }

    let time_period = DateInterval::builder()
        .start(&start)
        .end(&end)
        .build()
        .map_err(|e| e.to_string())?;

    let result = client
        .get_cost_forecast()
        .time_period(time_period)
        .granularity(Granularity::Monthly)
        .metric(aws_sdk_costexplorer::types::Metric::UnblendedCost)
        .send()
        .await;

    match result {
        Ok(response) => {
            let (amount, unit) = if let Some(total) = response.total() {
                (
                    total.amount().unwrap_or("0").to_string(),
                    total.unit().unwrap_or("USD").to_string(),
                )
            } else {
                ("0".to_string(), "USD".to_string())
            };
            Ok(CostForecastResponse {
                success: true,
                forecasted_amount: Some(amount),
                forecasted_unit: Some(unit),
                error_message: None,
            })
        }
        Err(e) => Ok(CostForecastResponse {
            success: false,
            forecasted_amount: None,
            forecasted_unit: None,
            error_message: Some(e.to_string()),
        }),
    }
}

/// Get a consolidated cost summary: current month, last month, forecast, top services, daily breakdown
#[tauri::command]
pub async fn get_cost_summary(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<CostSummaryResponse, String> {
    let client = build_ce_client(&access_key_id, &secret_access_key, &region).await;

    // --- Current month by service ---
    let (cm_start, cm_end) = get_current_month_range();
    let cm_period = DateInterval::builder()
        .start(&cm_start)
        .end(&cm_end)
        .build()
        .map_err(|e| e.to_string())?;

    let group = GroupDefinition::builder()
        .r#type(GroupDefinitionType::Dimension)
        .key("SERVICE")
        .build();

    let cm_result = client
        .get_cost_and_usage()
        .time_period(cm_period.clone())
        .granularity(Granularity::Monthly)
        .metrics("UnblendedCost")
        .group_by(group)
        .send()
        .await;

    let (current_month_total, services, unit) = match cm_result {
        Ok(response) => {
            let mut svcs = Vec::new();
            let mut total: f64 = 0.0;
            let mut u = "USD".to_string();
            for rbt in response.results_by_time() {
                for g in rbt.groups() {
                    let service_name = g.keys().first().map(|s| s.to_string()).unwrap_or_default();
                    let (amount, uu) = extract_amount_opt(g.metrics(), "UnblendedCost");
                    u = uu.clone();
                    let amt: f64 = amount.parse().unwrap_or(0.0);
                    if amt > 0.001 {
                        total += amt;
                        svcs.push(ServiceCost { service: service_name, amount, unit: uu });
                    }
                }
            }
            svcs.sort_by(|a, b| {
                let a_val: f64 = a.amount.parse().unwrap_or(0.0);
                let b_val: f64 = b.amount.parse().unwrap_or(0.0);
                b_val.partial_cmp(&a_val).unwrap_or(std::cmp::Ordering::Equal)
            });
            (Some(format!("{:.2}", total)), Some(svcs), u)
        }
        Err(e) => return Ok(CostSummaryResponse {
            success: false,
            current_month_total: None,
            last_month_total: None,
            forecast_month_total: None,
            unit: None,
            services: None,
            daily_costs: None,
            error_message: Some(e.to_string()),
        }),
    };

    // --- Last month total ---
    let (lm_start, lm_end) = get_last_month_range();
    let lm_period = DateInterval::builder()
        .start(&lm_start)
        .end(&lm_end)
        .build()
        .map_err(|e| e.to_string())?;

    let lm_result = client
        .get_cost_and_usage()
        .time_period(lm_period)
        .granularity(Granularity::Monthly)
        .metrics("UnblendedCost")
        .send()
        .await;

    let last_month_total = match lm_result {
        Ok(response) => {
            let mut total: f64 = 0.0;
            for rbt in response.results_by_time() {
                if let Some(t) = rbt.total() {
                    let (amount, _) = extract_amount(t, "UnblendedCost");
                    total += amount.parse::<f64>().unwrap_or(0.0);
                }
            }
            Some(format!("{:.2}", total))
        }
        Err(_) => None,
    };

    // --- Daily costs current month ---
    let cm_daily_period = DateInterval::builder()
        .start(&cm_start)
        .end(&cm_end)
        .build()
        .map_err(|e| e.to_string())?;

    let daily_result = client
        .get_cost_and_usage()
        .time_period(cm_daily_period)
        .granularity(Granularity::Daily)
        .metrics("UnblendedCost")
        .send()
        .await;

    let daily_costs = match daily_result {
        Ok(response) => {
            let mut costs = Vec::new();
            for rbt in response.results_by_time() {
                let date = rbt.time_period().map(|tp| tp.start().to_string()).unwrap_or_default();
                let (amount, u) = if let Some(total) = rbt.total() {
                    extract_amount(total, "UnblendedCost")
                } else {
                    ("0".to_string(), "USD".to_string())
                };
                costs.push(DailyCost { date, amount, unit: u });
            }
            Some(costs)
        }
        Err(_) => None,
    };

    // --- Forecast ---
    let (fc_start, fc_end) = get_forecast_range();
    let forecast_month_total = if fc_start < fc_end {
        let fc_period = DateInterval::builder()
            .start(&fc_start)
            .end(&fc_end)
            .build()
            .ok();

        if let Some(period) = fc_period {
            match client
                .get_cost_forecast()
                .time_period(period)
                .granularity(Granularity::Monthly)
                .metric(aws_sdk_costexplorer::types::Metric::UnblendedCost)
                .send()
                .await
            {
                Ok(response) => response.total().and_then(|t| t.amount().map(|a| a.to_string())),
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };

    Ok(CostSummaryResponse {
        success: true,
        current_month_total,
        last_month_total,
        forecast_month_total,
        unit: Some(unit),
        services,
        daily_costs,
        error_message: None,
    })
}

// ============================================================
// Usage Metrics
// ============================================================

#[derive(Serialize)]
pub struct UsageDetail {
    pub usage_type: String,
    pub amount: String,
    pub unit: String,
    pub cost: String,
    pub cost_unit: String,
    pub unit_price: String,
    pub is_free_tier: bool,
}

#[derive(Serialize)]
pub struct ServiceUsage {
    pub service: String,
    pub usage_details: Vec<UsageDetail>,
    pub total_usage: String,
    pub total_cost: String,
}

#[derive(Serialize)]
pub struct UsageByServiceResponse {
    pub success: bool,
    pub services: Option<Vec<ServiceUsage>>,
    pub error_message: Option<String>,
}

/// Get usage quantities AND cost grouped by service and usage type for the current month.
/// Returns usage × effective unit price = cost for each usage type.
#[tauri::command]
pub async fn get_usage_by_service(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<UsageByServiceResponse, String> {
    let client = build_ce_client(&access_key_id, &secret_access_key, &region).await;
    let (start, end) = get_current_month_range();

    let time_period = DateInterval::builder()
        .start(&start)
        .end(&end)
        .build()
        .map_err(|e| e.to_string())?;

    let group_service = GroupDefinition::builder()
        .r#type(GroupDefinitionType::Dimension)
        .key("SERVICE")
        .build();

    let group_usage_type = GroupDefinition::builder()
        .r#type(GroupDefinitionType::Dimension)
        .key("USAGE_TYPE")
        .build();

    let result = client
        .get_cost_and_usage()
        .time_period(time_period)
        .granularity(Granularity::Monthly)
        .metrics("UsageQuantity")
        .metrics("UnblendedCost")
        .group_by(group_service)
        .group_by(group_usage_type)
        .send()
        .await;

    match result {
        Ok(response) => {
            let mut service_map: std::collections::HashMap<String, Vec<UsageDetail>> = std::collections::HashMap::new();

            for result_by_time in response.results_by_time() {
                for group in result_by_time.groups() {
                    let keys: Vec<&str> = group.keys().iter().map(|s| s.as_str()).collect();
                    let service_name = keys.first().copied().unwrap_or("Unknown").to_string();
                    let usage_type = keys.get(1).copied().unwrap_or("Unknown").to_string();

                    let (amount, unit) = extract_amount_opt(group.metrics(), "UsageQuantity");
                    let (cost, cost_unit) = extract_amount_opt(group.metrics(), "UnblendedCost");

                    let amt: f64 = amount.parse().unwrap_or(0.0);

                    // Skip zero usage
                    if amt < 0.0001 {
                        continue;
                    }

                    let cost_val: f64 = cost.parse().unwrap_or(0.0);
                    let unit_price = if amt > 0.0 && cost_val > 0.0 {
                        cost_val / amt
                    } else {
                        0.0
                    };
                    let is_free_tier = amt > 0.0001 && cost_val < 0.000001;

                    service_map.entry(service_name).or_default().push(UsageDetail {
                        usage_type,
                        amount,
                        unit,
                        cost,
                        cost_unit,
                        unit_price: format!("{:.10}", unit_price),
                        is_free_tier,
                    });
                }
            }

            // Convert to Vec and sort by total cost descending
            let mut services: Vec<ServiceUsage> = service_map.into_iter().map(|(service, mut details)| {
                // Sort details within service by cost desc (then usage)
                details.sort_by(|a, b| {
                    let ac: f64 = a.cost.parse().unwrap_or(0.0);
                    let bc: f64 = b.cost.parse().unwrap_or(0.0);
                    let cmp = bc.partial_cmp(&ac).unwrap_or(std::cmp::Ordering::Equal);
                    if cmp == std::cmp::Ordering::Equal {
                        let av: f64 = a.amount.parse().unwrap_or(0.0);
                        let bv: f64 = b.amount.parse().unwrap_or(0.0);
                        bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        cmp
                    }
                });

                let total_usage: f64 = details.iter().map(|d| d.amount.parse::<f64>().unwrap_or(0.0)).sum();
                let total_cost: f64 = details.iter().map(|d| d.cost.parse::<f64>().unwrap_or(0.0)).sum();

                ServiceUsage {
                    service,
                    usage_details: details,
                    total_usage: format!("{:.4}", total_usage),
                    total_cost: format!("{:.6}", total_cost),
                }
            }).collect();

            // Sort services by total cost descending
            services.sort_by(|a, b| {
                let ac: f64 = a.total_cost.parse().unwrap_or(0.0);
                let bc: f64 = b.total_cost.parse().unwrap_or(0.0);
                bc.partial_cmp(&ac).unwrap_or(std::cmp::Ordering::Equal)
            });

            Ok(UsageByServiceResponse {
                success: true,
                services: Some(services),
                error_message: None,
            })
        }
        Err(e) => Ok(UsageByServiceResponse {
            success: false,
            services: None,
            error_message: Some(e.to_string()),
        }),
    }
}

// ============================================================
// Cost Report (custom date range)
// ============================================================

#[derive(Serialize)]
pub struct CostReportDataPoint {
    pub date: String,
    pub amount: String,
    pub unit: String,
}

#[derive(Serialize)]
pub struct CostReportResponse {
    pub success: bool,
    pub granularity: String,
    pub start_date: String,
    pub end_date: String,
    pub total_amount: Option<String>,
    pub total_unit: Option<String>,
    pub data_points: Option<Vec<CostReportDataPoint>>,
    pub services: Option<Vec<ServiceCost>>,
    pub error_message: Option<String>,
}

/// Get cost data for a custom date range.
/// Granularity is auto-determined: ≤62 days → Daily, >62 days → Monthly.
/// `start_date` and `end_date` must be in YYYY-MM-DD format.
#[tauri::command]
pub async fn get_cost_report(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    start_date: String,
    end_date: String,
) -> Result<CostReportResponse, String> {
    let client = build_ce_client(&access_key_id, &secret_access_key, &region).await;

    // Determine granularity based on date range
    let days_diff = estimate_days_between(&start_date, &end_date);
    let granularity = if days_diff <= 62 {
        Granularity::Daily
    } else {
        Granularity::Monthly
    };
    let granularity_str = if days_diff <= 62 { "DAILY" } else { "MONTHLY" };

    let time_period = DateInterval::builder()
        .start(&start_date)
        .end(&end_date)
        .build()
        .map_err(|e| e.to_string())?;

    // Fetch data points (total cost over time)
    let data_result = client
        .get_cost_and_usage()
        .time_period(time_period.clone())
        .granularity(granularity.clone())
        .metrics("UnblendedCost")
        .send()
        .await;

    let (data_points, total_amount, total_unit) = match data_result {
        Ok(response) => {
            let mut points = Vec::new();
            let mut total: f64 = 0.0;
            let mut unit = "USD".to_string();

            for rbt in response.results_by_time() {
                let date = rbt.time_period().map(|tp| tp.start().to_string()).unwrap_or_default();
                let (amount, u) = if let Some(t) = rbt.total() {
                    extract_amount(t, "UnblendedCost")
                } else {
                    ("0".to_string(), "USD".to_string())
                };
                unit = u.clone();
                total += amount.parse::<f64>().unwrap_or(0.0);
                points.push(CostReportDataPoint { date, amount, unit: u });
            }

            (Some(points), Some(format!("{:.2}", total)), Some(unit))
        }
        Err(e) => return Ok(CostReportResponse {
            success: false,
            granularity: granularity_str.to_string(),
            start_date,
            end_date,
            total_amount: None,
            total_unit: None,
            data_points: None,
            services: None,
            error_message: Some(e.to_string()),
        }),
    };

    // Fetch service breakdown for the same period
    let group = GroupDefinition::builder()
        .r#type(GroupDefinitionType::Dimension)
        .key("SERVICE")
        .build();

    let svc_time_period = DateInterval::builder()
        .start(&start_date)
        .end(&end_date)
        .build()
        .map_err(|e| e.to_string())?;

    let svc_result = client
        .get_cost_and_usage()
        .time_period(svc_time_period)
        .granularity(Granularity::Monthly)
        .metrics("UnblendedCost")
        .group_by(group)
        .send()
        .await;

    let services = match svc_result {
        Ok(response) => {
            let mut svc_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
            let mut unit = "USD".to_string();

            for rbt in response.results_by_time() {
                for g in rbt.groups() {
                    let name = g.keys().first().map(|s| s.to_string()).unwrap_or_default();
                    let (amount, u) = extract_amount_opt(g.metrics(), "UnblendedCost");
                    unit = u;
                    let amt: f64 = amount.parse().unwrap_or(0.0);
                    *svc_map.entry(name).or_insert(0.0) += amt;
                }
            }

            let mut svcs: Vec<ServiceCost> = svc_map.into_iter()
                .filter(|(_, v)| *v > 0.001)
                .map(|(service, amt)| ServiceCost {
                    service,
                    amount: format!("{:.2}", amt),
                    unit: unit.clone(),
                })
                .collect();

            svcs.sort_by(|a, b| {
                let av: f64 = a.amount.parse().unwrap_or(0.0);
                let bv: f64 = b.amount.parse().unwrap_or(0.0);
                bv.partial_cmp(&av).unwrap_or(std::cmp::Ordering::Equal)
            });

            Some(svcs)
        }
        Err(_) => None,
    };

    Ok(CostReportResponse {
        success: true,
        granularity: granularity_str.to_string(),
        start_date,
        end_date,
        total_amount,
        total_unit,
        data_points,
        services,
        error_message: None,
    })
}

/// Estimate days between two YYYY-MM-DD strings
fn estimate_days_between(start: &str, end: &str) -> i64 {
    let parse_date = |s: &str| -> i64 {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 { return 0; }
        let y: i64 = parts[0].parse().unwrap_or(0);
        let m: i64 = parts[1].parse().unwrap_or(1);
        let d: i64 = parts[2].parse().unwrap_or(1);
        y * 365 + m * 30 + d // rough estimate, good enough for threshold
    };
    (parse_date(end) - parse_date(start)).abs()
}
