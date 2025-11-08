use crate::app::{dto::*, pricing};
use crate::slicer;
use crate::utils;
use crate::AppState;
use axum::{extract::State, http::StatusCode, Json};
use tracing::{error, info};
use uuid::Uuid;

pub async fn quote(
    State(state): State<AppState>,
    Json(req): Json<QuoteRequest>,
) -> Result<Json<QuoteResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate request
    if let Err(e) = req.validate() {
        return Err(error_response(StatusCode::BAD_REQUEST, &e));
    }

    info!(
        "Processing quote request for material={}, infill={}, layer_thickness={}um",
        req.material, req.infill, req.layer_thickness
    );

    // Download STL file from presigned URL
    let stl_path = match utils::download::download_stl(&req.file_url, &state.config.temp_dir).await
    {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to download STL: {}", e);
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to download file: {}", e),
            ));
        }
    };

    // Slice model with Orca Slicer
    let metrics = match slicer::slice_model(
        &stl_path,
        &req.material,
        req.infill,
        req.layer_height_mm(),
        &state.config,
    )
    .await
    {
        Ok(m) => m,
        Err(e) => {
            error!("Slicing failed: {}", e);
            // Cleanup temp file
            let _ = tokio::fs::remove_file(&stl_path).await;
            return Err(error_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("Model slicing failed: {}", e),
            ));
        }
    };

    // Cleanup temp file
    if let Err(e) = tokio::fs::remove_file(&stl_path).await {
        error!("Failed to cleanup temp file: {}", e);
    }

    info!(
        "Slicing successful: print_time={}h, weight={}g",
        metrics.print_time_hours, metrics.filament_weight_g
    );

    // Calculate pricing
    let price = match pricing::calculate_price(&metrics, &req.material, &state.config) {
        Ok(p) => p,
        Err(e) => {
            error!("Pricing calculation failed: {}", e);
            return Err(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Pricing calculation failed: {}", e),
            ));
        }
    };

    let quote_id = Uuid::new_v4();
    let response = QuoteResponse {
        quote_id,
        total_usd: price.total_usd,
        material_cost_usd: price.material_cost_usd,
        machine_cost_usd: price.machine_cost_usd,
        base_fee_usd: price.base_fee_usd,
        lead_time_days: price.lead_time_days,
        print_time_hours: metrics.print_time_hours,
        filament_weight_g: metrics.filament_weight_g,
        volume_cm3: metrics.volume_cm3,
    };

    info!("Quote generated: id={}, total=${}", quote_id, price.total_usd);

    Ok(Json(response))
}

fn error_response(status: StatusCode, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: status.to_string(),
            message: message.to_string(),
        }),
    )
}
