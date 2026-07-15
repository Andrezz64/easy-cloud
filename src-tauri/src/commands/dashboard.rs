use serde::Serialize;
use aws_config::{Region, BehaviorVersion};
use aws_credential_types::Credentials;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_cloudformation::Client as CfClient;

// ============================================================
// Types
// ============================================================

#[derive(Serialize)]
pub struct DashboardBucket {
    pub name: String,
    pub creation_date: Option<String>,
}

#[derive(Serialize)]
pub struct DashboardStack {
    pub name: String,
    pub status: String,
    pub creation_time: Option<String>,
}

#[derive(Serialize)]
pub struct DashboardSummary {
    pub success: bool,
    pub buckets_count: u32,
    pub buckets: Vec<DashboardBucket>,
    pub stacks_count: u32,
    pub stacks: Vec<DashboardStack>,
    pub active_stacks: u32,
    pub failed_stacks: u32,
    pub error_message: Option<String>,
}

// ============================================================
// Command
// ============================================================

/// Fetch a consolidated dashboard summary: bucket count, stack count + statuses.
/// This is a lightweight aggregation of data from S3 and CloudFormation.
#[tauri::command]
pub async fn get_dashboard_summary(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<DashboardSummary, String> {
    let credentials = Credentials::new(&access_key_id, &secret_access_key, None, None, "easy-cloud-app");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(Region::new(region))
        .load()
        .await;

    let s3_client = S3Client::new(&config);
    let cf_client = CfClient::new(&config);

    // Fetch buckets
    let buckets_result = s3_client.list_buckets().send().await;
    let (buckets, buckets_count) = match buckets_result {
        Ok(resp) => {
            let b: Vec<DashboardBucket> = resp.buckets().iter().map(|b| DashboardBucket {
                name: b.name().unwrap_or_default().to_string(),
                creation_date: b.creation_date().map(|d| d.to_string()),
            }).collect();
            let count = b.len() as u32;
            (b, count)
        }
        Err(_) => (Vec::new(), 0),
    };

    // Fetch stacks
    let stacks_result = cf_client.describe_stacks().send().await;
    let (stacks, stacks_count, active_stacks, failed_stacks) = match stacks_result {
        Ok(resp) => {
            let s: Vec<DashboardStack> = resp.stacks().iter().map(|s| DashboardStack {
                name: s.stack_name().unwrap_or_default().to_string(),
                status: s.stack_status().map(|st| st.as_str().to_string()).unwrap_or_default(),
                creation_time: s.creation_time().map(|d| d.to_string()),
            }).collect();
            let count = s.len() as u32;
            let active = s.iter().filter(|st| {
                st.status.contains("COMPLETE") && !st.status.contains("DELETE")
            }).count() as u32;
            let failed = s.iter().filter(|st| {
                st.status.contains("FAILED") || st.status.contains("ROLLBACK")
            }).count() as u32;
            (s, count, active, failed)
        }
        Err(_) => (Vec::new(), 0, 0, 0),
    };

    Ok(DashboardSummary {
        success: true,
        buckets_count,
        buckets,
        stacks_count,
        stacks,
        active_stacks,
        failed_stacks,
        error_message: None,
    })
}
