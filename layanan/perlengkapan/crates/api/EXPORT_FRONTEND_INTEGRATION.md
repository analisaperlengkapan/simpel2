# Export Frontend Integration Guide

## Overview

This document describes how to integrate the export functionality into the Leptos frontend for the Perlengkapan service.

## Backend API Endpoints

### 1. Export to Excel (Synchronous/Asynchronous)

**Endpoint:** `GET /export/excel`

**Query Parameters:**

- `entity_type` (required): Type of entity to export
  - `kebutuhan_bmn` - BMN requirements
  - `pakaian_dinas` - Uniform requirements
  - `roadmap_sarpras` - Infrastructure roadmap
  - `riwayat_pemenuhan` - Fulfillment history
- `limit` (optional): Maximum rows to export (default: 50000, max: 50000)
- `tahun_anggaran` (optional): Filter by fiscal year
- `satker_id` (optional): Filter by organizational unit
- `status` (optional): Filter by status

**Response:**

For small datasets (<1000 rows) - **Synchronous**:

- Status: 200 OK
- Content-Type: `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`
- Body: Excel file binary data

For large datasets (>=1000 rows) - **Asynchronous**:

- Status: 202 Accepted
- Content-Type: `application/json`
- Body:

```json
{
  "data": {
    "job_id": "uuid",
    "status": "queued",
    "message": "Export job queued. You will be notified when ready."
  },
  "message": "Export job queued successfully"
}
```

### 2. Get Export Job Status

**Endpoint:** `GET /export/jobs/{job_id}/status`

**Response:**

```json
{
  "data": {
    "job_id": "uuid",
    "status": "queued|processing|completed|failed",
    "progress": 75.5,
    "document_id": "uuid",
    "error_message": "Error details if failed",
    "created_at": "2026-02-10T10:00:00Z",
    "completed_at": "2026-02-10T10:05:00Z"
  },
  "message": "Export job status retrieved successfully"
}
```

### 3. Download Completed Export

**Endpoint:** `GET /export/jobs/{job_id}/download`

**Response:**

- Status: 200 OK
- Content-Type: `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`
- Content-Disposition: `attachment; filename="export_kebutuhan_bmn_{job_id}.xlsx"`
- Body: Excel file binary data

## Frontend Implementation

### Component Structure

```rust
// antarmuka/perlengkapan/src/components/export_button.rs

use leptos::prelude::*;
use gloo_net::http::Request;

#[component]
pub fn ExportButton(
    entity_type: String,
    filters: Option<ExportFilters>,
) -> impl IntoView {
    let (is_exporting, set_is_exporting) = signal(false);
    let (export_job_id, set_export_job_id) = signal::<Option<String>>(None);
    let (export_progress, set_export_progress) = signal::<Option<f32>>(None);
    let (export_error, set_export_error) = signal::<Option<String>>(None);

    let handle_export = move |_| {
        set_is_exporting.set(true);
        set_export_error.set(None);

        spawn_local(async move {
            // Build query parameters
            let mut query_params = vec![
                format!("entity_type={}", entity_type),
            ];

            if let Some(ref f) = filters {
                if let Some(tahun) = f.tahun_anggaran {
                    query_params.push(format!("tahun_anggaran={}", tahun));
                }
                if let Some(ref satker_id) = f.satker_id {
                    query_params.push(format!("satker_id={}", satker_id));
                }
                if let Some(ref status) = f.status {
                    query_params.push(format!("status={}", status));
                }
            }

            let url = format!("/api/export/excel?{}", query_params.join("&"));

            match Request::get(&url)
                .send()
                .await
            {
                Ok(response) => {
                    if response.status() == 200 {
                        // Synchronous export - download immediately
                        if let Ok(blob) = response.binary().await {
                            download_file(&blob, &format!("export_{}.xlsx", entity_type));
                        }
                        set_is_exporting.set(false);
                    } else if response.status() == 202 {
                        // Asynchronous export - poll for status
                        if let Ok(json) = response.json::<ExportJobResponse>().await {
                            set_export_job_id.set(Some(json.data.job_id.clone()));
                            poll_export_status(
                                json.data.job_id,
                                set_export_progress,
                                set_is_exporting,
                                set_export_error,
                            );
                        }
                    } else {
                        set_export_error.set(Some("Export failed".to_string()));
                        set_is_exporting.set(false);
                    }
                }
                Err(e) => {
                    set_export_error.set(Some(format!("Export error: {}", e)));
                    set_is_exporting.set(false);
                }
            }
        });
    };

    view! {
        <div class="export-button-container">
            <button
                class="btn btn-primary"
                on:click=handle_export
                disabled=move || is_exporting.get()
            >
                {move || if is_exporting.get() {
                    "Exporting..."
                } else {
                    "Export to Excel"
                }}
            </button>

            {move || export_progress.get().map(|progress| view! {
                <div class="progress-bar">
                    <div class="progress-fill" style=format!("width: {}%", progress)></div>
                    <span class="progress-text">{format!("{:.0}%", progress)}</span>
                </div>
            })}

            {move || export_error.get().map(|error| view! {
                <div class="alert alert-error">
                    {error}
                </div>
            })}
        </div>
    }
}

// Helper function to poll export status
async fn poll_export_status(
    job_id: String,
    set_progress: WriteSignal<Option<f32>>,
    set_is_exporting: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        match Request::get(&format!("/api/export/jobs/{}/status", job_id))
            .send()
            .await
        {
            Ok(response) => {
                if let Ok(json) = response.json::<ExportJobStatusResponse>().await {
                    match json.data.status.as_str() {
                        "completed" => {
                            // Download the file
                            download_export_job(&job_id).await;
                            set_is_exporting.set(false);
                            break;
                        }
                        "failed" => {
                            set_error.set(Some(
                                json.data.error_message.unwrap_or("Export failed".to_string())
                            ));
                            set_is_exporting.set(false);
                            break;
                        }
                        "processing" => {
                            if let Some(progress) = json.data.progress {
                                set_progress.set(Some(progress));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                set_error.set(Some(format!("Status check error: {}", e)));
                set_is_exporting.set(false);
                break;
            }
        }
    }
}

// Helper function to download export job
async fn download_export_job(job_id: &str) {
    if let Ok(response) = Request::get(&format!("/api/export/jobs/{}/download", job_id))
        .send()
        .await
    {
        if let Ok(blob) = response.binary().await {
            download_file(&blob, &format!("export_{}.xlsx", job_id));
        }
    }
}

// Helper function to trigger browser download
fn download_file(data: &[u8], filename: &str) {
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Create blob
    let array = js_sys::Uint8Array::from(data);
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array);

    let mut blob_props = BlobPropertyBag::new();
    blob_props.type_("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");

    let blob = Blob::new_with_u8_array_sequence_and_options(&blob_parts, &blob_props).unwrap();

    // Create download link
    let url = Url::create_object_url_with_blob(&blob).unwrap();
    let anchor = document
        .create_element("a")
        .unwrap()
        .dyn_into::<HtmlAnchorElement>()
        .unwrap();

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();

    // Cleanup
    Url::revoke_object_url(&url).unwrap();
}

#[derive(Clone)]
pub struct ExportFilters {
    pub tahun_anggaran: Option<i32>,
    pub satker_id: Option<String>,
    pub status: Option<String>,
}

#[derive(serde::Deserialize)]
struct ExportJobResponse {
    data: ExportJobData,
}

#[derive(serde::Deserialize)]
struct ExportJobData {
    job_id: String,
    status: String,
    message: String,
}

#[derive(serde::Deserialize)]
struct ExportJobStatusResponse {
    data: ExportJobStatusData,
}

#[derive(serde::Deserialize)]
struct ExportJobStatusData {
    job_id: String,
    status: String,
    progress: Option<f32>,
    document_id: Option<String>,
    error_message: Option<String>,
    created_at: String,
    completed_at: Option<String>,
}
```

