use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{api::AppState, utils::error::AppError};

#[derive(Debug, Deserialize)]
pub struct ListKernelsQuery {
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadKernelResponse {
    pub id: String,
    pub version: String,
    pub download_url: String,
}

/// 上传内核
///
/// POST /api/v1/kernels
pub async fn upload_kernel(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut version: Option<String> = None;
    let mut filename: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to process multipart field: {}", e)))?
    {
        match field.name() {
            Some("file") => {
                if let Some(name) = field.file_name() {
                    filename = Some(name.to_string());
                }
                let bytes = field.bytes().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read file bytes: {}", e))
                })?;
                file_bytes = Some(bytes.to_vec());
            },
            Some("version") => {
                version =
                    Some(field.text().await.map_err(|e| {
                        AppError::BadRequest(format!("Failed to read version: {}", e))
                    })?);
            },
            _ => {},
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| AppError::BadRequest("Missing file field".to_string()))?;
    let version =
        version.ok_or_else(|| AppError::BadRequest("Missing version field".to_string()))?;
    let filename = filename.unwrap_or_else(|| "kernel.wasm".to_string());

    let kernel = state.kernel_service.upload_kernel(&version, file_bytes, &filename).await?;

    let response = UploadKernelResponse {
        id: kernel.id,
        version: kernel.version.clone(),
        download_url: format!("/api/v1/kernels/{}/download", kernel.version),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// 列出所有内核
///
/// GET /api/v1/kernels
pub async fn list_kernels(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListKernelsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let kernels = state.kernel_service.list_kernels(query.status.as_deref()).await?;

    Ok(Json(kernels))
}

/// 获取内核详情
///
/// GET /api/v1/kernels/:version
pub async fn get_kernel(
    State(state): State<Arc<AppState>>,
    Path(version): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let kernel = state.kernel_service.get_kernel(&version).await?;

    Ok(Json(kernel))
}

/// 下载内核
///
/// GET /api/v1/kernels/:version/download
pub async fn download_kernel(
    State(state): State<Arc<AppState>>,
    Path(version): Path<String>,
) -> Result<Response<Body>, AppError> {
    let file_bytes = state.kernel_service.download_kernel(&version).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/wasm")
        .header("X-Kernel-Version", version)
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(file_bytes))
        .map_err(|e| AppError::InternalWithMessage(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// 发布内核版本
///
/// POST /api/v1/kernels/:version/publish
pub async fn publish_kernel(
    State(state): State<Arc<AppState>>,
    Path(version): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.kernel_service.publish_kernel(&version).await?;

    Ok(StatusCode::OK)
}

/// 删除内核
///
/// DELETE /api/v1/kernels/:version
pub async fn delete_kernel(
    State(state): State<Arc<AppState>>,
    Path(version): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.kernel_service.delete_kernel(&version).await?;

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Public Kernel Endpoints (for demo/public access)
// ============================================================================

/// 获取最新稳定内核（公开）
///
/// GET /api/v1/public/kernels/latest
pub async fn get_latest_kernel_public(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let kernels = state.kernel_service.list_kernels(Some("stable")).await?;

    if kernels.is_empty() {
        return Err(AppError::NotFound("No stable kernels available".to_string()));
    }

    // Return the most recent stable kernel
    let latest = kernels.into_iter().max_by_key(|k| k.created_at.clone()).unwrap();

    Ok(Json(latest))
}

/// 列出所有稳定内核（公开）
///
/// GET /api/v1/public/kernels
pub async fn list_stable_kernels_public(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let kernels = state.kernel_service.list_kernels(Some("stable")).await?;
    Ok(Json(kernels))
}

/// 下载内核（公开）
///
/// GET /api/v1/public/kernels/:version/download
pub async fn download_kernel_public(
    State(state): State<Arc<AppState>>,
    Path(version): Path<String>,
) -> Result<Response<Body>, AppError> {
    // Only allow downloading stable kernels publicly
    let kernel = state.kernel_service.get_kernel(&version).await?;

    if kernel.status != "stable" {
        return Err(AppError::BadRequest(
            "Only stable kernels can be downloaded publicly".to_string(),
        ));
    }

    let file_bytes = state.kernel_service.download_kernel(&version).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/wasm")
        .header("X-Kernel-Version", version)
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .body(Body::from(file_bytes))
        .map_err(|e| AppError::InternalWithMessage(format!("Failed to build response: {}", e)))?;

    Ok(response)
}
