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

// ============================================================
// Get Template
// ============================================================

#[derive(Serialize)]
pub struct CfGetTemplateResponse {
    pub success: bool,
    pub template_body: Option<String>,
    pub stages_available: Option<Vec<String>>,
    pub error_message: Option<String>,
}

/// Get the template body of a deployed stack
#[tauri::command]
pub async fn get_stack_template(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
) -> Result<CfGetTemplateResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.get_template()
        .stack_name(&stack_name)
        .send()
        .await
    {
        Ok(response) => Ok(CfGetTemplateResponse {
            success: true,
            template_body: response.template_body().map(|s| s.to_string()),
            stages_available: Some(
                response.stages_available().iter().map(|s| s.as_str().to_string()).collect()
            ),
            error_message: None,
        }),
        Err(e) => Ok(CfGetTemplateResponse {
            success: false,
            template_body: None,
            stages_available: None,
            error_message: Some(e.to_string()),
        }),
    }
}

// ============================================================
// Drift Detection
// ============================================================

#[derive(Serialize)]
pub struct CfDriftDetectionResponse {
    pub success: bool,
    pub detection_id: Option<String>,
    pub error_message: Option<String>,
}

/// Initiate drift detection on a stack
#[tauri::command]
pub async fn detect_stack_drift(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
) -> Result<CfDriftDetectionResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.detect_stack_drift()
        .stack_name(&stack_name)
        .send()
        .await
    {
        Ok(response) => Ok(CfDriftDetectionResponse {
            success: true,
            detection_id: response.stack_drift_detection_id().map(|s| s.to_string()),
            error_message: None,
        }),
        Err(e) => Ok(CfDriftDetectionResponse {
            success: false,
            detection_id: None,
            error_message: Some(e.to_string()),
        }),
    }
}

#[derive(Serialize)]
pub struct CfDriftStatusResponse {
    pub success: bool,
    pub detection_status: Option<String>,
    pub drift_status: Option<String>,
    pub drifted_stack_resource_count: Option<i32>,
    pub error_message: Option<String>,
}

/// Check the status of a drift detection operation
#[tauri::command]
pub async fn describe_drift_detection_status(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    detection_id: String,
) -> Result<CfDriftStatusResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.describe_stack_drift_detection_status()
        .stack_drift_detection_id(&detection_id)
        .send()
        .await
    {
        Ok(response) => Ok(CfDriftStatusResponse {
            success: true,
            detection_status: response.detection_status().map(|s| s.as_str().to_string()),
            drift_status: response.stack_drift_status().map(|s| s.as_str().to_string()),
            drifted_stack_resource_count: response.drifted_stack_resource_count(),
            error_message: None,
        }),
        Err(e) => Ok(CfDriftStatusResponse {
            success: false,
            detection_status: None,
            drift_status: None,
            drifted_stack_resource_count: None,
            error_message: Some(e.to_string()),
        }),
    }
}

#[derive(Serialize)]
pub struct CfResourceDrift {
    pub logical_resource_id: Option<String>,
    pub physical_resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub drift_status: Option<String>,
    pub expected_properties: Option<String>,
    pub actual_properties: Option<String>,
}

#[derive(Serialize)]
pub struct CfResourceDriftsResponse {
    pub success: bool,
    pub drifts: Option<Vec<CfResourceDrift>>,
    pub error_message: Option<String>,
}

/// Get detailed drift information for each resource
#[tauri::command]
pub async fn describe_stack_resource_drifts(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
) -> Result<CfResourceDriftsResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.describe_stack_resource_drifts()
        .stack_name(&stack_name)
        .send()
        .await
    {
        Ok(response) => {
            let drifts: Vec<CfResourceDrift> = response.stack_resource_drifts().iter().map(|d| {
                CfResourceDrift {
                    logical_resource_id: d.logical_resource_id().map(|s| s.to_string()),
                    physical_resource_id: d.physical_resource_id().map(|s| s.to_string()),
                    resource_type: d.resource_type().map(|s| s.to_string()),
                    drift_status: d.stack_resource_drift_status().map(|s| s.as_str().to_string()),
                    expected_properties: d.expected_properties().map(|s| s.to_string()),
                    actual_properties: d.actual_properties().map(|s| s.to_string()),
                }
            }).collect();
            Ok(CfResourceDriftsResponse { success: true, drifts: Some(drifts), error_message: None })
        }
        Err(e) => Ok(CfResourceDriftsResponse { success: false, drifts: None, error_message: Some(e.to_string()) }),
    }
}

// ============================================================
// Change Sets
// ============================================================

#[derive(Serialize)]
pub struct CfChangeSetSummary {
    pub change_set_id: Option<String>,
    pub change_set_name: Option<String>,
    pub status: Option<String>,
    pub execution_status: Option<String>,
    pub description: Option<String>,
    pub creation_time: Option<String>,
}

