use anyhow::Result;
use colored::*;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::OnceLock;

pub struct SimpleOutput;

impl SimpleOutput {
    pub fn new() -> Self {
        Self
    }
    
    pub fn success(&self, message: &str) {
        success(message);
    }
    
    pub fn error(&self, message: &str) {
        error(message);
    }
    
    pub fn warning(&self, message: &str) {
        warning(message);
    }
    
    pub fn info(&self, message: &str) {
        info(message);
    }
    
    pub fn debug(&self, message: &str) {
        debug(message);
    }
    
    pub fn header(&self, title: &str) {
        header(title);
    }
    
    pub fn subheader(&self, title: &str) {
        subheader(title);
    }
    
    pub fn step(&self, step: usize, total: usize, message: &str) {
        step(step, total, message);
    }
    
    pub fn create_progress_bar(&self, len: u64, message: &str) -> ProgressBar {
        create_progress_bar(len, message)
    }
    
    pub fn prompt_confirmation(&self, message: &str) -> Result<bool> {
        prompt_confirmation(message)
    }
    
    pub fn prompt_input(&self, message: &str, default: Option<&str>) -> Result<String> {
        prompt_input(message, default)
    }
    
    pub fn prompt_select<T: ToString>(&self, message: &str, items: &[T]) -> Result<usize> {
        prompt_select(message, items)
    }
    
    pub fn table_row(&self, columns: &[&str]) {
        table_row(columns);
    }
    
    pub fn table_header(&self, columns: &[&str]) {
        table_header(columns);
    }
    
    pub fn code_block(&self, code: &str, language: Option<&str>) {
        code_block(code, language);
    }
}

pub type SimplOutput = SimpleOutput;

static OUTPUT_CONFIG: OnceLock<OutputConfig> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub colored: bool,
    pub verbose: bool,
    pub quiet: bool,
}

pub fn init(verbose: bool, color: bool) {
    let config = OutputConfig {
        verbose,
        color,
        term: Term::stdout(),
    };
    OUTPUT_CONFIG.set(config).unwrap();
}

pub fn get_config() -> &'static OutputConfig {
    OUTPUT_CONFIG.get().unwrap_or_else(|| {
        init(false, true);
        OUTPUT_CONFIG.get().unwrap()
    })
}

pub fn success(message: &str) {
    let config = get_config();
    let output = if config.color {
        format!("✅ {}", message.green())
    } else {
        format!("✅ {}", message)
    };
    println!("{}", output);
}

pub fn error(message: &str) {
    let config = get_config();
    let output = if config.color {
        format!("❌ {}", message.red())
    } else {
        format!("❌ {}", message)
    };
    eprintln!("{}", output);
}

pub fn warning(message: &str) {
    let config = get_config();
    let output = if config.color {
        format!("⚠️  {}", message.yellow())
    } else {
        format!("⚠️  {}", message)
    };
    println!("{}", output);
}

pub fn info(message: &str) {
    let config = get_config();
    let output = if config.color {
        format!("ℹ️  {}", message.blue())
    } else {
        format!("ℹ️  {}", message)
    };
    println!("{}", output);
}

pub fn debug(message: &str) {
    let config = get_config();
    if config.verbose {
        let output = if config.color {
            format!("🐛 {}", message.dimmed())
        } else {
            format!("🐛 {}", message)
        };
        println!("{}", output);
    }
}

pub fn header(title: &str) {
    let config = get_config();
    let separator = "=".repeat(60);
    
    if config.color {
        println!("{}", separator.cyan());
        println!("{}", title.cyan().bold());
        println!("{}", separator.cyan());
    } else {
        println!("{}", separator);
        println!("{}", title);
        println!("{}", separator);
    }
}

pub fn subheader(title: &str) {
    let config = get_config();
    let separator = "-".repeat(40);
    
    if config.color {
        println!("{}", separator.blue());
        println!("{}", title.blue().bold());
        println!("{}", separator.blue());
    } else {
        println!("{}", separator);
        println!("{}", title);
        println!("{}", separator);
    }
}

pub fn step(step: usize, total: usize, message: &str) {
    let config = get_config();
    let output = if config.color {
        format!("📋 [{}/{}] {}", step.to_string().bold(), total.to_string().bold(), message)
    } else {
        format!("📋 [{}/{}] {}", step, total, message)
    };
    println!("{}", output);
}

pub fn create_progress_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message(message.to_string());
    pb
}

pub fn prompt_confirmation(message: &str) -> Result<bool> {
    use dialoguer::Confirm;
    
    let confirmation = Confirm::new()
        .with_prompt(message)
        .default(false)
        .interact()?;
    
    Ok(confirmation)
}

pub fn prompt_input(message: &str, default: Option<&str>) -> Result<String> {
    use dialoguer::Input;
    
    if let Some(default) = default {
        let result = Input::new()
            .with_prompt(message)
            .default(default.to_string())
            .interact_text()?;
        Ok(result)
    } else {
        let result = Input::new()
            .with_prompt(message)
            .interact_text()?;
        Ok(result)
    }
}

pub fn prompt_select<T: ToString>(message: &str, items: &[T]) -> Result<usize> {
    use dialoguer::Select;
    
    let selection = Select::new()
        .with_prompt(message)
        .items(items)
        .default(0)
        .interact()?;
    
    Ok(selection)
}

pub fn table_row(columns: &[&str]) {
    let config = get_config();
    let separator = " | ";
    
    if config.color {
        let formatted: Vec<String> = columns.iter().map(|col| col.white().to_string()).collect();
        println!("{}", formatted.join(separator));
    } else {
        println!("{}", columns.join(separator));
    }
}

pub fn table_header(columns: &[&str]) {
    let config = get_config();
    let separator = " | ";
    
    if config.color {
        let formatted: Vec<String> = columns.iter().map(|col| col.cyan().bold().to_string()).collect();
        println!("{}", formatted.join(separator));
        
        // Add separator line
        let line_length = columns.iter().map(|col| col.len()).sum::<usize>() + (columns.len() - 1) * separator.len();
        println!("{}", "-".repeat(line_length).cyan());
    } else {
        println!("{}", columns.join(separator));
        
        // Add separator line
        let line_length = columns.iter().map(|col| col.len()).sum::<usize>() + (columns.len() - 1) * separator.len();
        println!("{}", "-".repeat(line_length));
    }
}

pub fn code_block(code: &str, language: Option<&str>) {
    let config = get_config();
    
    if config.color {
        println!("{}", "```".dimmed());
        if let Some(lang) = language {
            println!("{}", lang.dimmed());
        }
        println!("{}", code.white());
        println!("{}", "```".dimmed());
    } else {
        println!("```");
        if let Some(lang) = language {
            println!("{}", lang);
        }
        println!("{}", code);
        println!("```");
    }
}
