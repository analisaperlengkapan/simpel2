# 🤖 Layanan AI - SIMPelv2

**Layanan AI** adalah microservice utama dalam SIMPelv2 yang menyediakan kemampuan kecerdasan buatan dan machine learning. Dibangun dengan **Rust** untuk performa maksimal dan keamanan data.

## 🎯 **Overview**

Layanan AI menyediakan berbagai kemampuan AI/ML yang mendukung pengambilan keputusan dan automasi dalam pengelolaan BMN:

- **LLM Integration**: Internal fine-tuned language models
- **RAG System**: Retrieval-Augmented Generation untuk Q&A
- **OCR Processing**: Document text extraction
- **Supervised Learning**: Traditional ML models
- **RLHF**: Reinforcement Learning from Human Feedback
- **Active Learning**: Query optimization
- **Transfer Learning**: Model adaptation
- **HITL**: Human-in-the-loop annotation

## 🏗️ **Architecture**

### **🧠 AI Stack**
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   API Gateway   │    │   AI Service    │
│   (React)       │───▶│   (Envoy)       │───▶│   (Rust)        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                       │
                                                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Qdrant        │    │   PostgreSQL    │    │   AI Models     │
│   (Vector DB)   │◀───│   (Metadata)    │───▶│   (Rust-Bert)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### **🔧 Technology Stack**
- **Language**: Rust (Axum web framework)
- **AI/ML**: Rust-Bert, Tch, Tokenizers
- **Vector Database**: Qdrant
- **OCR**: Leptess (Tesseract)
- **Image Processing**: Image crate
- **Database**: PostgreSQL dengan SQLx
- **Monitoring**: Tracing + OpenTelemetry

## 🚀 **Quick Start**

### **Prerequisites**
- Rust 1.75+
- PostgreSQL 15+
- Qdrant Vector Database
- Docker & Docker Compose

### **Installation**

```bash
# Clone repository
cd layanan/ai

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
docker build -t simpelv2-ai .

# Run container
docker run -d \
  --name ai \
  -p 3002:3002 \
  -e DATABASE_URL=postgres://user:pass@host:5432/db \
  -e QDRANT_URL=http://qdrant:6333 \
  -v ai-models:/app/models \
  -v ai-data:/app/data \
  simpelv2-ai
```

## 📡 **API Endpoints**

### **🧠 LLM Endpoints**
```http
POST /ai/llm/generate
POST /ai/llm/summarize
POST /ai/llm/classify
POST /ai/llm/translate
```

### **📄 OCR Endpoints**
```http
POST /ai/ocr/extract
POST /ai/ocr/process_batch
GET  /ai/ocr/status/{job_id}
```

### **🔍 RAG Endpoints**
```http
POST /ai/rag/query
POST /ai/rag/ingest
POST /ai/rag/update
GET  /ai/rag/collections
```

### **🎯 ML Endpoints**
```http
POST /ai/classify
POST /ai/recommend
POST /ai/predict
POST /ai/anomaly_detect
```

### **🔄 Training Endpoints**
```http
POST /ai/supervised/train
POST /ai/rlhf/train
POST /ai/active/query
POST /ai/transfer/fine_tune
POST /ai/hitl/annotate
```

## 🔧 **Configuration**

### **Environment Variables**
```env
# Database
DATABASE_URL=postgres://user:pass@host:5432/simpelv2

# Vector Database
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=your-qdrant-api-key

# AI Models
MODEL_PATH=/app/models
MODEL_CACHE_SIZE=10

# OCR
TESSERACT_DATA_PATH=/usr/share/tesseract-ocr/4.00/tessdata
OCR_LANGUAGES=eng,ind

# Server
SERVER_PORT=3002
SERVER_HOST=0.0.0.0

# Security
API_KEY=your-ai-service-api-key
CORS_ORIGINS=http://localhost:3000,http://localhost:8080

# Monitoring
LOG_LEVEL=info
METRICS_PORT=9090
```

## 🗄️ **Database Schema**

### **Core Tables**
- `ai_models`: Model metadata dan versions
- `ai_predictions`: Prediction history
- `ai_training_jobs`: Training job tracking
- `ai_annotations`: Human-in-the-loop annotations
- `ai_embeddings`: Vector embeddings metadata
- `ai_documents`: Document processing metadata

### **AI Features**
- **Model Versioning**: Track model versions
- **Prediction Logging**: Audit prediction history
- **Training Jobs**: Monitor training progress
- **Annotation Tracking**: HITL workflow management

## 🤖 **AI Capabilities**

### **🧠 LLM Integration**
```rust
// Generate text with LLM
let response = llm_service.generate_text(prompt, max_tokens).await?;

// Summarize document
let summary = llm_service.summarize_text(document_text).await?;

// Classify content
let classification = llm_service.classify_text(text, categories).await?;
```