#[derive(Serialize)]
pub struct CfListChangeSetsResponse {
    pub success: bool,
    pub change_sets: Option<Vec<CfChangeSetSummary>>,
    pub error_message: Option<String>,
}

/// List all change sets for a stack
#[tauri::command]
pub async fn list_change_sets(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
) -> Result<CfListChangeSetsResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.list_change_sets().stack_name(&stack_name).send().await {
        Ok(response) => {
            let sets: Vec<CfChangeSetSummary> = response.summaries().iter().map(|cs| CfChangeSetSummary {
                change_set_id: cs.change_set_id().map(|s| s.to_string()),
                change_set_name: cs.change_set_name().map(|s| s.to_string()),
                status: cs.status().map(|s| s.as_str().to_string()),
                execution_status: cs.execution_status().map(|s| s.as_str().to_string()),
                description: cs.description().map(|s| s.to_string()),
                creation_time: cs.creation_time().map(|d| d.to_string()),
            }).collect();
            Ok(CfListChangeSetsResponse { success: true, change_sets: Some(sets), error_message: None })
        }
        Err(e) => Ok(CfListChangeSetsResponse { success: false, change_sets: None, error_message: Some(e.to_string()) }),
    }
}

#[derive(Serialize)]
pub struct CfChange {
    pub action: Option<String>,
    pub logical_resource_id: Option<String>,
    pub physical_resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub replacement: Option<String>,
}

#[derive(Serialize)]
pub struct CfDescribeChangeSetResponse {
    pub success: bool,
    pub change_set_name: Option<String>,
    pub change_set_id: Option<String>,
    pub stack_name: Option<String>,
    pub status: Option<String>,
    pub execution_status: Option<String>,
    pub status_reason: Option<String>,
    pub changes: Option<Vec<CfChange>>,
    pub error_message: Option<String>,
}

/// Describe a specific change set (preview changes)
#[tauri::command]
pub async fn describe_change_set(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
    change_set_name: String,
) -> Result<CfDescribeChangeSetResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.describe_change_set()
        .stack_name(&stack_name)
        .change_set_name(&change_set_name)
        .send()
        .await
    {
        Ok(response) => {
            let changes: Vec<CfChange> = response.changes().iter().filter_map(|c| {
                c.resource_change().map(|rc| CfChange {
                    action: rc.action().map(|a| a.as_str().to_string()),
                    logical_resource_id: rc.logical_resource_id().map(|s| s.to_string()),
                    physical_resource_id: rc.physical_resource_id().map(|s| s.to_string()),
                    resource_type: rc.resource_type().map(|s| s.to_string()),
                    replacement: rc.replacement().map(|r| r.as_str().to_string()),
                })
            }).collect();

            Ok(CfDescribeChangeSetResponse {
                success: true,
                change_set_name: response.change_set_name().map(|s| s.to_string()),
                change_set_id: response.change_set_id().map(|s| s.to_string()),
                stack_name: response.stack_name().map(|s| s.to_string()),
                status: response.status().map(|s| s.as_str().to_string()),
                execution_status: response.execution_status().map(|s| s.as_str().to_string()),
                status_reason: response.status_reason().map(|s| s.to_string()),
                changes: Some(changes),
                error_message: None,
            })
        }
        Err(e) => Ok(CfDescribeChangeSetResponse {
            success: false,
            change_set_name: None, change_set_id: None, stack_name: None,
            status: None, execution_status: None, status_reason: None,
            changes: None, error_message: Some(e.to_string()),
        }),
    }
}

/// Create a change set (preview update without applying)
#[tauri::command]
pub async fn create_change_set(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
    change_set_name: String,
    template_body: String,
    description: Option<String>,
) -> Result<CfDriftDetectionResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    let mut req = client.create_change_set()
        .stack_name(&stack_name)
        .change_set_name(&change_set_name)
        .template_body(&template_body)
        .capabilities(aws_sdk_cloudformation::types::Capability::CapabilityNamedIam);

    if let Some(desc) = description {
        req = req.description(&desc);
    }

    match req.send().await {
        Ok(response) => Ok(CfDriftDetectionResponse {
            success: true,
            detection_id: response.id().map(|s| s.to_string()),
            error_message: None,
        }),
        Err(e) => Ok(CfDriftDetectionResponse {
            success: false,
            detection_id: None,
            error_message: Some(e.to_string()),
        }),
    }
}

