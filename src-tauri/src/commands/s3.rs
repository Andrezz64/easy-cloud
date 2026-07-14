use serde::Serialize;
use aws_config::{Region, BehaviorVersion};
use aws_credential_types::Credentials;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;
use std::path::PathBuf;

use super::GenericResponse;

// ============================================================
// Types
// ============================================================

#[derive(Serialize)]
pub struct S3Bucket {
    name: String,
    creation_date: Option<String>,
}

#[derive(Serialize)]
pub struct S3ListResponse {
    success: bool,
    buckets: Option<Vec<S3Bucket>>,
    error_message: Option<String>,
}

#[derive(Serialize)]
pub struct S3BucketDetails {
    name: String,
    region: Option<String>,
    versioning: Option<String>,
    encryption: Option<String>,
}

#[derive(Serialize)]
pub struct S3Object {
    key: String,
    size: i64,
    last_modified: Option<String>,
    is_folder: bool,
}

#[derive(Serialize)]
pub struct S3ListObjectsResponse {
    success: bool,
    objects: Option<Vec<S3Object>>,
    prefixes: Option<Vec<String>>,
    error_message: Option<String>,
}

#[derive(Serialize)]
pub struct S3UploadResponse {
    success: bool,
    key: Option<String>,
    error_message: Option<String>,
}

#[derive(Serialize)]
pub struct S3DownloadResponse {
    success: bool,
    file_path: Option<String>,
    error_message: Option<String>,
}

#[derive(Serialize)]
pub struct S3HeadObjectResponse {
    success: bool,
    size: Option<i64>,
    content_type: Option<String>,
    last_modified: Option<String>,
    etag: Option<String>,
    error_message: Option<String>,
}

#[derive(Serialize)]
pub struct S3PolicyResponse {
    success: bool,
    policy: Option<String>,
    error_message: Option<String>,
}

#[derive(Serialize, serde::Deserialize)]
pub struct S3Tag {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct S3TagsResponse {
    success: bool,
    tags: Option<Vec<S3Tag>>,
    error_message: Option<String>,
}

#[derive(Serialize)]
pub struct S3PreviewResponse {
    success: bool,
    content: Option<String>,
    content_type: Option<String>,
    is_binary: bool,
    size: Option<i64>,
    error_message: Option<String>,
}

// ============================================================
// Helper
// ============================================================

async fn build_s3_client(access_key_id: &str, secret_access_key: &str, region: &str) -> S3Client {
    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "easy-cloud-app");
    let config = aws_config::defaults(BehaviorVersion::latest())
        .credentials_provider(credentials)
        .region(Region::new(region.to_string()))
        .load()
        .await;
    S3Client::new(&config)
}

// ============================================================
// Commands
// ============================================================

