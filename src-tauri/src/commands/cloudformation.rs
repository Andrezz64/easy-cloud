use serde::Serialize;
use aws_config::{Region, BehaviorVersion};
use aws_credential_types::Credentials;
use aws_sdk_cloudformation::Client as CfClient;

// ============================================================
// Types
// ============================================================

#[derive(Serialize)]
pub struct CfStack {
    name: String,
    status: String,
    creation_time: Option<String>,
}

#[derive(Serialize)]
pub struct CfListResponse {
    success: bool,
    stacks: Option<Vec<CfStack>>,
    error_message: Option<String>,
}

#[derive(Serialize)]
pub struct CfStackEvent {
    event_id: String,
    logical_resource_id: Option<String>,
    physical_resource_id: Option<String>,
    resource_type: Option<String>,
    resource_status: Option<String>,
    resource_status_reason: Option<String>,
    timestamp: Option<String>,
}

#[derive(Serialize)]
pub struct CfStackEventsResponse {
    success: bool,
    events: Option<Vec<CfStackEvent>>,
    error_message: Option<String>,
}

#[derive(Serialize)]
pub struct DeployResponse {
    success: bool,
    stack_id: Option<String>,
    error_message: Option<String>,
}

// ============================================================
// Helper
// ============================================================

async fn build_cf_client(access_key_id: &str, secret_access_key: &str, region: &str) -> CfClient {
    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "easy-cloud-app");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(Region::new(region.to_string()))
        .load()
        .await;
    CfClient::new(&config)
}

// ============================================================
// Commands
// ============================================================