/// Execute an existing change set
#[tauri::command]
pub async fn execute_change_set(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
    change_set_name: String,
) -> Result<GenericResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.execute_change_set()
        .stack_name(&stack_name)
        .change_set_name(&change_set_name)
        .send()
        .await
    {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

/// Delete a change set
#[tauri::command]
pub async fn delete_change_set(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    stack_name: String,
    change_set_name: String,
) -> Result<GenericResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.delete_change_set()
        .stack_name(&stack_name)
        .change_set_name(&change_set_name)
        .send()
        .await
    {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

// ============================================================
// Exports & Imports
// ============================================================

#[derive(Serialize)]
pub struct CfExport {
    pub name: Option<String>,
    pub value: Option<String>,
    pub exporting_stack_id: Option<String>,
}

#[derive(Serialize)]
pub struct CfExportsResponse {
    pub success: bool,
    pub exports: Option<Vec<CfExport>>,
    pub error_message: Option<String>,
}

/// List all CloudFormation exports in the region
#[tauri::command]
pub async fn list_cf_exports(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<CfExportsResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.list_exports().send().await {
        Ok(response) => {
            let exports: Vec<CfExport> = response.exports().iter().map(|e| CfExport {
                name: e.name().map(|s| s.to_string()),
                value: e.value().map(|s| s.to_string()),
                exporting_stack_id: e.exporting_stack_id().map(|s| s.to_string()),
            }).collect();
            Ok(CfExportsResponse { success: true, exports: Some(exports), error_message: None })
        }
        Err(e) => Ok(CfExportsResponse { success: false, exports: None, error_message: Some(e.to_string()) }),
    }
}

#[derive(Serialize)]
pub struct CfImportsResponse {
    pub success: bool,
    pub imports: Option<Vec<String>>,
    pub error_message: Option<String>,
}

/// List stacks that import a given export
#[tauri::command]
pub async fn list_cf_imports(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    export_name: String,
) -> Result<CfImportsResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.list_imports().export_name(&export_name).send().await {
        Ok(response) => {
            let imports: Vec<String> = response.imports().iter().map(|s| s.to_string()).collect();
            Ok(CfImportsResponse { success: true, imports: Some(imports), error_message: None })
        }
        Err(e) => Ok(CfImportsResponse { success: false, imports: None, error_message: Some(e.to_string()) }),
    }
}

// ============================================================
// Estimate Template Cost
// ============================================================

#[derive(Serialize)]
pub struct CfEstimateCostResponse {
    pub success: bool,
    pub url: Option<String>,
    pub error_message: Option<String>,
}

/// Get an AWS Calculator URL estimating the cost of a template
#[tauri::command]
pub async fn estimate_template_cost(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    template_body: String,
) -> Result<CfEstimateCostResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.estimate_template_cost()
        .template_body(&template_body)
        .send()
        .await
    {
        Ok(response) => Ok(CfEstimateCostResponse {
            success: true,
            url: response.url().map(|s| s.to_string()),
            error_message: None,
        }),
        Err(e) => Ok(CfEstimateCostResponse {
            success: false,
            url: None,
            error_message: Some(e.to_string()),
        }),
    }
}

// ============================================================
// Get Template Summary (preview without deploy)
// ============================================================

#[derive(Serialize)]
pub struct CfTemplateSummaryParam {
    pub key: Option<String>,
    pub default_value: Option<String>,
    pub param_type: Option<String>,
    pub description: Option<String>,
    pub no_echo: bool,
}

#[derive(Serialize)]
pub struct CfTemplateSummaryResponse {
    pub success: bool,
    pub description: Option<String>,
    pub resource_types: Option<Vec<String>>,
    pub parameters: Option<Vec<CfTemplateSummaryParam>>,
    pub capabilities: Option<Vec<String>>,
    pub error_message: Option<String>,
}

/// Get a summary of a template (resources, params, etc.) without deploying
#[tauri::command]
pub async fn get_template_summary(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    template_body: String,
) -> Result<CfTemplateSummaryResponse, String> {
    let client = build_cf_client(&access_key_id, &secret_access_key, &region).await;

    match client.get_template_summary()
        .template_body(&template_body)
        .send()
        .await
    {
        Ok(response) => {
            let params: Vec<CfTemplateSummaryParam> = response.parameters().iter().map(|p| {
                CfTemplateSummaryParam {
                    key: p.parameter_key().map(|s| s.to_string()),
                    default_value: p.default_value().map(|s| s.to_string()),
                    param_type: p.parameter_type().map(|s| s.to_string()),
                    description: p.description().map(|s| s.to_string()),
                    no_echo: p.no_echo().unwrap_or(false),
                }
            }).collect();

            let resource_types: Vec<String> = response.resource_types().iter().map(|s| s.to_string()).collect();
            let capabilities: Vec<String> = response.capabilities().iter().map(|c| c.as_str().to_string()).collect();

            Ok(CfTemplateSummaryResponse {
                success: true,
                description: response.description().map(|s| s.to_string()),
                resource_types: Some(resource_types),
                parameters: Some(params),
                capabilities: Some(capabilities),
                error_message: None,
            })
        }
        Err(e) => Ok(CfTemplateSummaryResponse {
            success: false,
            description: None,
            resource_types: None,
            parameters: None,
            capabilities: None,
            error_message: Some(e.to_string()),
        }),
    }
}