#[tauri::command]
pub async fn list_s3_buckets(
    access_key_id: String,
    secret_access_key: String,
    region: String,
) -> Result<S3ListResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    match client.list_buckets().send().await {
        Ok(response) => {
            let buckets: Vec<S3Bucket> = response.buckets().iter().map(|b| S3Bucket {
                name: b.name().unwrap_or_default().to_string(),
                creation_date: b.creation_date().map(|d| d.to_string()),
            }).collect();
            Ok(S3ListResponse { success: true, buckets: Some(buckets), error_message: None })
        }
        Err(e) => Ok(S3ListResponse { success: false, buckets: None, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn get_s3_bucket_details(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
) -> Result<S3BucketDetails, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let location = match client.get_bucket_location().bucket(&bucket_name).send().await {
        Ok(resp) => {
            let loc = resp.location_constraint().map(|l| l.as_str().to_string());
            match loc {
                Some(l) if !l.is_empty() => Some(l),
                _ => Some(region.clone()),
            }
        }
        Err(_) => Some(region.clone()),
    };

    let versioning = match client.get_bucket_versioning().bucket(&bucket_name).send().await {
        Ok(resp) => resp.status().map(|s| s.as_str().to_string()),
        Err(_) => None,
    };

    let encryption = match client.get_bucket_encryption().bucket(&bucket_name).send().await {
        Ok(resp) => {
            resp.server_side_encryption_configuration()
                .and_then(|c| c.rules().first())
                .and_then(|r| r.apply_server_side_encryption_by_default())
                .map(|d| d.sse_algorithm().as_str().to_string())
        }
        Err(_) => None,
    };

    Ok(S3BucketDetails { name: bucket_name, region: location, versioning, encryption })
}

#[tauri::command]
pub async fn list_s3_objects(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    prefix: Option<String>,
) -> Result<S3ListObjectsResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let mut request = client.list_objects_v2()
        .bucket(&bucket_name)
        .delimiter("/")
        .max_keys(1000);

    if let Some(ref p) = prefix {
        request = request.prefix(p);
    }

    match request.send().await {
        Ok(response) => {
            let mut objects = Vec::new();
            let mut prefixes = Vec::new();

            if let Some(common_prefixes) = response.common_prefixes.as_ref() {
                for cp in common_prefixes {
                    if let Some(p) = cp.prefix() {
                        prefixes.push(p.to_string());
                        objects.push(S3Object {
                            key: p.to_string(),
                            size: 0,
                            last_modified: None,
                            is_folder: true,
                        });
                    }
                }
            }

            if let Some(contents) = response.contents.as_ref() {
                for obj in contents {
                    let key = obj.key().unwrap_or_default().to_string();
                    if prefix.as_deref() == Some(&key) {
                        continue;
                    }
                    objects.push(S3Object {
                        key,
                        size: obj.size().unwrap_or(0),
                        last_modified: obj.last_modified().map(|d| d.to_string()),
                        is_folder: false,
                    });
                }
            }

            Ok(S3ListObjectsResponse {
                success: true,
                objects: Some(objects),
                prefixes: Some(prefixes),
                error_message: None,
            })
        }
        Err(e) => Ok(S3ListObjectsResponse {
            success: false,
            objects: None,
            prefixes: None,
            error_message: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn upload_s3_object(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    key: String,
    file_path: String,
) -> Result<S3UploadResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let file_bytes = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let body = ByteStream::from(file_bytes);

    // Infer content type from the key (file extension)
    let content_type = mime_from_extension(&key).unwrap_or("application/octet-stream");

    match client.put_object()
        .bucket(&bucket_name)
        .key(&key)
        .content_type(content_type)
        .body(body)
        .send()
        .await
    {
        Ok(_) => Ok(S3UploadResponse { success: true, key: Some(key), error_message: None }),
        Err(e) => Ok(S3UploadResponse { success: false, key: None, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn download_s3_object(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    key: String,
    destination_path: String,
) -> Result<S3DownloadResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    match client.get_object().bucket(&bucket_name).key(&key).send().await {
        Ok(response) => {
            let bytes = response.body.collect().await
                .map_err(|e| format!("Failed to read response body: {}", e))?;

            let dest = PathBuf::from(&destination_path);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            std::fs::write(&dest, bytes.into_bytes())
                .map_err(|e| format!("Failed to write file: {}", e))?;

            Ok(S3DownloadResponse { success: true, file_path: Some(destination_path), error_message: None })
        }
        Err(e) => Ok(S3DownloadResponse { success: false, file_path: None, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn delete_s3_object(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    key: String,
) -> Result<GenericResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    match client.delete_object().bucket(&bucket_name).key(&key).send().await {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn delete_s3_objects(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    keys: Vec<String>,
) -> Result<GenericResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let objects: Vec<aws_sdk_s3::types::ObjectIdentifier> = keys
        .iter()
        .filter_map(|k| {
            aws_sdk_s3::types::ObjectIdentifier::builder()
                .key(k)
                .build()
                .ok()
        })
        .collect();

    let delete = aws_sdk_s3::types::Delete::builder()
        .set_objects(Some(objects))
        .build()
        .map_err(|e| format!("Failed to build delete request: {}", e))?;

    match client.delete_objects().bucket(&bucket_name).delete(delete).send().await {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn create_s3_folder(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    folder_key: String,
) -> Result<GenericResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let key = if folder_key.ends_with('/') {
        folder_key
    } else {
        format!("{}/", folder_key)
    };

    match client.put_object().bucket(&bucket_name).key(&key).body(ByteStream::from_static(b"")).send().await {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn head_s3_object(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    key: String,
) -> Result<S3HeadObjectResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    match client.head_object().bucket(&bucket_name).key(&key).send().await {
        Ok(resp) => Ok(S3HeadObjectResponse {
            success: true,
            size: resp.content_length(),
            content_type: resp.content_type().map(|s| s.to_string()),
            last_modified: resp.last_modified().map(|d| d.to_string()),
            etag: resp.e_tag().map(|s| s.to_string()),
            error_message: None,
        }),
        Err(e) => Ok(S3HeadObjectResponse {
            success: false,
            size: None,
            content_type: None,
            last_modified: None,
            etag: None,
            error_message: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn get_s3_bucket_policy(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
) -> Result<S3PolicyResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    match client.get_bucket_policy().bucket(&bucket_name).send().await {
        Ok(resp) => Ok(S3PolicyResponse { success: true, policy: resp.policy, error_message: None }),
        Err(e) => {
            let msg = format!("{:?}", e);
            if msg.contains("NoSuchBucketPolicy") || msg.contains("The bucket policy does not exist") || msg.contains("NoSuch") {
                Ok(S3PolicyResponse { success: true, policy: None, error_message: None })
            } else {
                Ok(S3PolicyResponse { success: false, policy: None, error_message: Some(format!("{}", e)) })
            }
        }
    }
}

#[tauri::command]
pub async fn put_s3_bucket_policy(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    policy: String,
) -> Result<GenericResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    match client.put_bucket_policy().bucket(&bucket_name).policy(&policy).send().await {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn get_s3_bucket_tags(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
) -> Result<S3TagsResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    match client.get_bucket_tagging().bucket(&bucket_name).send().await {
        Ok(resp) => {
            let tags: Vec<S3Tag> = resp.tag_set().iter().map(|t| S3Tag {
                key: t.key().to_string(),
                value: t.value().to_string(),
            }).collect();
            Ok(S3TagsResponse { success: true, tags: Some(tags), error_message: None })
        }
        Err(e) => {
            let msg = format!("{:?}", e);
            if msg.contains("NoSuchTagSet") || msg.contains("NoSuch") {
                Ok(S3TagsResponse { success: true, tags: Some(Vec::new()), error_message: None })
            } else {
                Ok(S3TagsResponse { success: false, tags: None, error_message: Some(format!("{}", e)) })
            }
        }
    }
}

#[tauri::command]
pub async fn put_s3_bucket_tags(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    tags: Vec<S3Tag>,
) -> Result<GenericResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let tag_set: Vec<aws_sdk_s3::types::Tag> = tags.iter().filter_map(|t| {
        aws_sdk_s3::types::Tag::builder()
            .key(&t.key)
            .value(&t.value)
            .build()
            .ok()
    }).collect();

    let tagging = aws_sdk_s3::types::Tagging::builder()
        .set_tag_set(Some(tag_set))
        .build()
        .map_err(|e| format!("Failed to build tagging: {}", e))?;

    match client.put_bucket_tagging().bucket(&bucket_name).tagging(tagging).send().await {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

#[tauri::command]
pub async fn put_s3_bucket_versioning(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    enabled: bool,
) -> Result<GenericResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let status = if enabled {
        aws_sdk_s3::types::BucketVersioningStatus::Enabled
    } else {
        aws_sdk_s3::types::BucketVersioningStatus::Suspended
    };

    let vc = aws_sdk_s3::types::VersioningConfiguration::builder()
        .status(status)
        .build();

    match client.put_bucket_versioning().bucket(&bucket_name).versioning_configuration(vc).send().await {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

/// Infer MIME type from file extension when S3 returns application/octet-stream
fn mime_from_extension(key: &str) -> Option<&'static str> {
    let ext = key.rsplit('.').next()?.to_lowercase();
    let ext_str = ext.as_str();
    match ext_str {
        // Text
        "txt" | "log" | "md" | "markdown" => Some("text/plain"),
        "html" | "htm" => Some("text/html"),
        "css" => Some("text/css"),
        "csv" => Some("text/csv"),
        "xml" | "svg" | "xsl" => Some("text/xml"),
        // Code / config
        "js" | "mjs" | "cjs" => Some("application/javascript"),
        "ts" | "tsx" => Some("text/typescript"),
        "jsx" => Some("text/jsx"),
        "json" | "jsonl" => Some("application/json"),
        "yaml" | "yml" => Some("text/yaml"),
        "toml" => Some("text/toml"),
        "ini" | "cfg" | "conf" => Some("text/plain"),
        "env" => Some("text/plain"),
        "sh" | "bash" | "zsh" | "fish" => Some("text/x-shellscript"),
        "py" => Some("text/x-python"),
        "rs" => Some("text/x-rust"),
        "go" => Some("text/x-go"),
        "java" => Some("text/x-java"),
        "rb" => Some("text/x-ruby"),
        "php" => Some("text/x-php"),
        "c" | "h" => Some("text/x-c"),
        "cpp" | "cc" | "cxx" | "hpp" => Some("text/x-c++"),
        "cs" => Some("text/x-csharp"),
        "swift" => Some("text/x-swift"),
        "kt" | "kts" => Some("text/x-kotlin"),
        "sql" => Some("text/x-sql"),
        "graphql" | "gql" => Some("text/x-graphql"),
        "dockerfile" => Some("text/plain"),
        "makefile" => Some("text/plain"),
        "tf" | "tfvars" => Some("text/plain"),
        "vue" => Some("text/html"),
        "svelte" => Some("text/html"),
        // Images
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "bmp" => Some("image/bmp"),
        "ico" => Some("image/x-icon"),
        "avif" => Some("image/avif"),
        // PDF
        "pdf" => Some("application/pdf"),
        _ => None,
    }
}

/// Determine effective content type using S3 metadata + extension fallback
fn effective_content_type(s3_content_type: &str, key: &str) -> String {
    if s3_content_type == "application/octet-stream" || s3_content_type == "binary/octet-stream" {
        mime_from_extension(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| s3_content_type.to_string())
    } else {
        s3_content_type.to_string()
    }
}

fn is_text_content_type(ct: &str) -> bool {
    ct.starts_with("text/")
        || ct.contains("json")
        || ct.contains("xml")
        || ct.contains("yaml")
        || ct.contains("javascript")
        || ct.contains("csv")
        || ct.contains("x-python")
        || ct.contains("x-rust")
        || ct.contains("x-go")
        || ct.contains("x-java")
        || ct.contains("x-ruby")
        || ct.contains("x-php")
        || ct.contains("x-c")
        || ct.contains("x-shellscript")
        || ct.contains("x-sql")
        || ct.contains("x-graphql")
        || ct.contains("x-kotlin")
        || ct.contains("x-swift")
        || ct.contains("x-csharp")
        || ct.contains("toml")
}

#[tauri::command]
pub async fn preview_s3_object(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    key: String,
) -> Result<S3PreviewResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    // Head to check size and type
    let head = client.head_object().bucket(&bucket_name).key(&key).send().await
        .map_err(|e| e.to_string())?;

    let raw_content_type = head.content_type().unwrap_or("application/octet-stream").to_string();
    let content_type = effective_content_type(&raw_content_type, &key);
    let size = head.content_length().unwrap_or(0);

    if size > 5_000_000 {
        return Ok(S3PreviewResponse {
            success: true,
            content: None,
            content_type: Some(content_type),
            is_binary: true,
            size: Some(size),
            error_message: Some("File too large for preview (>5MB)".to_string()),
        });
    }

    match client.get_object().bucket(&bucket_name).key(&key).send().await {
        Ok(resp) => {
            let bytes = resp.body.collect().await
                .map_err(|e| format!("Failed to read body: {}", e))?;
            let raw = bytes.into_bytes();

            if is_text_content_type(&content_type) {
                let text = String::from_utf8_lossy(&raw).to_string();
                Ok(S3PreviewResponse {
                    success: true,
                    content: Some(text),
                    content_type: Some(content_type),
                    is_binary: false,
                    size: Some(size),
                    error_message: None,
                })
            } else if content_type.starts_with("image/") {
                use base64::Engine;
                let b64 = base64::engine::general_purpose::STANDARD.encode(&raw);
                let data_url = format!("data:{};base64,{}", content_type, b64);
                Ok(S3PreviewResponse {
                    success: true,
                    content: Some(data_url),
                    content_type: Some(content_type),
                    is_binary: true,
                    size: Some(size),
                    error_message: None,
                })
            } else {
                // Last resort: try to detect if it's valid UTF-8 text
                if raw.is_empty() {
                    // Empty file is always editable as text
                    return Ok(S3PreviewResponse {
                        success: true,
                        content: Some(String::new()),
                        content_type: Some(content_type),
                        is_binary: false,
                        size: Some(size),
                        error_message: None,
                    });
                }

                if raw.len() < 1_000_000 {
                    if let Ok(text) = std::str::from_utf8(&raw) {
                        // Heuristic: if <5% of chars are control chars, treat as text
                        let control_count = text.chars().filter(|c| c.is_control() && *c != '\n' && *c != '\r' && *c != '\t').count();
                        if (control_count as f64 / text.len() as f64) < 0.05 {
                            return Ok(S3PreviewResponse {
                                success: true,
                                content: Some(text.to_string()),
                                content_type: Some(content_type),
                                is_binary: false,
                                size: Some(size),
                                error_message: None,
                            });
                        }
                    }
                }

                Ok(S3PreviewResponse {
                    success: true,
                    content: None,
                    content_type: Some(content_type),
                    is_binary: true,
                    size: Some(size),
                    error_message: None,
                })
            }
        }
        Err(e) => Ok(S3PreviewResponse {
            success: false,
            content: None,
            content_type: None,
            is_binary: false,
            size: None,
            error_message: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
pub async fn copy_s3_object(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    source_bucket: String,
    source_key: String,
    dest_bucket: String,
    dest_key: String,
) -> Result<GenericResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let copy_source = format!("{}/{}", source_bucket, source_key);

    match client.copy_object().bucket(&dest_bucket).key(&dest_key).copy_source(&copy_source).send().await {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse { success: false, error_message: Some(e.to_string()) }),
    }
}

// ============================================================
// Move Object (Copy + Delete)
// ============================================================

#[tauri::command]
pub async fn move_s3_object(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    source_bucket: String,
    source_key: String,
    dest_bucket: String,
    dest_key: String,
) -> Result<GenericResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let copy_source = format!("{}/{}", source_bucket, source_key);

    // Step 1: Copy
    match client.copy_object()
        .bucket(&dest_bucket)
        .key(&dest_key)
        .copy_source(&copy_source)
        .send()
        .await
    {
        Ok(_) => {},
        Err(e) => return Ok(GenericResponse {
            success: false,
            error_message: Some(format!("Copy failed: {}", e)),
        }),
    }

    // Step 2: Delete source
    match client.delete_object()
        .bucket(&source_bucket)
        .key(&source_key)
        .send()
        .await
    {
        Ok(_) => Ok(GenericResponse { success: true, error_message: None }),
        Err(e) => Ok(GenericResponse {
            success: false,
            error_message: Some(format!("Copy succeeded but delete source failed: {}", e)),
        }),
    }
}

// ============================================================
// Multipart Upload (for large files)
// ============================================================

#[derive(Serialize)]
pub struct MultipartUploadResponse {
    pub success: bool,
    pub key: Option<String>,
    pub parts_uploaded: Option<u32>,
    pub error_message: Option<String>,
}

/// Upload a file using multipart upload. Splits files into 10MB chunks.
/// Recommended for files > 100MB but works for any size.
#[tauri::command]
pub async fn multipart_upload_s3_object(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    key: String,
    file_path: String,
) -> Result<MultipartUploadResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let content_type = mime_from_extension(&key).unwrap_or("application/octet-stream");

    // Read file
    let file_bytes = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let file_size = file_bytes.len();
    const PART_SIZE: usize = 10 * 1024 * 1024; // 10MB per part

    // If file is small enough, use simple upload
    if file_size <= PART_SIZE {
        let body = ByteStream::from(file_bytes);
        match client.put_object()
            .bucket(&bucket_name)
            .key(&key)
            .content_type(content_type)
            .body(body)
            .send()
            .await
        {
            Ok(_) => return Ok(MultipartUploadResponse {
                success: true,
                key: Some(key),
                parts_uploaded: Some(1),
                error_message: None,
            }),
            Err(e) => return Ok(MultipartUploadResponse {
                success: false,
                key: None,
                parts_uploaded: None,
                error_message: Some(e.to_string()),
            }),
        }
    }

    // Initiate multipart upload
    let create_resp = client.create_multipart_upload()
        .bucket(&bucket_name)
        .key(&key)
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| format!("Failed to initiate multipart upload: {}", e))?;

    let upload_id = create_resp.upload_id()
        .ok_or_else(|| "No upload_id returned".to_string())?
        .to_string();

    // Upload parts
    let mut completed_parts: Vec<aws_sdk_s3::types::CompletedPart> = Vec::new();
    let mut part_number: i32 = 1;
    let mut offset: usize = 0;

    while offset < file_size {
        let end = std::cmp::min(offset + PART_SIZE, file_size);
        let chunk = file_bytes[offset..end].to_vec();
        let body = ByteStream::from(chunk);

        let upload_part_resp = client.upload_part()
            .bucket(&bucket_name)
            .key(&key)
            .upload_id(&upload_id)
            .part_number(part_number)
            .body(body)
            .send()
            .await;

        match upload_part_resp {
            Ok(resp) => {
                let etag = resp.e_tag().unwrap_or_default().to_string();
                let completed = aws_sdk_s3::types::CompletedPart::builder()
                    .part_number(part_number)
                    .e_tag(&etag)
                    .build();
                completed_parts.push(completed);
            }
            Err(e) => {
                // Abort on failure
                let _ = client.abort_multipart_upload()
                    .bucket(&bucket_name)
                    .key(&key)
                    .upload_id(&upload_id)
                    .send()
                    .await;

                return Ok(MultipartUploadResponse {
                    success: false,
                    key: None,
                    parts_uploaded: Some((part_number - 1) as u32),
                    error_message: Some(format!("Part {} upload failed: {}", part_number, e)),
                });
            }
        }

        offset = end;
        part_number += 1;
    }

    // Complete multipart upload
    let completed_upload = aws_sdk_s3::types::CompletedMultipartUpload::builder()
        .set_parts(Some(completed_parts.clone()))
        .build();

    match client.complete_multipart_upload()
        .bucket(&bucket_name)
        .key(&key)
        .upload_id(&upload_id)
        .multipart_upload(completed_upload)
        .send()
        .await
    {
        Ok(_) => Ok(MultipartUploadResponse {
            success: true,
            key: Some(key),
            parts_uploaded: Some(completed_parts.len() as u32),
            error_message: None,
        }),
        Err(e) => {
            // Try to abort
            let _ = client.abort_multipart_upload()
                .bucket(&bucket_name)
                .key(&key)
                .upload_id(&upload_id)
                .send()
                .await;

            Ok(MultipartUploadResponse {
                success: false,
                key: None,
                parts_uploaded: Some(completed_parts.len() as u32),
                error_message: Some(format!("Complete multipart failed: {}", e)),
            })
        }
    }
}

// ============================================================
// Presigned URL (temporary access)
// ============================================================

#[derive(Serialize)]
pub struct PresignedUrlResponse {
    pub success: bool,
    pub url: Option<String>,
    pub expires_in_secs: Option<u64>,
    pub error_message: Option<String>,
}

/// Generate a presigned URL for temporary access to an S3 object.
/// `expires_in_secs`: how long the URL is valid (default 3600 = 1h, max 604800 = 7 days)
#[tauri::command]
pub async fn generate_presigned_url(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    key: String,
    expires_in_secs: Option<u64>,
) -> Result<PresignedUrlResponse, String> {
    use aws_sdk_s3::presigning::PresigningConfig;
    use std::time::Duration;

    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let expiry = expires_in_secs.unwrap_or(3600).min(604800); // cap at 7 days

    let presigning_config = PresigningConfig::builder()
        .expires_in(Duration::from_secs(expiry))
        .build()
        .map_err(|e| format!("Failed to build presigning config: {}", e))?;

    match client.get_object()
        .bucket(&bucket_name)
        .key(&key)
        .presigned(presigning_config)
        .await
    {
        Ok(presigned_request) => {
            let url = presigned_request.uri().to_string();
            Ok(PresignedUrlResponse {
                success: true,
                url: Some(url),
                expires_in_secs: Some(expiry),
                error_message: None,
            })
        }
        Err(e) => Ok(PresignedUrlResponse {
            success: false,
            url: None,
            expires_in_secs: None,
            error_message: Some(e.to_string()),
        }),
    }
}

// ============================================================
// Put Object Content (direct string upload for editor)
// ============================================================

#[derive(Serialize)]
pub struct PutContentResponse {
    pub success: bool,
    pub key: Option<String>,
    pub error_message: Option<String>,
}

/// Upload text content directly to S3 (for the in-app editor save).
#[tauri::command]
pub async fn put_s3_object_content(
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket_name: String,
    key: String,
    content: String,
) -> Result<PutContentResponse, String> {
    let client = build_s3_client(&access_key_id, &secret_access_key, &region).await;

    let content_type = mime_from_extension(&key).unwrap_or("text/plain");
    let body = ByteStream::from(content.into_bytes());

    match client.put_object()
        .bucket(&bucket_name)
        .key(&key)
        .content_type(content_type)
        .body(body)
        .send()
        .await
    {
        Ok(_) => Ok(PutContentResponse { success: true, key: Some(key), error_message: None }),
        Err(e) => Ok(PutContentResponse { success: false, key: None, error_message: Some(e.to_string()) }),
    }
}
