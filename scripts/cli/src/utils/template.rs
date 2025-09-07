use handlebars::Handlebars;
use serde_json::Value;
use anyhow::Result;
use std::collections::HashMap;

pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            handlebars: Handlebars::new(),
        }
    }

    pub fn register_template(&mut self, name: &str, template: &str) -> Result<()> {
        self.handlebars.register_template_string(name, template)?;
        Ok(())
    }

    pub fn render(&self, template_name: &str, data: &HashMap<String, Value>) -> Result<String> {
        Ok(self.handlebars.render(template_name, data)?)
    }

    pub fn render_string(&self, template: &str, data: &HashMap<String, Value>) -> Result<String> {
        Ok(self.handlebars.render_template(template, data)?)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}
