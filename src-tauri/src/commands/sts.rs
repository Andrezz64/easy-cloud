use serde::Serialize;
use aws_config::{Region, BehaviorVersion};
use aws_credential_types::Credentials;
use aws_sdk_sts::Client as StsClient;

#[derive(Serialize)]
pub struct VerifyResponse {
    success: bool,
    account: Option<String>,
    arn: Option<String>,
    error_message: Option<String>,
}

#[tauri::command]
pub async fn verify_aws_credentials(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<VerifyResponse, String> {
    let credentials = Credentials::new(
        access_key_id,
        secret_access_key,
        None,
        None,
        "easy-cloud-app",
    );

    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(Region::new(region))
        .load()
        .await;

    let sts_client = StsClient::new(&config);

    match sts_client.get_caller_identity().send().await {
        Ok(response) => Ok(VerifyResponse {
            success: true,
            account: response.account().map(String::from),
            arn: response.arn().map(String::from),
            error_message: None,
        }),
        Err(e) => Ok(VerifyResponse {
            success: false,
            account: None,
            arn: None,
            error_message: Some(e.to_string()),
        }),
    }
}
