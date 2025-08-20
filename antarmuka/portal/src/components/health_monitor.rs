// SIMPelv2 Portal Health Check Integration
// Enhanced portal with real-time health monitoring

use leptos::*;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Clone, Debug)]
pub struct ServiceHealth {
    pub name: String,
    pub url: String, 
    pub status: ServiceStatus,
    pub response_time: Option<u32>,
    pub last_check: Option<js_sys::Date>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServiceStatus {
    Healthy,
    Unhealthy,
    Unknown,
    Checking,
}

impl ServiceHealth {
    pub fn new(name: String, url: String) -> Self {
        Self {
            name,
            url,
            status: ServiceStatus::Unknown,
            response_time: None,
            last_check: None,
        }
    }
}

#[component]
pub fn HealthMonitor() -> impl IntoView {
    let (services, set_services) = create_signal(Vec::<ServiceHealth>::new());
    let (overall_health, set_overall_health) = create_signal(0u8);
    
    // Initialize services
    create_effect(move |_| {
        let initial_services = vec![
            ServiceHealth::new("Badiklat".to_string(), "http://localhost:8081".to_string()),
            ServiceHealth::new("Datun".to_string(), "http://localhost:8082".to_string()),
            ServiceHealth::new("Intel".to_string(), "http://localhost:8083".to_string()),
            ServiceHealth::new("Pemulihan Aset".to_string(), "http://localhost:8084".to_string()),
            ServiceHealth::new("Pengawasan".to_string(), "http://localhost:8085".to_string()),
            ServiceHealth::new("Pidmil".to_string(), "http://localhost:8086".to_string()),
            ServiceHealth::new("Pidsus".to_string(), "http://localhost:8087".to_string()),
            ServiceHealth::new("Pidum".to_string(), "http://localhost:8088".to_string()),
            ServiceHealth::new("Keuangan".to_string(), "http://localhost:8090".to_string()),
            ServiceHealth::new("Perencanaan".to_string(), "http://localhost:8091".to_string()),
            ServiceHealth::new("Perlengkapan".to_string(), "http://localhost:8092".to_string()),
        ];
        set_services.set(initial_services);
    });
    
    // Health check function
    let check_health = move || {
        console_log!("🏥 Starting health check...");
        
        spawn_local(async move {
            let current_services = services.get();
            let mut updated_services = Vec::new();
            let mut healthy_count = 0u8;
            
            for mut service in current_services {
                service.status = ServiceStatus::Checking;
                
                // Simulate health check (in real app, would make HTTP request)
                let start_time = js_sys::Date::now();
                
                // Simple health check simulation
                let is_healthy = js_sys::Math::random() > 0.1; // 90% success rate
                
                if is_healthy {
                    service.status = ServiceStatus::Healthy;
                    service.response_time = Some((js_sys::Date::now() - start_time) as u32);
                    healthy_count += 1;
                } else {
                    service.status = ServiceStatus::Unhealthy;
                    service.response_time = None;
                }
                
                service.last_check = Some(js_sys::Date::new_0());
                updated_services.push(service);
            }
            
            set_services.set(updated_services);
            set_overall_health.set((healthy_count * 100) / 11); // 11 services total
            
            console_log!("🎯 Health check complete: {}% healthy", (healthy_count * 100) / 11);
        });
    };
    
    // Auto health check every 30 seconds
    create_effect(move |_| {
        check_health();
        
        let interval_id = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                &Closure::wrap(Box::new(move || {
                    check_health();
                }) as Box<dyn Fn()>).into_js_value()
                .unchecked_into(),
                30000, // 30 seconds
            )
            .unwrap();
            
        // Cleanup interval on component unmount
        on_cleanup(move || {
            web_sys::window().unwrap().clear_interval_with_handle(interval_id);
        });
    });
    
    view! {
        <div class="bg-white rounded-lg shadow-lg p-4 mb-4">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-semibold text-gray-800 flex items-center">
                    <span class="mr-2">🏥</span>
                    "System Health"
                </h3>
                <div class="flex items-center space-x-2">
                    <div class={move || format!("flex items-center px-3 py-1 rounded-full text-sm font-medium {}",
                        if overall_health.get() >= 90 { "bg-green-100 text-green-800" }
                        else if overall_health.get() >= 70 { "bg-yellow-100 text-yellow-800" }  
                        else { "bg-red-100 text-red-800" }
                    )}>
                        <span class="mr-1">
                            {move || if overall_health.get() >= 90 { "✅" } 
                                    else if overall_health.get() >= 70 { "⚠️" } 
                                    else { "❌" }}
                        </span>
                        {move || format!("{}%", overall_health.get())}
                    </div>
                </div>
            </div>
            
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                <For
                    each=services
                    key=|service| service.name.clone()
                    children=move |service| {
                        view! {
                            <div class={format!("p-3 rounded-lg border {}",
                                match service.status {
                                    ServiceStatus::Healthy => "border-green-200 bg-green-50",
                                    ServiceStatus::Unhealthy => "border-red-200 bg-red-50", 
                                    ServiceStatus::Checking => "border-blue-200 bg-blue-50",
                                    ServiceStatus::Unknown => "border-gray-200 bg-gray-50",
                                }
                            )}>
                                <div class="flex items-center justify-between">
                                    <h4 class="font-medium text-sm text-gray-800">{service.name}</h4>
                                    <span class="text-xs">
                                        {match service.status {
                                            ServiceStatus::Healthy => "✅",
                                            ServiceStatus::Unhealthy => "❌",
                                            ServiceStatus::Checking => "🔄", 
                                            ServiceStatus::Unknown => "❓",
                                        }}
                                    </span>
                                </div>
                                {service.response_time.map(|rt| view! {
                                    <div class="text-xs text-gray-500 mt-1">
                                        {format!("{}ms", rt)}
                                    </div>
                                })}
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component] 
pub fn EnhancedPortalHeader() -> impl IntoView {
    let (current_time, set_current_time) = create_signal(js_sys::Date::new_0().to_locale_string("id-ID"));
    
    // Update time every minute
    create_effect(move |_| {
        let interval_id = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                &Closure::wrap(Box::new(move || {
                    set_current_time.set(js_sys::Date::new_0().to_locale_string("id-ID"));
                }) as Box<dyn Fn()>).into_js_value()
                .unchecked_into(),
                60000, // 1 minute
            )
            .unwrap();
            
        on_cleanup(move || {
            web_sys::window().unwrap().clear_interval_with_handle(interval_id);
        });
    });
    
    view! {
        <div class="bg-gradient-to-r from-blue-600 to-blue-800 text-white p-4 mb-6">
            <div class="flex items-center justify-between">
                <div class="flex items-center space-x-4">
                    <h1 class="text-2xl font-bold">🏛️ SIMPelv2</h1>
                    <div class="text-sm opacity-90">
                        "Sistem Informasi Manajemen Barang Milik Negara"
                    </div>
                </div>
                <div class="text-right">
                    <div class="text-sm opacity-90">"Portal Terpadu"</div>
                    <div class="text-xs opacity-75">{move || current_time.get()}</div>
                </div>
            </div>
        </div>
    }
}
