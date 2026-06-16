use super::error::AppError;
use super::template_models::*;
use handlebars::Handlebars;
use uuid::Uuid;

pub struct TemplateService {
    handlebars: Handlebars<'static>,
}

impl TemplateService {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        // Non-strict: a missing/optional template variable renders as empty
        // rather than failing the whole document. Office documents legitimately
        // have optional fields (no STNK for non-vehicles, blank approval date on
        // a draft, etc.); strict mode turned any such gap into a hard 500 on SK
        // generation. Template authoring errors are caught by the per-template
        // `variables` contract + tests, not by crashing live rendering.
        handlebars.set_strict_mode(false);

        // Register custom helpers
        handlebars.register_helper("format_date", Box::new(format_date_helper));
        handlebars.register_helper("format_currency", Box::new(format_currency_helper));
        handlebars.register_helper("format_number", Box::new(format_number_helper));

        Self { handlebars }
    }

    /// Create a new template
    pub async fn create_template(
        &self,
        pool: &deadpool_postgres::Pool,
        request: CreateTemplateRequest,
        created_by: Uuid,
    ) -> Result<DocumentTemplate, AppError> {
        self.validate_template(&request.content)?;

        let client = pool.get().await?;
        let row = client
            .query_one(
                "INSERT INTO dokumen.document_templates (
                    name, description, template_type, content, format, output_format,
                    variables, sample_data, letterhead_config, created_by
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *",
                &[
                    &request.name,
                    &request.description,
                    &request.template_type,
                    &request.content,
                    &request.format,
                    &request.output_format,
                    &request.variables,
                    &request.sample_data,
                    &request.letterhead_config,
                    &created_by,
                ],
            )
            .await?;

        let template = DocumentTemplate::from(&row);

        // Create initial version
        self.create_version(
            pool,
            template.id,
            &template,
            created_by,
            Some("Initial version"),
        )
        .await?;

        Ok(template)
    }

    /// Get template by ID
    pub async fn get_template(
        &self,
        pool: &deadpool_postgres::Pool,
        template_id: Uuid,
    ) -> Result<DocumentTemplate, AppError> {
        let client = pool.get().await?;
        let row = client
            .query_opt(
                "SELECT * FROM dokumen.document_templates WHERE id = $1",
                &[&template_id],
            )
            .await?
            .ok_or_else(|| AppError::NotFound("Template not found".to_string()))?;

        Ok(DocumentTemplate::from(&row))
    }

    /// List templates with filtering
    pub async fn list_templates(
        &self,
        pool: &deadpool_postgres::Pool,
        query: ListTemplatesQuery,
    ) -> Result<ListTemplatesResponse, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.per_page.unwrap_or(20).min(100);
        let offset = (page - 1) * per_page;

        let client = pool.get().await?;

        // Build parameterized queries
        let (rows, total) = match (&query.template_type, query.is_active) {
            (Some(tt), Some(ia)) => {
                let rows = client
                    .query(
                        "SELECT * FROM dokumen.document_templates WHERE template_type = $1 AND is_active = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                        &[tt, &ia, &per_page, &offset],
                    )
                    .await?;
                let count_row = client
                    .query_one(
                        "SELECT COUNT(*) as count FROM dokumen.document_templates WHERE template_type = $1 AND is_active = $2",
                        &[tt, &ia],
                    )
                    .await?;
                let total: i64 = count_row.get(0);
                (rows, total)
            }
            (Some(tt), None) => {
                let rows = client
                    .query(
                        "SELECT * FROM dokumen.document_templates WHERE template_type = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                        &[tt, &per_page, &offset],
                    )
                    .await?;
                let count_row = client
                    .query_one(
                        "SELECT COUNT(*) as count FROM dokumen.document_templates WHERE template_type = $1",
                        &[tt],
                    )
                    .await?;
                let total: i64 = count_row.get(0);
                (rows, total)
            }
            (None, Some(ia)) => {
                let rows = client
                    .query(
                        "SELECT * FROM dokumen.document_templates WHERE is_active = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                        &[&ia, &per_page, &offset],
                    )
                    .await?;
                let count_row = client
                    .query_one(
                        "SELECT COUNT(*) as count FROM dokumen.document_templates WHERE is_active = $1",
                        &[&ia],
                    )
                    .await?;
                let total: i64 = count_row.get(0);
                (rows, total)
            }
            (None, None) => {
                let rows = client
                    .query(
                        "SELECT * FROM dokumen.document_templates ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                        &[&per_page, &offset],
                    )
                    .await?;
                let count_row = client
                    .query_one(
                        "SELECT COUNT(*) as count FROM dokumen.document_templates",
                        &[],
                    )
                    .await?;
                let total: i64 = count_row.get(0);
                (rows, total)
            }
        };

        let templates: Vec<DocumentTemplate> = rows.iter().map(DocumentTemplate::from).collect();

        Ok(ListTemplatesResponse {
            templates,
            total,
            page,
            per_page,
        })
    }

    /// Update template (creates new version)
    pub async fn update_template(
        &self,
        pool: &deadpool_postgres::Pool,
        template_id: Uuid,
        request: UpdateTemplateRequest,
        updated_by: Uuid,
    ) -> Result<DocumentTemplate, AppError> {
        let template = self.get_template(pool, template_id).await?;

        if let Some(ref content) = request.content {
            self.validate_template(content)?;
        }

        let new_version = template.version + 1;

        let client = pool.get().await?;
        let row = client
            .query_one(
                "UPDATE dokumen.document_templates
                 SET
                    name = COALESCE($1, name),
                    description = COALESCE($2, description),
                    content = COALESCE($3, content),
                    format = COALESCE($4, format),
                    output_format = COALESCE($5, output_format),
                    variables = COALESCE($6, variables),
                    sample_data = COALESCE($7, sample_data),
                    letterhead_config = COALESCE($8, letterhead_config),
                    version = $9,
                    updated_by = $10,
                    updated_at = NOW()
                 WHERE id = $11
                 RETURNING *",
                &[
                    &request.name,
                    &request.description,
                    &request.content,
                    &request.format,
                    &request.output_format,
                    &request.variables,
                    &request.sample_data,
                    &request.letterhead_config,
                    &new_version,
                    &updated_by,
                    &template_id,
                ],
            )
            .await?;

        let updated = DocumentTemplate::from(&row);

        self.create_version(
            pool,
            template_id,
            &updated,
            updated_by,
            request.change_notes.as_deref(),
        )
        .await?;

        Ok(updated)
    }

    /// Delete template (soft delete by setting is_active = false)
    pub async fn delete_template(
        &self,
        pool: &deadpool_postgres::Pool,
        template_id: Uuid,
    ) -> Result<(), AppError> {
        let client = pool.get().await?;
        client
            .execute(
                "UPDATE dokumen.document_templates SET is_active = false WHERE id = $1",
                &[&template_id],
            )
            .await?;

        Ok(())
    }

    /// Get template versions
    pub async fn get_template_versions(
        &self,
        pool: &deadpool_postgres::Pool,
        template_id: Uuid,
    ) -> Result<Vec<TemplateVersion>, AppError> {
        let client = pool.get().await?;
        let rows = client
            .query(
                "SELECT * FROM dokumen.template_versions WHERE template_id = $1 ORDER BY version DESC",
                &[&template_id],
            )
            .await?;

        Ok(rows.iter().map(TemplateVersion::from).collect())
    }

    /// Preview template with sample data
    pub async fn preview_template(
        &self,
        pool: &deadpool_postgres::Pool,
        request: TemplatePreviewRequest,
    ) -> Result<TemplatePreviewResponse, AppError> {
        let template = self.get_template(pool, request.template_id).await?;

        let html = self.render_template(&template.content, &request.data)?;
        let variables_used = self.extract_variables(&template.content);

        Ok(TemplatePreviewResponse {
            html,
            variables_used,
        })
    }

    /// Render template with data
    pub fn render_template(
        &self,
        template_content: &str,
        data: &serde_json::Value,
    ) -> Result<String, AppError> {
        self.handlebars
            .render_template(template_content, data)
            .map_err(|e| AppError::Validation(format!("Template rendering failed: {}", e)))
    }

    /// Validate template syntax
    fn validate_template(&self, content: &str) -> Result<(), AppError> {
        self.handlebars
            .render_template(content, &serde_json::json!({}))
            .map_err(|e| AppError::Validation(format!("Invalid template syntax: {}", e)))?;
        Ok(())
    }

    /// Extract variable names from template
    fn extract_variables(&self, content: &str) -> Vec<String> {
        let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        re.captures_iter(content)
            .map(|cap| cap[1].trim().to_string())
            .collect()
    }

    /// Create version history entry
    async fn create_version(
        &self,
        pool: &deadpool_postgres::Pool,
        template_id: Uuid,
        template: &DocumentTemplate,
        created_by: Uuid,
        change_notes: Option<&str>,
    ) -> Result<(), AppError> {
        let client = pool.get().await?;
        client
            .execute(
                "INSERT INTO dokumen.template_versions (
                    template_id, version, content, variables, created_by, change_notes
                )
                VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &template_id,
                    &template.version,
                    &template.content,
                    &template.variables,
                    &created_by,
                    &change_notes,
                ],
            )
            .await?;

        Ok(())
    }
}

// Handlebars helpers
fn format_date_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let date_str = h.param(0).and_then(|v| v.value().as_str()).ok_or_else(|| {
        handlebars::RenderError::from(handlebars::RenderErrorReason::Other(
            "Date parameter required".to_string(),
        ))
    })?;

    out.write(date_str)?;
    Ok(())
}

fn format_currency_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let amount = h.param(0).and_then(|v| v.value().as_f64()).ok_or_else(|| {
        handlebars::RenderError::from(handlebars::RenderErrorReason::Other(
            "Amount parameter required".to_string(),
        ))
    })?;

    let formatted = format!("Rp {:.2}", amount);
    out.write(&formatted)?;
    Ok(())
}

fn format_number_helper(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
) -> handlebars::HelperResult {
    let number = h.param(0).and_then(|v| v.value().as_i64()).ok_or_else(|| {
        handlebars::RenderError::from(handlebars::RenderErrorReason::Other(
            "Number parameter required".to_string(),
        ))
    })?;

    let formatted = format!("{}", number);
    out.write(&formatted)?;
    Ok(())
}

impl Default for TemplateService {
    fn default() -> Self {
        Self::new()
    }
}
