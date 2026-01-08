# 🌐 Layanan Portal - SIMPelv2

**Layanan Portal** adalah microservice backend dalam SIMPelv2 yang menyediakan API untuk portal microfrontend, termasuk dashboard dan visualisasi data lintas layanan. Dibangun dengan **Rust** untuk performa dan skalabilitas maksimal.

## 🎯 **Overview**

Layanan Portal menyediakan backend untuk portal microfrontend SIMPelv2:

- **Performance Summary**: Ringkasan performa instansi
- **Data Visualization**: Visualisasi data lintas layanan
- **Real-time Analytics**: Analytics real-time
- **Custom Dashboards**: Dashboard yang dapat dikustomisasi
- **Microfrontend Integration**: Integrasi dengan microfrontend
- **Export Capabilities**: Export data dan laporan

## 🏗️ **Architecture**

### **📊 Dashboard Stack**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   API Gateway   │    │  Dashboard Svc  │
│   (React)       │───▶│   (Envoy)       │───▶│   (Rust)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                                                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   All Services  │    │   PostgreSQL    │    │   Cache Layer   │
│   (Data)        │◀───│   (Analytics)   │───▶│   (Redis)       │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### **🦀 Technology Stack**
- **Language**: Rust (Axum web framework)
- **Data Aggregation**: Multi-service data collection
- **Caching**: Redis untuk performance
- **Database**: PostgreSQL dengan SQLx
- **Real-time**: WebSocket support
- **Monitoring**: Tracing + OpenTelemetry

## 🚀 **Quick Start**

### **Prerequisites**
- Rust 1.75+
- PostgreSQL 15+
- Redis (untuk caching)
- Docker & Docker Compose

### **Installation**

```bash
# Clone repository
cd layanan/dasbor

# Install dependencies
cargo build --release

# Setup environment
cp .env.example .env
# Edit .env with your configuration

# Run database migrations
psql -d simpelv2 -f db/schema.sql

# Start service
cargo run
```

### **Docker Deployment**

```bash
# Build image
docker build -t simpelv2-dasbor .

# Run container
docker run -d \
  --name dasbor \
  -p 3007:3007 \
  -e DATABASE_URL=postgres://user:pass@host:5432/db \
  -e REDIS_URL=redis://redis:6379 \
  simpelv2-dasbor
```

## 📡 **API Endpoints**

### **📊 Dashboard Data**
```http
GET  /dashboard/overview
GET  /dashboard/performance
GET  /dashboard/analytics
GET  /dashboard/metrics
```

### **📈 Charts & Visualizations**
```http
GET  /charts/performance
GET  /charts/trends
GET  /charts/comparison
POST /charts/custom
```

### **🔍 Data Queries**
```http
POST /data/query
GET  /data/export
GET  /data/summary
POST /data/aggregate
```

### **⚙️ Dashboard Configuration**
```http
GET  /config/widgets
POST /config/widgets
PUT  /config/widgets/{id}
DELETE /config/widgets/{id}

GET  /config/layouts
POST /config/layouts
PUT  /config/layouts/{id}
```

### **📱 Real-time Updates**
```http
GET  /realtime/stream
POST /realtime/subscribe
DELETE /realtime/unsubscribe
```

## 🔧 **Configuration**

### **Environment Variables**
```env
# Database
DATABASE_URL=postgres://user:pass@host:5432/simpelv2

# Redis
REDIS_URL=redis://localhost:6379

# Service URLs
SECURITY_SERVICE_URL=http://localhost:3001
AI_SERVICE_URL=http://localhost:3002
DOCUMENT_SERVICE_URL=http://localhost:3003
NOTIFICATION_SERVICE_URL=http://localhost:3004
CONFIGURATION_SERVICE_URL=http://localhost:3005
HELP_SERVICE_URL=http://localhost:3006

# Server
SERVER_PORT=3007
SERVER_HOST=0.0.0.0

# Security
API_KEY=your-dashboard-api-key
CORS_ORIGINS=http://localhost:3000,http://localhost:8080

# Caching
CACHE_TTL=300
CACHE_MAX_SIZE=1000

# Real-time
WEBSOCKET_ENABLED=true
WEBSOCKET_PORT=3008

# Monitoring
LOG_LEVEL=info
METRICS_PORT=9090
```

## 🗄️ **Database Schema**

### **Core Tables**
- `dashboard_widgets`: Widget configurations
- `dashboard_layouts`: Layout definitions
- `dashboard_data`: Cached dashboard data
- `analytics_metrics`: Performance metrics
- `real_time_events`: Real-time event data
- `export_jobs`: Data export tracking

### **Dashboard Features**
- **Widget Management**: Flexible widget system
- **Layout Persistence**: User layout preferences
- **Data Caching**: Performance optimization
- **Export Tracking**: Job management

## 📊 **Dashboard Features**

### **📈 Performance Summary**
```rust
// Get overview data
let overview = dashboard_service.get_overview(
    user_id,
    filters
).await?;

// Get performance metrics
let performance = dashboard_service.get_performance(
    period,
    metrics
).await?;

// Get analytics data
let analytics = dashboard_service.get_analytics(
    query,
    dimensions
).await?;
```

