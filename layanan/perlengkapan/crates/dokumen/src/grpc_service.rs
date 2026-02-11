use crate::excel_generator::ExcelGenerator;
use crate::pdf_generator::PdfGenerator;
use crate::template_models::GeneratedDocument;
use crate::storage::StorageService;
use crate::template_service::TemplateService;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// Include generated proto code
pub mod dokumen_proto {
    tonic::include_proto!("dokumen");
}

use dokumen_proto::document_service_server::{DocumentService, DocumentServiceServer};

pub struct DocumentServiceImpl {
    pool: Arc<deadpool_postgres::Pool>,
    template_service: Arc<TemplateService>,
    pdf_generator: Arc<PdfGenerator>,
    excel_generator: Arc<ExcelGenerator>,
    storage_service: Arc<StorageService>,
}

impl DocumentServiceImpl {
    pub fn new(
        pool: Arc<deadpool_postgres::Pool>,
        template_service: Arc<TemplateService>,
        pdf_generator: Arc<PdfGenerator>,
        excel_generator: Arc<ExcelGenerator>,
        storage_service: Arc<StorageService>,
    ) -> Self {
        Self {
            pool,
            template_service,
            pdf_generator,
            excel_generator,
            storage_service,
        }
    }
}

#[tonic::async_trait]
impl DocumentService for DocumentServiceImpl {
    async fn generate_document(
        &self,
        request: Request<dokumen_proto::GenerateDocumentRequest>,
    ) -> Result<Response<dokumen_proto::GenerateDocumentResponse>, Status> {
        let req = request.into_inner();

        let template_id = Uuid::parse_str(&req.template_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid template_id: {}", e)))?;

        let template = self
            .template_service
            .get_template(&self.pool, template_id)
            .await
            .map_err(|e| Status::not_found(format!("Template not found: {}", e)))?;

        let data: serde_json::Value = serde_json::from_str(&req.data_json)
            .map_err(|e| Status::invalid_argument(format!("Invalid data_json: {}", e)))?;

        let output_format = req.output_format.unwrap_or(template.output_format.clone());

        let document_id = Uuid::new_v4();
        let filename = format!("{}_{}.{}", template.name, document_id, output_format);
        let output_path = format!("/tmp/{}", filename);

        let file_bytes = match output_format.as_str() {
            "pdf" => self
                .pdf_generator
                .generate_pdf(&template, &data, &output_path)
                .await
                .map_err(|e| Status::internal(format!("PDF generation failed: {}", e)))?,
            "xlsx" | "excel" => self
                .excel_generator
                .generate_excel(&template, &data, &output_path)
                .await
                .map_err(|e| Status::internal(format!("Excel generation failed: {}", e)))?,
            _ => {
                return Err(Status::invalid_argument(format!(
                    "Unsupported output format: {}",
                    output_format
                )));
            }
        };

        let checksum = self.storage_service.calculate_checksum(&file_bytes);

        let storage_path = format!("documents/{}", filename);
        self.storage_service
            .save_encrypted(&storage_path, &file_bytes)
            .await
            .map_err(|e| Status::internal(format!("Storage failed: {}", e)))?;

        let document_number = format!(
            "DOC/{}/{:04}",
            chrono::Utc::now().format("%Y/%m"),
            rand::random::<u16>()
        );

        let metadata: Option<serde_json::Value> = req
            .metadata_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|e| Status::invalid_argument(format!("Invalid metadata_json: {}", e)))?;

        let generated_by = Uuid::new_v4(); // TODO: Get from auth context
        let size = file_bytes.len() as i64;
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Status::internal(format!("Pool error: {}", e)))?;
        client
            .execute(
                "INSERT INTO dokumen.generated_documents (
                    id, template_id, document_number, filename, storage_path,
                    format, size, checksum, generated_data, generated_by, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[
                    &document_id,
                    &template_id,
                    &document_number,
                    &filename,
                    &storage_path,
                    &output_format,
                    &size,
                    &checksum,
                    &data,
                    &generated_by,
                    &metadata,
                ],
            )
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        Ok(Response::new(dokumen_proto::GenerateDocumentResponse {
            document_id: document_id.to_string(),
            document_number,
            filename,
            download_url: format!("/api/documents/{}/download", document_id),
            checksum,
        }))
    }

    async fn get_document(
        &self,
        request: Request<dokumen_proto::GetDocumentRequest>,
    ) -> Result<Response<dokumen_proto::GetDocumentResponse>, Status> {
        let req = request.into_inner();

        let document_id = Uuid::parse_str(&req.document_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid document_id: {}", e)))?;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Status::internal(format!("Pool error: {}", e)))?;
        let row = client
            .query_opt(
                "SELECT * FROM dokumen.generated_documents WHERE id = $1",
                &[&document_id],
            )
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Document not found"))?;

        let doc = GeneratedDocument::from(&row);

        let content = self
            .storage_service
            .load_decrypted(&doc.storage_path)
            .await
            .map_err(|e| Status::internal(format!("Storage error: {}", e)))?;

        Ok(Response::new(dokumen_proto::GetDocumentResponse {
            document_id: doc.id.to_string(),
            document_number: doc.document_number,
            filename: doc.filename,
            storage_path: doc.storage_path,
            format: doc.format,
            size: doc.size,
            checksum: doc.checksum,
            generated_by: doc.generated_by.to_string(),
            generated_at: doc.generated_at.to_rfc3339(),
            status: doc.status,
            content,
        }))
    }

    async fn list_documents(
        &self,
        request: Request<dokumen_proto::ListDocumentsRequest>,
    ) -> Result<Response<dokumen_proto::ListDocumentsResponse>, Status> {
        let req = request.into_inner();

        let page = req.page.unwrap_or(1).max(1);
        let per_page = req.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Status::internal(format!("Pool error: {}", e)))?;

        // Build parameterized query
        let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        let mut conditions = Vec::new();
        let mut param_idx = 1u32;

        if let Some(ref template_id) = req.template_id {
            let tid = Uuid::parse_str(template_id)
                .map_err(|e| Status::invalid_argument(format!("Invalid template_id: {}", e)))?;
            conditions.push(format!("template_id = ${}", param_idx));
            params.push(Box::new(tid));
            param_idx += 1;
        }

        if let Some(ref status) = req.status {
            conditions.push(format!("status = ${}", param_idx));
            params.push(Box::new(status.clone()));
            param_idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let query = format!(
            "SELECT * FROM dokumen.generated_documents{} ORDER BY generated_at DESC LIMIT ${} OFFSET ${}",
            where_clause, param_idx, param_idx + 1
        );
        params.push(Box::new(per_page));
        params.push(Box::new(offset));

        let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();

        let rows = client
            .query(&query, &param_refs)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        let docs: Vec<_> = rows
            .iter()
            .map(|row| {
                let doc = GeneratedDocument::from(row);
                dokumen_proto::DocumentInfo {
                    document_id: doc.id.to_string(),
                    document_number: doc.document_number,
                    filename: doc.filename,
                    format: doc.format,
                    size: doc.size,
                    generated_by: doc.generated_by.to_string(),
                    generated_at: doc.generated_at.to_rfc3339(),
                    status: doc.status,
                }
            })
            .collect();

        let count_query = format!(
            "SELECT COUNT(*) FROM dokumen.generated_documents{}",
            where_clause
        );
        // Re-build count params (without limit/offset)
        let mut count_params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
        if let Some(ref template_id) = req.template_id {
            let tid = Uuid::parse_str(template_id).unwrap();
            count_params.push(Box::new(tid));
        }
        if let Some(ref status) = req.status {
            count_params.push(Box::new(status.clone()));
        }
        let count_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
            count_params.iter().map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync)).collect();

        let count_row = client
            .query_one(&count_query, &count_refs)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        let total: i64 = count_row.get(0);

        Ok(Response::new(dokumen_proto::ListDocumentsResponse {
            documents: docs,
            total,
            page,
            per_page,
        }))
    }

    async fn create_template(
        &self,
        request: Request<dokumen_proto::CreateTemplateRequest>,
    ) -> Result<Response<dokumen_proto::CreateTemplateResponse>, Status> {
        let req = request.into_inner();

        let variables: serde_json::Value = serde_json::from_str(&req.variables_json)
            .map_err(|e| Status::invalid_argument(format!("Invalid variables_json: {}", e)))?;

        let sample_data = req
            .sample_data_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|e| Status::invalid_argument(format!("Invalid sample_data_json: {}", e)))?;

        let letterhead_config = req
            .letterhead_config_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|e| {
                Status::invalid_argument(format!("Invalid letterhead_config_json: {}", e))
            })?;

        let create_req = crate::template_models::CreateTemplateRequest {
            name: req.name,
            description: req.description,
            template_type: req.template_type,
            content: req.content,
            format: req.format,
            output_format: req.output_format,
            variables,
            sample_data,
            letterhead_config,
        };

        let created_by = Uuid::new_v4(); // TODO: Get from auth context
        let template = self
            .template_service
            .create_template(&self.pool, create_req, created_by)
            .await
            .map_err(|e| Status::internal(format!("Template creation failed: {}", e)))?;

        Ok(Response::new(dokumen_proto::CreateTemplateResponse {
            template_id: template.id.to_string(),
            name: template.name,
            version: template.version,
        }))
    }

    async fn get_template(
        &self,
        request: Request<dokumen_proto::GetTemplateRequest>,
    ) -> Result<Response<dokumen_proto::GetTemplateResponse>, Status> {
        let req = request.into_inner();

        let template_id = Uuid::parse_str(&req.template_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid template_id: {}", e)))?;

        let template = self
            .template_service
            .get_template(&self.pool, template_id)
            .await
            .map_err(|e| Status::not_found(format!("Template not found: {}", e)))?;

        Ok(Response::new(dokumen_proto::GetTemplateResponse {
            template_id: template.id.to_string(),
            name: template.name,
            description: template.description,
            template_type: template.template_type,
            content: template.content,
            format: template.format,
            output_format: template.output_format,
            version: template.version,
            is_active: template.is_active,
            variables_json: template.variables.to_string(),
            sample_data_json: template.sample_data.map(|v| v.to_string()),
            letterhead_config_json: template.letterhead_config.map(|v| v.to_string()),
            created_at: template.created_at.to_rfc3339(),
            updated_at: template.updated_at.to_rfc3339(),
        }))
    }

    async fn update_template(
        &self,
        request: Request<dokumen_proto::UpdateTemplateRequest>,
    ) -> Result<Response<dokumen_proto::UpdateTemplateResponse>, Status> {
        let req = request.into_inner();

        let template_id = Uuid::parse_str(&req.template_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid template_id: {}", e)))?;

        let variables = req
            .variables_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|e| Status::invalid_argument(format!("Invalid variables_json: {}", e)))?;

        let sample_data = req
            .sample_data_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|e| Status::invalid_argument(format!("Invalid sample_data_json: {}", e)))?;

        let letterhead_config = req
            .letterhead_config_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|e| {
                Status::invalid_argument(format!("Invalid letterhead_config_json: {}", e))
            })?;

        let update_req = crate::template_models::UpdateTemplateRequest {
            name: req.name,
            description: req.description,
            content: req.content,
            format: req.format,
            output_format: req.output_format,
            variables,
            sample_data,
            letterhead_config,
            change_notes: req.change_notes,
        };

        let updated_by = Uuid::new_v4(); // TODO: Get from auth context
        let template = self
            .template_service
            .update_template(&self.pool, template_id, update_req, updated_by)
            .await
            .map_err(|e| Status::internal(format!("Template update failed: {}", e)))?;

        Ok(Response::new(dokumen_proto::UpdateTemplateResponse {
            template_id: template.id.to_string(),
            new_version: template.version,
        }))
    }

    async fn delete_template(
        &self,
        request: Request<dokumen_proto::DeleteTemplateRequest>,
    ) -> Result<Response<dokumen_proto::DeleteTemplateResponse>, Status> {
        let req = request.into_inner();

        let template_id = Uuid::parse_str(&req.template_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid template_id: {}", e)))?;

        self.template_service
            .delete_template(&self.pool, template_id)
            .await
            .map_err(|e| Status::internal(format!("Template deletion failed: {}", e)))?;

        Ok(Response::new(dokumen_proto::DeleteTemplateResponse {
            success: true,
        }))
    }

    async fn list_templates(
        &self,
        request: Request<dokumen_proto::ListTemplatesRequest>,
    ) -> Result<Response<dokumen_proto::ListTemplatesResponse>, Status> {
        let req = request.into_inner();

        let query = crate::template_models::ListTemplatesQuery {
            template_type: req.template_type,
            is_active: req.is_active,
            page: req.page,
            per_page: req.per_page,
        };

        let result = self
            .template_service
            .list_templates(&self.pool, query)
            .await
            .map_err(|e| Status::internal(format!("Template listing failed: {}", e)))?;

        let templates = result
            .templates
            .into_iter()
            .map(|t| dokumen_proto::TemplateInfo {
                template_id: t.id.to_string(),
                name: t.name,
                description: t.description,
                template_type: t.template_type,
                output_format: t.output_format,
                version: t.version,
                is_active: t.is_active,
                created_at: t.created_at.to_rfc3339(),
            })
            .collect();

        Ok(Response::new(dokumen_proto::ListTemplatesResponse {
            templates,
            total: result.total,
            page: result.page,
            per_page: result.per_page,
        }))
    }

    async fn preview_template(
        &self,
        request: Request<dokumen_proto::PreviewTemplateRequest>,
    ) -> Result<Response<dokumen_proto::PreviewTemplateResponse>, Status> {
        let req = request.into_inner();

        let template_id = Uuid::parse_str(&req.template_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid template_id: {}", e)))?;

        let data: serde_json::Value = serde_json::from_str(&req.data_json)
            .map_err(|e| Status::invalid_argument(format!("Invalid data_json: {}", e)))?;

        let preview_req = crate::template_models::TemplatePreviewRequest {
            template_id,
            data,
        };

        let result = self
            .template_service
            .preview_template(&self.pool, preview_req)
            .await
            .map_err(|e| Status::internal(format!("Template preview failed: {}", e)))?;

        Ok(Response::new(dokumen_proto::PreviewTemplateResponse {
            html: result.html,
            variables_used: result.variables_used,
        }))
    }
}

pub fn create_grpc_server(
    pool: Arc<deadpool_postgres::Pool>,
    template_service: Arc<TemplateService>,
    pdf_generator: Arc<PdfGenerator>,
    excel_generator: Arc<ExcelGenerator>,
    storage_service: Arc<StorageService>,
) -> DocumentServiceServer<DocumentServiceImpl> {
    let service = DocumentServiceImpl::new(
        pool,
        template_service,
        pdf_generator,
        excel_generator,
        storage_service,
    );
    DocumentServiceServer::new(service)
}