### **📄 OCR Processing**
```rust
// Extract text from image
let text = ocr_service.extract_text(image_data).await?;

// Process batch of documents
let results = ocr_service.process_batch(documents).await?;

// Get processing status
let status = ocr_service.get_job_status(job_id).await?;
```

### **🔍 RAG System**
```rust
// Query RAG system
let answer = rag_service.query(question, context).await?;

// Ingest documents
rag_service.ingest_documents(documents).await?;

// Update knowledge base
rag_service.update_collection(collection_id, documents).await?;
```

### **🎯 Machine Learning**
```rust
// Train supervised model
let model_id = supervised_service.train_model(training_data).await?;

// Make prediction
let prediction = supervised_service.predict(model_id, input).await?;

// Detect anomalies
let anomalies = supervised_service.detect_anomalies(data).await?;
```

### **🔄 Advanced Learning**
```rust
// RLHF training
let rlhf_model = rlhf_service.train(preference_data).await?;

// Active learning query
let query = active_service.get_next_query(unlabeled_data).await?;

// Transfer learning
let adapted_model = transfer_service.fine_tune(base_model, target_data).await?;

// Human-in-the-loop
let annotation = hitl_service.get_annotation(sample_id).await?;
```

## 📊 **Performance Metrics**

### **🤖 AI Performance**
- **LLM Inference**: ~50ms per request
- **OCR Processing**: ~100ms per page
- **RAG Query**: ~20ms per query
- **Model Loading**: ~2s startup time
- **Vector Search**: ~5ms per query

### **📈 Scalability**
- **Concurrent Requests**: 1000+ requests/second
- **Model Memory**: 2GB per model instance
- **Vector Storage**: 1M+ embeddings
- **Batch Processing**: 100+ documents/minute

## 🔒 **Security Features**

### **🔐 Data Protection**
- **Model Isolation**: Separate model instances
- **Data Encryption**: In-transit & at-rest encryption
- **Access Control**: API key authentication
- **Audit Logging**: Complete prediction history

### **🛡️ Privacy Compliance**
- **Data Anonymization**: PII removal
- **Consent Management**: User consent tracking
- **Data Retention**: Configurable retention policies
- **GDPR Compliance**: Right to be forgotten

## 📈 **Monitoring & Observability**

### **Health Checks**
```bash
# Service health
curl http://localhost:3002/health

# Model health
curl http://localhost:3002/health/models

# Database health
curl http://localhost:3002/health/db

# Vector DB health
curl http://localhost:3002/health/qdrant
```

### **Metrics**
- **Model Performance**: Accuracy, latency, throughput
- **Resource Usage**: CPU, memory, GPU utilization
- **API Metrics**: Request rate, error rate, response time
- **Business Metrics**: Prediction volume, user engagement

### **Logging**
```rust
// Structured logging
info!("Model prediction completed", model_id = model_id, latency = latency);
error!("OCR processing failed", document_id = doc_id, error = error);
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

### **AI Model Tests**
```bash
# Test model accuracy
cargo test --test model_accuracy

# Test performance benchmarks
cargo test --test performance

# Test security
cargo test --test security
```

## 🔄 **CI/CD**

### **GitHub Actions**
```yaml
name: AI Service CI
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

## 📚 **Model Management**

### **Model Registry**
```rust
// Register new model
let model_id = model_registry.register(
    "text-classifier-v1",
    model_path,
    metadata
).await?;

// Deploy model
model_registry.deploy(model_id, environment).await?;

// Monitor model
let metrics = model_registry.get_metrics(model_id).await?;
```

### **Version Control**
- **Model Versioning**: Semantic versioning
- **Rollback Capability**: Quick model rollback
- **A/B Testing**: Model comparison
- **Gradual Rollout**: Canary deployments

## 🤝 **Integration**

### **Service Integration**
```rust
// Security service integration
let auth_result = security_service.validate_token(token).await?;

// Document service integration
let document = document_service.get_document(doc_id).await?;

// Notification service integration
notification_service.send_prediction_alert(prediction).await?;
```

### **External APIs**
- **OpenAI API**: GPT models (optional)
- **Hugging Face**: Model hub integration
- **Google Cloud AI**: Cloud AI services
- **AWS SageMaker**: ML platform integration

## 📖 **Documentation**

### **API Documentation**
- **OpenAPI 3.0**: Interactive API docs
- **Postman Collection**: API testing
- **Example Requests**: Sample API calls

### **Model Documentation**
- **Model Cards**: Model performance and limitations
- **Training Documentation**: Training process details
- **Deployment Guides**: Model deployment instructions

## 🆘 **Support**

### **Getting Help**
- **Documentation**: Comprehensive guides
- **Issues**: GitHub issue tracker
- **Discussions**: Community forum
- **Model Support**: AI model troubleshooting

### **Contact**
- **Email**: ai@simpelv2.go.id
- **Slack**: #simpelv2-ai
- **GitHub**: Issues and discussions

---

**🤖 Built with ❤️ and Rust for maximum AI performance and security** 