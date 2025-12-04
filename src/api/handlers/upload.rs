use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Serialize;
use tokio::fs;
use uuid::Uuid;

use crate::{api::AppState, utils::error::AppError};

#[derive(Debug, Serialize)]
pub struct UploadKernelResponse {
    pub url: String,
    pub checksum: String,
    pub file_size: u64,
}

/// 上传内核文件处理器
///
/// POST /api/v1/uploads/kernel
pub async fn upload_kernel(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_bytes: Option<Vec<u8>> = None;
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
            _ => {},
        }
    }

    let file_bytes =
        file_bytes.ok_or_else(|| AppError::BadRequest("Missing file field".to_string()))?;
    let filename = filename.unwrap_or_else(|| "kernel.wasm".to_string());

    // 生成唯一ID
    let id = Uuid::new_v4();
    let upload_dir = PathBuf::from("uploads").join("kernels").join(id.to_string());

    // 创建目录
    fs::create_dir_all(&upload_dir).await.map_err(|e| {
        AppError::InternalWithMessage(format!("Failed to create upload directory: {}", e))
    })?;

    // 保存文件
    let file_path = upload_dir.join(&filename);
    fs::write(&file_path, &file_bytes)
        .await
        .map_err(|e| AppError::InternalWithMessage(format!("Failed to write file: {}", e)))?;

    // 计算校验和 (SHA256)
    let checksum = hex::encode(ring::digest::digest(&ring::digest::SHA256, &file_bytes).as_ref());

    // 构建响应
    let url = format!("/uploads/kernels/{}/{}", id, filename);
    let response = UploadKernelResponse {
        url,
        checksum,
        file_size: file_bytes.len() as u64,
    };

    Ok((StatusCode::CREATED, Json(response)))
}