### Usage in Data Tables

```rust
// In your data table component
use crate::components::export_button::ExportButton;

#[component]
pub fn KebutuhanBmnTable() -> impl IntoView {
    // ... table implementation ...

    view! {
        <div class="table-container">
            <div class="table-header">
                <h2>"Kebutuhan BMN"</h2>
                <ExportButton
                    entity_type="kebutuhan_bmn".to_string()
                    filters=Some(ExportFilters {
                        tahun_anggaran: Some(2026),
                        satker_id: None,
                        status: None,
                    })
                />
            </div>
            // ... table content ...
        </div>
    }
}
```

## Notification Integration

For asynchronous exports, integrate with the notification service to notify users when exports are complete:

```rust
// Backend: After export job completes
notification_service.send_notification(SendNotificationRequest {
    user_id: user_id.to_string(),
    notification_type: "export_complete",
    title: "Export Ready".to_string(),
    message: format!("Your {} export is ready for download", entity_type),
    data: serde_json::json!({
        "job_id": job_id,
        "entity_type": entity_type,
    }),
    priority: "normal",
}).await?;
```

## Styling

Add CSS for the export button and progress indicator:

```css
.export-button-container {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.progress-bar {
    position: relative;
    width: 200px;
    height: 24px;
    background-color: #e5e7eb;
    border-radius: 4px;
    overflow: hidden;
}

.progress-fill {
    height: 100%;
    background-color: #3b82f6;
    transition: width 0.3s ease;
}

.progress-text {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    font-size: 12px;
    font-weight: 600;
    color: #1f2937;
}

.alert-error {
    padding: 0.75rem;
    background-color: #fee2e2;
    border: 1px solid #fecaca;
    border-radius: 4px;
    color: #991b1b;
}
```

## Testing

### Manual Testing

1. **Small Dataset Export (Synchronous)**

   ```bash
   curl -X GET "http://localhost:8093/export/excel?entity_type=kebutuhan_bmn&limit=100" \
     -H "Authorization: Bearer YOUR_TOKEN" \
     -o export_small.xlsx
   ```

2. **Large Dataset Export (Asynchronous)**

   ```bash
   # Start export
   curl -X GET "http://localhost:8093/export/excel?entity_type=kebutuhan_bmn&limit=5000" \
     -H "Authorization: Bearer YOUR_TOKEN"

   # Check status
   curl -X GET "http://localhost:8093/export/jobs/{JOB_ID}/status" \
     -H "Authorization: Bearer YOUR_TOKEN"

   # Download when complete
   curl -X GET "http://localhost:8093/export/jobs/{JOB_ID}/download" \
     -H "Authorization: Bearer YOUR_TOKEN" \
     -o export_large.xlsx
   ```

### Integration Testing

Create integration tests in the frontend to verify:

1. Export button renders correctly
2. Synchronous export triggers download
3. Asynchronous export shows progress
4. Error handling displays appropriate messages
5. Download link works correctly

## Requirements Satisfied

- ✅ REQ-K014: System SHALL support export to Excel
- ✅ Export button added to data tables
- ✅ Export progress indicator implemented
- ✅ Notification on export completion
- ✅ Download link created

## Future Enhancements

1. **Export Filters UI**: Add a filter panel to allow users to customize export criteria
2. **Export History**: Show list of recent exports with download links
3. **Scheduled Exports**: Allow users to schedule recurring exports
4. **Export Templates**: Save export configurations for reuse
5. **Multi-format Export**: Add PDF and CSV export options
