//! Global search hook for all microfrontends
//!
//! Provides search functionality across applications, pages, and documents

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

/// Search result item
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    /// Unique identifier for the search result
    pub id: String,
    /// Display title of the result
    pub title: String,
    /// Description or summary of the result
    pub description: String,
    /// Category classification of the result
    pub category: SearchCategory,
    /// URL to navigate to when selected
    pub url: String,
    /// Icon identifier for visual representation
    pub icon: String,
    /// Optional: Module/app name where this result belongs
    pub module: Option<String>,
}

/// Search result category
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SearchCategory {
    /// Application or module result
    Application,
    /// Page or route result
    Page,
    /// Document or file result
    Document,
    /// User or profile result
    User,
    /// Custom category
    Custom(String),
}

impl SearchCategory {
    /// Get display name in Indonesian
    pub fn display_name(&self) -> String {
        match self {
            Self::Application => "Aplikasi".to_string(),
            Self::Page => "Halaman".to_string(),
            Self::Document => "Dokumen".to_string(),
            Self::User => "Pengguna".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }

    /// Get color classes for styling
    pub fn color_classes(&self) -> &'static str {
        match self {
            Self::Application => "bg-blue-100 dark:bg-blue-900 text-blue-600 dark:text-blue-300",
            Self::Page => "bg-green-100 dark:bg-green-900 text-green-600 dark:text-green-300",
            Self::Document => {
                "bg-yellow-100 dark:bg-yellow-900 text-yellow-600 dark:text-yellow-300"
            }
            Self::User => "bg-purple-100 dark:bg-purple-900 text-purple-600 dark:text-purple-300",
            Self::Custom(_) => "bg-gray-100 dark:bg-gray-900 text-gray-600 dark:text-gray-300",
        }
    }
}

/// Search context for managing search state
#[derive(Clone, Copy)]
pub struct SearchContext {
    /// Current search query
    pub query: RwSignal<String>,
    /// Search results
    pub results: RwSignal<Vec<SearchResult>>,
    /// Loading state
    pub is_loading: RwSignal<bool>,
    /// All searchable data
    search_data: RwSignal<Vec<SearchResult>>,
}

impl SearchContext {
    /// Create new search context
    pub fn new() -> Self {
        Self {
            query: RwSignal::new(String::new()),
            results: RwSignal::new(Vec::new()),
            is_loading: RwSignal::new(false),
            search_data: RwSignal::new(Vec::new()),
        }
    }

    /// Register searchable data
    pub fn register_data(&self, data: Vec<SearchResult>) {
        self.search_data.update(|existing| {
            existing.extend(data);
        });
    }

    /// Perform search
    pub fn search(&self, query: String) {
        if query.trim().is_empty() {
            self.results.set(Vec::new());
            return;
        }

        self.is_loading.set(true);
        self.query.set(query.clone());

        let query_lower = query.to_lowercase();
        let mut filtered_results = Vec::new();

        self.search_data.with(|data| {
            filtered_results = data
                .iter()
                .filter(|item| {
                    item.title.to_lowercase().contains(&query_lower)
                        || item.description.to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect();
        });

        // Sort by relevance
        filtered_results.sort_by(|a, b| {
            let a_exact = a.title.to_lowercase() == query_lower;
            let b_exact = b.title.to_lowercase() == query_lower;

            match (a_exact, b_exact) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.title.cmp(&b.title),
            }
        });

        self.results.set(filtered_results);
        self.is_loading.set(false);
    }

    /// Clear search
    pub fn clear(&self) {
        self.query.set(String::new());
        self.results.set(Vec::new());
    }

    /// Get results count
    pub fn results_count(&self) -> usize {
        self.results.with(|r| r.len())
    }
}

impl Default for SearchContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook to use global search
pub fn use_search() -> SearchContext {
    if let Some(ctx) = use_context::<SearchContext>() {
        return ctx;
    }

    let ctx = SearchContext::new();
    provide_context(ctx);
    ctx
}

/// Debounced search hook
pub fn use_debounced_search(delay_ms: u32) -> impl Fn(String) {
    let search_ctx = use_search();
    let (pending_query, set_pending_query) = signal(String::new());

    Effect::new(move |_| {
        let query = pending_query.get();
        if query.is_empty() {
            search_ctx.clear();
            return;
        }

        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(delay_ms).await;

            if pending_query.get() == query {
                search_ctx.search(query);
            }
        });
    });

    move |query: String| {
        set_pending_query.set(query);
    }
}