### **📊 Data Visualization**
```rust
// Create chart
let chart = chart_service.create_chart(
    chart_type,
    data,
    options
).await?;

// Get trend data
let trends = chart_service.get_trends(
    metric,
    period,
    granularity
).await?;

// Compare data
let comparison = chart_service.compare_data(
    datasets,
    comparison_type
).await?;
```

### **🔍 Data Queries**
```rust
// Execute query
let results = query_service.execute_query(
    query,
    parameters
).await?;

// Export data
let export = query_service.export_data(
    query,
    format
).await?;

// Aggregate data
let aggregated = query_service.aggregate_data(
    data,
    aggregation_rules
).await?;
```

### **⚙️ Dashboard Configuration**
```rust
// Create widget
let widget = widget_service.create_widget(
    widget_type,
    configuration,
    user_id
).await?;

// Update layout
let layout = layout_service.update_layout(
    user_id,
    layout_config
).await?;

// Get user preferences
let preferences = preference_service.get_preferences(user_id).await?;
```

### **📱 Real-time Updates**
```rust
// Subscribe to updates
let subscription = realtime_service.subscribe(
    user_id,
    channels
).await?;

// Send update
realtime_service.send_update(
    channel,
    data
).await?;

// Get stream
let stream = realtime_service.get_stream(user_id).await?;
```

## 📊 **Performance Metrics**

### **📊 Dashboard Performance**
- **Data Loading**: ~100ms per widget
- **Chart Rendering**: ~50ms per chart
- **Real-time Updates**: ~10ms latency
- **Export Generation**: ~5s per export
- **Cache Hit Rate**: 90%+ cache efficiency

### **📈 Scalability**
- **Concurrent Users**: 1000+ users
- **Widget Count**: 100+ widgets per dashboard
- **Data Points**: 1M+ data points
- **Real-time Connections**: 500+ concurrent connections

## 🔒 **Security Features**

### **🔐 Data Protection**
- **Access Control**: Role-based permissions
- **Data Filtering**: User-specific data views
- **Audit Logging**: Dashboard access history
- **Export Security**: Secure data export

### **🛡️ Privacy Compliance**
- **Data Anonymization**: PII removal
- **Access Logging**: Complete audit trail
- **Data Retention**: Configurable retention policies
- **GDPR Compliance**: User data management

## 📈 **Monitoring & Observability**

### **Health Checks**
```bash
# Service health
curl http://localhost:3007/health

# Database health
curl http://localhost:3007/health/db

# Cache health
curl http://localhost:3007/health/cache

# Service dependencies
curl http://localhost:3007/health/services
```

### **Metrics**
- **Dashboard Usage**: Most accessed dashboards
- **Widget Performance**: Widget load times
- **Data Volume**: Data points processed
- **User Engagement**: User interaction patterns

### **Logging**
```rust
// Structured logging
info!("Dashboard accessed", user_id = user_id, dashboard_id = dashboard_id);
error!("Data loading failed", widget_id = widget_id, error = error);
```

## 🧪 **Testing**

### **Unit Tests**
```bash
cargo test
```

### **Integration Tests**
```bash
cargo test --test integration
```

### **Dashboard Tests**
```bash
# Test data aggregation
cargo test --test data_aggregation

# Test chart generation
cargo test --test chart_generation

# Test real-time updates
cargo test --test realtime_updates
```

## 🔄 **CI/CD**

### **GitHub Actions**
```yaml
name: Dashboard Service CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test
      - run: cargo clippy
      - run: cargo audit
```

## 🤝 **Integration**

### **Service Integration**
```rust
// Security service integration
let user_permissions = security_service.get_user_permissions(user_id).await?;

// AI service integration
let insights = ai_service.get_dashboard_insights(data).await?;

// Configuration service integration
let user_config = config_service.get_user_preferences(user_id).await?;
```

### **External Integrations**
- **Analytics Platforms**: Google Analytics, Mixpanel
- **Data Sources**: External APIs, databases
- **Visualization Libraries**: Chart.js, D3.js
- **Export Services**: PDF generation, Excel export

## 📚 **Documentation**

### **API Documentation**
- **OpenAPI 3.0**: Interactive API docs
- **Postman Collection**: API testing
- **Example Requests**: Sample API calls

### **User Guides**
- **Dashboard Creation**: How to create custom dashboards
- **Widget Configuration**: Widget setup and configuration
- **Data Export**: Export procedures and formats

## 🆘 **Support**

### **Getting Help**
- **Documentation**: Comprehensive guides
- **Issues**: GitHub issue tracker
- **Discussions**: Community forum
- **Dashboard Support**: Dashboard configuration help

### **Contact**
- **Email**: dasbor@simpelv2.go.id
- **Slack**: #simpelv2-dasbor
- **GitHub**: Issues dan discussions

---

**📊 Built with ❤️ and Rust for powerful and scalable dashboard system**