#[tauri::command]
pub async fn list_cf_stacks(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<CfListResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.describe_stacks().send().await {
        Ok(response) => {
            let stacks: Vec<CfStack> = response.stacks().iter().map(|s| CfStack {
                name: s.stack_name().unwrap_or_default().to_string(),
                status: s.stack_status().map(|st| st.as_str().to_string()).unwrap_or_default(),
                creation_time: s.creation_time().map(|d| d.to_string()),
            }).collect();
            Ok(CfListResponse { success: true, stacks: Some(stacks), error_message: None })
        }
        Err(e) => Ok(CfListResponse { success: false, stacks: None, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn deploy_cf_stack(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
    template_body: String,
) -> Result<DeployResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.create_stack()
        .stack_name(&stack_name)
        .template_body(&template_body)
        .capabilities(aws_sdk_cloudformation::types::Capability::CapabilityNamedIam)
        .send()
        .await
    {
        Ok(response) => Ok(DeployResponse {
            success: true,
            stack_id: response.stack_id().map(|s| s.to_string()),
            error_message: None,
        }),
        Err(e) => Ok(DeployResponse {
            success: false,
            stack_id: None,
            error_message: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn describe_stack_events(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
) -> Result<CfStackEventsResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.describe_stack_events().stack_name(&stack_name).send().await {
        Ok(response) => {
            let events: Vec<CfStackEvent> = response.stack_events().iter().map(|e| CfStackEvent {
                event_id: e.event_id().unwrap_or_default().to_string(),
                logical_resource_id: e.logical_resource_id().map(|s| s.to_string()),
                physical_resource_id: e.physical_resource_id().map(|s| s.to_string()),
                resource_type: e.resource_type().map(|s| s.to_string()),
                resource_status: e.resource_status().map(|s| s.as_str().to_string()),
                resource_status_reason: e.resource_status_reason().map(|s| s.to_string()),
                timestamp: e.timestamp().map(|d| d.to_string()),
            }).collect();
            Ok(CfStackEventsResponse { success: true, events: Some(events), error_message: None })
        }
        Err(e) => Ok(CfStackEventsResponse { success: false, events: None, error_message: Some(e.to_string()) }),
    }
}

// ============================================================
// Extended Types
// ============================================================

#[derive(Serialize)]
pub struct CfStackOutput {
    pub key: Option<String>,
    pub value: Option<String>,
    pub description: Option<String>,
    pub export_name: Option<String>,
}

#[derive(Serialize)]
pub struct CfStackParameter {
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Serialize)]
pub struct CfStackDetails {
    pub success: bool,
    pub name: Option<String>,
    pub stack_id: Option<String>,
    pub status: Option<String>,
    pub status_reason: Option<String>,
    pub description: Option<String>,
    pub creation_time: Option<String>,
    pub last_updated_time: Option<String>,
    pub outputs: Option<Vec<CfStackOutput>>,
    pub parameters: Option<Vec<CfStackParameter>>,
    pub capabilities: Option<Vec<String>>,
    pub role_arn: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Serialize)]
pub struct CfStackResource {
    pub logical_resource_id: Option<String>,
    pub physical_resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub resource_status: Option<String>,
    pub resource_status_reason: Option<String>,
    pub last_updated: Option<String>,
}

#[derive(Serialize)]
pub struct CfStackResourcesResponse {
    pub success: bool,
    pub resources: Option<Vec<CfStackResource>>,
    pub error_message: Option<String>,
}

#[derive(Serialize)]
pub struct CfValidateResponse {
    pub success: bool,
    pub valid: bool,
    pub description: Option<String>,
    pub parameters: Option<Vec<String>>,
    pub capabilities: Option<Vec<String>>,
    pub error_message: Option<String>,
}

use super::GenericResponse;

// ============================================================
// Extended Commands
// ============================================================

/// Get full details of a specific stack
#[tauri::command]
pub async fn get_stack_details(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
) -> Result<CfStackDetails, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.describe_stacks().stack_name(&stack_name).send().await {
        Ok(response) => {
            if let Some(stack) = response.stacks().first() {
                let outputs: Vec<CfStackOutput> = stack.outputs().iter().map(|o| CfStackOutput {
                    key: o.output_key().map(|s| s.to_string()),
                    value: o.output_value().map(|s| s.to_string()),
                    description: o.description().map(|s| s.to_string()),
                    export_name: o.export_name().map(|s| s.to_string()),
                }).collect();

                let parameters: Vec<CfStackParameter> = stack.parameters().iter().map(|p| CfStackParameter {
                    key: p.parameter_key().map(|s| s.to_string()),
                    value: p.parameter_value().map(|s| s.to_string()),
                }).collect();

                let capabilities: Vec<String> = stack.capabilities().iter()
                    .map(|c| c.as_str().to_string())
                    .collect();

                Ok(CfStackDetails {
                    success: true,
                    name: stack.stack_name().map(|s| s.to_string()),
                    stack_id: stack.stack_id().map(|s| s.to_string()),
                    status: stack.stack_status().map(|s| s.as_str().to_string()),
                    status_reason: stack.stack_status_reason().map(|s| s.to_string()),
                    description: stack.description().map(|s| s.to_string()),
                    creation_time: stack.creation_time().map(|d| d.to_string()),
                    last_updated_time: stack.last_updated_time().map(|d| d.to_string()),
                    outputs: Some(outputs),
                    parameters: Some(parameters),
                    capabilities: Some(capabilities),
                    role_arn: stack.role_arn().map(|s| s.to_string()),
                    error_message: None,
                })
            } else {
                Ok(CfStackDetails {
                    success: false,
                    name: None, stack_id: None, status: None, status_reason: None,
                    description: None, creation_time: None, last_updated_time: None,
                    outputs: None, parameters: None, capabilities: None, role_arn: None,
                    error_message: Some("Stack not found".to_string()),
                })
            }
        }
        Err(e) => Ok(CfStackDetails {
            success: false,
            name: None, stack_id: None, status: None, status_reason: None,
            description: None, creation_time: None, last_updated_time: None,
            outputs: None, parameters: None, capabilities: None, role_arn: None,
            error_message: Some(e.to_string()),
        }),
    }
}

/// List all resources in a stack
#[tauri::command]
pub async fn list_stack_resources(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
) -> Result<CfStackResourcesResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.list_stack_resources().stack_name(&stack_name).send().await {
        Ok(response) => {
            let resources: Vec<CfStackResource> = response.stack_resource_summaries().iter().map(|r| CfStackResource {
                logical_resource_id: r.logical_resource_id().map(|s| s.to_string()),
                physical_resource_id: r.physical_resource_id().map(|s| s.to_string()),
                resource_type: r.resource_type().map(|s| s.to_string()),
                resource_status: r.resource_status().map(|s| s.as_str().to_string()),
                resource_status_reason: r.resource_status_reason().map(|s| s.to_string()),
                last_updated: r.last_updated_timestamp().map(|d| d.to_string()),
            }).collect();
            Ok(CfStackResourcesResponse { success: true, resources: Some(resources), error_message: None })
        }
        Err(e) => Ok(CfStackResourcesResponse { success: false, resources: None, error_message: Some(e.to_string()) }),
    }
}

/// Delete a stack
#[tauri::command]
pub async fn delete_cf_stack(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
) -> Result<GenericResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.delete_stack().stack_name(&stack_name).send().await {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

/// Update a stack with a new template
#[tauri::command]
pub async fn update_cf_stack(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
    template_body: String,
) -> Result<DeployResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.update_stack()
        .stack_name(&stack_name)
        .template_body(&template_body)
        .capabilities(aws_sdk_cloudformation::types::Capability::CapabilityNamedIam)
        .send()
        .await
    {
        Ok(response) => Ok(DeployResponse {
            success: true,
            stack_id: response.stack_id().map(|s| s.to_string()),
            error_message: None,
        }),
        Err(e) => Ok(DeployResponse {
            success: false,
            stack_id: None,
            error_message: Some(e.to_string()),
        }),
    }
}

/// Validate a CloudFormation template
#[tauri::command]
pub async fn validate_cf_template(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    template_body: String,
) -> Result<CfValidateResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.validate_template()
        .template_body(&template_body)
        .send()
        .await
    {
        Ok(response) => {
            let params: Vec<String> = response.parameters().iter()
                .filter_map(|p| p.parameter_key().map(|s| s.to_string()))
                .collect();
            let caps: Vec<String> = response.capabilities().iter()
                .map(|c| c.as_str().to_string())
                .collect();

            Ok(CfValidateResponse {
                success: true,
                valid: true,
                description: response.description().map(|s| s.to_string()),
                parameters: Some(params),
                capabilities: Some(caps),
                error_message: None,
            })
        }
        Err(e) => Ok(CfValidateResponse {
            success: true,
            valid: false,
            description: None,
            parameters: None,
            capabilities: None,
            error_message: Some(e.to_string()),
        }),
    }
}
