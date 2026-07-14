mod commands;

use commands::sts::*;
use commands::s3::*;
use commands::cloudformation::*;
use commands::billing::*;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            // STS
            verify_aws_credentials,
            // S3
            list_s3_buckets,
            get_s3_bucket_details,
            list_s3_objects,
            upload_s3_object,
            download_s3_object,
            delete_s3_object,
            delete_s3_objects,
            create_s3_folder,
            head_s3_object,
            get_s3_bucket_policy,
            put_s3_bucket_policy,
            get_s3_bucket_tags,
            put_s3_bucket_tags,
            put_s3_bucket_versioning,
            preview_s3_object,
            copy_s3_object,
            move_s3_object,
            multipart_upload_s3_object,
            generate_presigned_url,
            put_s3_object_content,
            // CloudFormation
            list_cf_stacks,
            deploy_cf_stack,
            describe_stack_events,
            get_stack_details,
            list_stack_resources,
            delete_cf_stack,
            update_cf_stack,
            validate_cf_template,
            // Billing / Cost Explorer
            get_monthly_cost_by_service,
            get_daily_costs,
            get_cost_forecast,
            get_cost_summary,
            get_usage_by_service,
            get_cost_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
