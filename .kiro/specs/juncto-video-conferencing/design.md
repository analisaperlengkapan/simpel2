# Design Document - Juncto Video Conferencing System

## Overview

Juncto adalah sistem video conferencing enterprise-grade yang dibangun dengan arsitektur modern untuk Kejaksaan Agung RI. Sistem ini menggunakan WebRTC untuk komunikasi real-time, SFU (Selective Forwarding Unit) untuk media routing yang efisien, dan terintegrasi penuh dengan ekosistem SIMPelv2.

### Design Goals

1. **Security First**: End-to-end encryption, zero-trust architecture, compliance dengan standar keamanan pemerintah
2. **High Performance**: Support 100+ concurrent meetings, low latency (<200ms), adaptive bitrate
3. **Seamless Integration**: Native integration dengan Portal, Authenc, Secreton, dan layanan SIMPelv2 lainnya
4. **Scalability**: Horizontal scaling untuk media servers, distributed architecture
5. **User Experience**: Intuitive interface, responsive design, accessibility compliant
6. **Reliability**: 99.9% uptime, automatic failover, graceful degradation

### Technology Stack

**Frontend (antarmuka/juncto)**:
- Leptos 0.8.x + WebAssembly
- WebRTC API (web-sys bindings)
- MediaStream API untuk camera/microphone
- Canvas API untuk virtual backgrounds
- WebSocket untuk signaling
- Shared microfrontend components

**Backend (layanan/shared/juncto)**:
- Rust + Axum 0.8.6
- WebRTC media server (mediasoup-rust or custom SFU)
- WebSocket (axum-tungstenite) untuk signaling
- PostgreSQL untuk persistence
- Redis untuk real-time state
- TURN/STUN servers (coturn)

**Integration**:
- Authenc untuk authentication/authorization
- Secreton untuk credential management
- Notification service untuk alerts
- Document service untuk file sharing
- Portal untuk unified access

## Architecture

### High-Level Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        Portal[Portal Microfrontend]
        Juncto[Juncto Microfrontend]
    end

    subgraph "API Gateway"
        Nginx[Nginx Reverse Proxy]
        Gerbang[Envoy API Gateway]
    end

    subgraph "Application Layer"
        SignalingAPI[Signaling API Service]
        MeetingAPI[Meeting Management API]
        RecordingAPI[Recording Service]
        AnalyticsAPI[Analytics Service]
    end

    subgraph "Media Layer"
        MediaServer1[Media Server 1 - SFU]
        MediaServer2[Media Server 2 - SFU]
        MediaServerN[Media Server N - SFU]
        TURNServer[TURN/STUN Server]
    end

    subgraph "Integration Layer"
        Authenc[Authenc IAM]
        Secreton[Secreton Secrets]
        NotifService[Notification Service]
        DocService[Document Service]
    end

    subgraph "Data Layer"
        PostgreSQL[(PostgreSQL)]
        Redis[(Redis)]
        S3[Object Storage]
    end

    Portal --> Juncto
    Juncto --> Nginx
    Nginx --> Gerbang
    Gerbang --> SignalingAPI
    Gerbang --> MeetingAPI
    Gerbang --> RecordingAPI
    Gerbang --> AnalyticsAPI

    SignalingAPI --> MediaServer1
    SignalingAPI --> MediaServer2
    SignalingAPI --> MediaServerN

    Juncto -.WebRTC.-> MediaServer1
    Juncto -.WebRTC.-> TURNServer

    MeetingAPI --> Authenc
    MeetingAPI --> Secreton
    MeetingAPI --> NotifService
    MeetingAPI --> DocService

    SignalingAPI --> Redis
    MeetingAPI --> PostgreSQL
    RecordingAPI --> S3
    AnalyticsAPI --> PostgreSQL
```

### Component Architecture

```mermaid
graph LR
    subgraph "Juncto Microfrontend"
        UI[UI Components]
        WebRTCClient[WebRTC Client]
        SignalingClient[Signaling Client]
        StateManager[State Manager]
    end

    subgraph "Juncto Backend Service"
        SignalingServer[Signaling Server]
        RoomManager[Room Manager]
        ParticipantManager[Participant Manager]
        MediaController[Media Controller]
        RecordingController[Recording Controller]
    end

    UI --> WebRTCClient
    UI --> SignalingClient
    WebRTCClient --> StateManager
    SignalingClient --> StateManager

    SignalingClient -.WebSocket.-> SignalingServer
    WebRTCClient -.WebRTC.-> MediaController

    SignalingServer --> RoomManager
    SignalingServer --> ParticipantManager
    RoomManager --> MediaController
    ParticipantManager --> MediaController
    MediaController --> RecordingController
```

## Components and Interfaces

### 1. Frontend Components (antarmuka/juncto)

#### 1.1 Core Components

**MeetingRoom Component**
- Main container for active meeting
- Manages video grid layout (gallery, speaker, presentation modes)
- Handles participant video/audio streams
- Coordinates with WebRTC client

**ControlBar Component**
- Audio/video toggle buttons
- Screen sharing control
- Chat toggle
- Participant list toggle
- Settings menu
- Leave/end meeting button

**VideoTile Component**
- Individual participant video display
- Audio level indicator
- Name overlay
- Connection quality indicator
- Pin/spotlight controls

**Chat Component**
- Message list with timestamps
- Message input with emoji support
- File sharing interface
- Private message support

**ParticipantList Component**
- List of all participants
- Host controls (mute, remove, promote)
- Waiting room management
- Breakout room assignment

**ScreenShare Component**
- Screen selection interface
- Active screen share display
- Annotation tools
- Presenter controls

**Settings Component**
- Device selection (camera, microphone, speaker)
- Video quality settings
- Audio settings (echo cancellation, noise suppression)
- Virtual background configuration
- Accessibility options

#### 1.2 WebRTC Client Module

```rust
pub struct WebRTCClient {
    peer_connections: HashMap<ParticipantId, RTCPeerConnection>,
    local_stream: Option<MediaStream>,
    remote_streams: HashMap<ParticipantId, MediaStream>,
    signaling_client: SignalingClient,
    ice_servers: Vec<IceServer>,
}

impl WebRTCClient {
    pub async fn initialize(&mut self) -> Result<()>;
    pub async fn join_room(&mut self, room_id: &str) -> Result<()>;
    pub async fn publish_stream(&mut self, stream: MediaStream) -> Result<()>;
    pub async fn subscribe_to_participant(&mut self, participant_id: ParticipantId) -> Result<()>;
    pub async fn enable_screen_share(&mut self) -> Result<()>;
    pub async fn toggle_audio(&mut self, enabled: bool) -> Result<()>;
    pub async fn toggle_video(&mut self, enabled: bool) -> Result<()>;
    pub async fn leave_room(&mut self) -> Result<()>;
}
```

#### 1.3 Signaling Client Module

```rust
pub struct SignalingClient {
    websocket: WebSocket,
    room_id: Option<String>,
    participant_id: Option<String>,
    message_handlers: HashMap<MessageType, Callback>,
}

impl SignalingClient {
    pub async fn connect(&mut self, url: &str) -> Result<()>;
    pub async fn join_room(&mut self, room_id: &str, token: &str) -> Result<JoinResponse>;
    pub async fn send_offer(&mut self, target: ParticipantId, offer: RTCSessionDescription) -> Result<()>;
    pub async fn send_answer(&mut self, target: ParticipantId, answer: RTCSessionDescription) -> Result<()>;
    pub async fn send_ice_candidate(&mut self, target: ParticipantId, candidate: RTCIceCandidate) -> Result<()>;
    pub async fn send_chat_message(&mut self, message: ChatMessage) -> Result<()>;
    pub fn on_message(&mut self, msg_type: MessageType, handler: Callback);
}
```

### 2. Backend Components (layanan/shared/juncto)

#### 2.1 Signaling Server

**Purpose**: Handle WebRTC signaling and room coordination

```rust
pub struct SignalingServer {
    rooms: Arc<RwLock<HashMap<RoomId, Room>>>,
    participants: Arc<RwLock<HashMap<ParticipantId, Participant>>>,
    websocket_connections: Arc<RwLock<HashMap<ParticipantId, WebSocketSender>>>,
    redis_client: RedisClient,
}

impl SignalingServer {
    pub async fn handle_connection(&self, ws: WebSocket, token: String) -> Result<()>;
    pub async fn handle_join_room(&self, participant_id: ParticipantId, room_id: RoomId) -> Result<()>;
    pub async fn handle_offer(&self, from: ParticipantId, to: ParticipantId, offer: Offer) -> Result<()>;
    pub async fn handle_answer(&self, from: ParticipantId, to: ParticipantId, answer: Answer) -> Result<()>;
    pub async fn handle_ice_candidate(&self, from: ParticipantId, to: ParticipantId, candidate: IceCandidate) -> Result<()>;
    pub async fn handle_leave_room(&self, participant_id: ParticipantId) -> Result<()>;
    pub async fn broadcast_to_room(&self, room_id: RoomId, message: SignalingMessage) -> Result<()>;
}
```

#### 2.2 Meeting Management Service

**Purpose**: CRUD operations for meetings, scheduling, permissions

```rust
pub struct MeetingService {
    db_pool: PgPool,
    authenc_client: AuthencClient,
    secreton_client: SecretonClient,
    notification_client: NotificationClient,
}

impl MeetingService {
    pub async fn create_meeting(&self, request: CreateMeetingRequest, user_id: UserId) -> Result<Meeting>;
    pub async fn get_meeting(&self, meeting_id: MeetingId, user_id: UserId) -> Result<Meeting>;
    pub async fn update_meeting(&self, meeting_id: MeetingId, request: UpdateMeetingRequest, user_id: UserId) -> Result<Meeting>;
    pub async fn delete_meeting(&self, meeting_id: MeetingId, user_id: UserId) -> Result<()>;
    pub async fn list_meetings(&self, user_id: UserId, filter: MeetingFilter) -> Result<Vec<Meeting>>;
    pub async fn generate_join_token(&self, meeting_id: MeetingId, user_id: UserId) -> Result<String>;
    pub async fn validate_join_token(&self, token: &str) -> Result<JoinTokenClaims>;
}
```

#### 2.3 Media Server (SFU)

**Purpose**: Route media streams efficiently without transcoding

**Architecture Decision**: Use mediasoup-rust or implement custom SFU

```rust
pub struct MediaServer {
    workers: Vec<Worker>,
    routers: HashMap<RoomId, Router>,
    transports: HashMap<TransportId, WebRtcTransport>,
    producers: HashMap<ProducerId, Producer>,
    consumers: HashMap<ConsumerId, Consumer>,
}

impl MediaServer {
    pub async fn create_router(&mut self, room_id: RoomId) -> Result<Router>;
    pub async fn create_webrtc_transport(&mut self, router_id: RouterId) -> Result<WebRtcTransport>;
    pub async fn connect_transport(&mut self, transport_id: TransportId, dtls_parameters: DtlsParameters) -> Result<()>;
    pub async fn produce(&mut self, transport_id: TransportId, kind: MediaKind, rtp_parameters: RtpParameters) -> Result<Producer>;
    pub async fn consume(&mut self, transport_id: TransportId, producer_id: ProducerId) -> Result<Consumer>;
    pub async fn close_producer(&mut self, producer_id: ProducerId) -> Result<()>;
    pub async fn close_consumer(&mut self, consumer_id: ConsumerId) -> Result<()>;
}
```


#### 2.4 Recording Service

**Purpose**: Record meetings and manage recordings

```rust
pub struct RecordingService {
    ffmpeg_pool: FfmpegPool,
    storage_client: S3Client,
    secreton_client: SecretonClient,
    db_pool: PgPool,
}

impl RecordingService {
    pub async fn start_recording(&self, room_id: RoomId, config: RecordingConfig) -> Result<RecordingId>;
    pub async fn stop_recording(&self, recording_id: RecordingId) -> Result<()>;
    pub async fn get_recording(&self, recording_id: RecordingId, user_id: UserId) -> Result<Recording>;
    pub async fn list_recordings(&self, meeting_id: MeetingId, user_id: UserId) -> Result<Vec<Recording>>;
    pub async fn delete_recording(&self, recording_id: RecordingId, user_id: UserId) -> Result<()>;
    pub async fn generate_playback_url(&self, recording_id: RecordingId, user_id: UserId) -> Result<String>;
}
```

#### 2.5 Analytics Service

**Purpose**: Collect and analyze meeting metrics

```rust
pub struct AnalyticsService {
    db_pool: PgPool,
    metrics_registry: Registry,
}

impl AnalyticsService {
    pub async fn record_meeting_event(&self, event: MeetingEvent) -> Result<()>;
    pub async fn record_participant_event(&self, event: ParticipantEvent) -> Result<()>;
    pub async fn record_media_quality(&self, metrics: MediaQualityMetrics) -> Result<()>;
    pub async fn get_meeting_statistics(&self, meeting_id: MeetingId) -> Result<MeetingStatistics>;
    pub async fn get_usage_report(&self, filter: ReportFilter) -> Result<UsageReport>;
    pub fn export_prometheus_metrics(&self) -> String;
}
```

## Data Models

### Database Schema (PostgreSQL)

```sql
-- Meetings table
CREATE TABLE meetings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    description TEXT,
    host_user_id UUID NOT NULL,
    scheduled_start_time TIMESTAMPTZ,
    scheduled_end_time TIMESTAMPTZ,
    actual_start_time TIMESTAMPTZ,
    actual_end_time TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL, -- scheduled, active, ended, cancelled
    max_participants INTEGER DEFAULT 100,
    waiting_room_enabled BOOLEAN DEFAULT false,
    recording_enabled BOOLEAN DEFAULT false,
    chat_enabled BOOLEAN DEFAULT true,
    screen_share_enabled BOOLEAN DEFAULT true,
    breakout_rooms_enabled BOOLEAN DEFAULT false,
    access_code VARCHAR(50),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_host FOREIGN KEY (host_user_id) REFERENCES users(id)
);

CREATE INDEX idx_meetings_host ON meetings(host_user_id);
CREATE INDEX idx_meetings_status ON meetings(status);
CREATE INDEX idx_meetings_scheduled_start ON meetings(scheduled_start_time);

-- Meeting participants table
CREATE TABLE meeting_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    user_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL, -- host, co-host, participant
    joined_at TIMESTAMPTZ,
    left_at TIMESTAMPTZ,
    duration_seconds INTEGER,
    is_admitted BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_meeting FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_participants_meeting ON meeting_participants(meeting_id);
CREATE INDEX idx_participants_user ON meeting_participants(user_id);

-- Meeting invitations table
CREATE TABLE meeting_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    user_id UUID NOT NULL,
    invited_by UUID NOT NULL,
    status VARCHAR(50) NOT NULL, -- pending, accepted, declined
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    responded_at TIMESTAMPTZ,
    CONSTRAINT fk_meeting FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE,
    CONSTRAINT fk_invitee FOREIGN KEY (user_id) REFERENCES users(id),
    CONSTRAINT fk_inviter FOREIGN KEY (invited_by) REFERENCES users(id)
);

CREATE INDEX idx_invitations_meeting ON meeting_invitations(meeting_id);
CREATE INDEX idx_invitations_user ON meeting_invitations(user_id);

-- Recordings table
CREATE TABLE recordings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size_bytes BIGINT,
    duration_seconds INTEGER,
    format VARCHAR(50) DEFAULT 'mp4',
    encryption_key_id VARCHAR(255), -- Reference to Secreton
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL, -- recording, processing, completed, failed
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_meeting FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE
);

CREATE INDEX idx_recordings_meeting ON recordings(meeting_id);
CREATE INDEX idx_recordings_status ON recordings(status);

-- Chat messages table
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    sender_user_id UUID NOT NULL,
    recipient_user_id UUID, -- NULL for public messages
    message_text TEXT NOT NULL,
    message_type VARCHAR(50) DEFAULT 'text', -- text, file, system
    file_url VARCHAR(500),
    sent_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_meeting FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE,
    CONSTRAINT fk_sender FOREIGN KEY (sender_user_id) REFERENCES users(id),
    CONSTRAINT fk_recipient FOREIGN KEY (recipient_user_id) REFERENCES users(id)
);

CREATE INDEX idx_chat_meeting ON chat_messages(meeting_id);
CREATE INDEX idx_chat_sender ON chat_messages(sender_user_id);

-- Meeting events table (for analytics)
CREATE TABLE meeting_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    user_id UUID,
    event_type VARCHAR(100) NOT NULL, -- join, leave, mute, unmute, screen_share_start, etc.
    event_data JSONB,
    occurred_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_meeting FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_events_meeting ON meeting_events(meeting_id);
CREATE INDEX idx_events_type ON meeting_events(event_type);
CREATE INDEX idx_events_occurred ON meeting_events(occurred_at);

-- Meeting templates table
CREATE TABLE meeting_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    template_config JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_templates_user ON meeting_templates(user_id);

-- Breakout rooms table
CREATE TABLE breakout_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    meeting_id UUID NOT NULL,
    room_number INTEGER NOT NULL,
    name VARCHAR(255),
    opened_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ,
    CONSTRAINT fk_meeting FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE
);

CREATE INDEX idx_breakout_meeting ON breakout_rooms(meeting_id);

-- Breakout room assignments table
CREATE TABLE breakout_room_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    breakout_room_id UUID NOT NULL,
    user_id UUID NOT NULL,
    assigned_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_breakout_room FOREIGN KEY (breakout_room_id) REFERENCES breakout_rooms(id) ON DELETE CASCADE,
    CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_assignments_room ON breakout_room_assignments(breakout_room_id);
CREATE INDEX idx_assignments_user ON breakout_room_assignments(user_id);
```

### Redis Data Structures

**Active Rooms** (Hash):
```
room:{room_id} -> {
    meeting_id: UUID,
    host_id: UUID,
    participant_count: Integer,
    status: String,
    media_server_id: String,
    created_at: Timestamp
}
```

**Active Participants** (Hash):
```
participant:{participant_id} -> {
    user_id: UUID,
    room_id: UUID,
    connection_id: String,
    audio_enabled: Boolean,
    video_enabled: Boolean,
    screen_sharing: Boolean,
    joined_at: Timestamp
}
```

**Room Participants** (Set):
```
room:{room_id}:participants -> Set<participant_id>
```

**Waiting Room** (List):
```
room:{room_id}:waiting -> List<participant_id>
```

**Media Server Load** (Sorted Set):
```
media_servers -> {
    server_id: score (participant_count)
}
```

### Rust Data Models

```rust
// Meeting model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub host_user_id: Uuid,
    pub scheduled_start_time: Option<DateTime<Utc>>,
    pub scheduled_end_time: Option<DateTime<Utc>>,
    pub actual_start_time: Option<DateTime<Utc>>,
    pub actual_end_time: Option<DateTime<Utc>>,
    pub status: MeetingStatus,
    pub max_participants: i32,
    pub waiting_room_enabled: bool,
    pub recording_enabled: bool,
    pub chat_enabled: bool,
    pub screen_share_enabled: bool,
    pub breakout_rooms_enabled: bool,
    pub access_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeetingStatus {
    Scheduled,
    Active,
    Ended,
    Cancelled,
}

// Participant model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub role: ParticipantRole,
    pub connection_id: String,
    pub audio_enabled: bool,
    pub video_enabled: bool,
    pub screen_sharing: bool,
    pub is_admitted: bool,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    Host,
    CoHost,
    Participant,
}

// Signaling messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    JoinRoom { room_id: String, token: String },
    JoinedRoom { participant_id: String, participants: Vec<ParticipantInfo> },
    ParticipantJoined { participant: ParticipantInfo },
    ParticipantLeft { participant_id: String },
    Offer { from: String, to: String, sdp: String },
    Answer { from: String, to: String, sdp: String },
    IceCandidate { from: String, to: String, candidate: String },
    ChatMessage { from: String, to: Option<String>, message: String },
    MediaStateChanged { participant_id: String, audio: bool, video: bool },
    ScreenShareStarted { participant_id: String },
    ScreenShareStopped { participant_id: String },
    RecordingStarted,
    RecordingStopped,
    Error { code: String, message: String },
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum JunctoError {
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("Meeting not found: {0}")]
    MeetingNotFound(Uuid),

    #[error("Meeting is full")]
    MeetingFull,

    #[error("Meeting has ended")]
    MeetingEnded,

    #[error("Invalid access code")]
    InvalidAccessCode,

    #[error("Participant not found: {0}")]
    ParticipantNotFound(String),

    #[error("Media server error: {0}")]
    MediaServerError(String),

    #[error("Recording error: {0}")]
    RecordingError(String),

    #[error("WebRTC error: {0}")]
    WebRTCError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for JunctoError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            JunctoError::AuthenticationError(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            JunctoError::AuthorizationError(_) => (StatusCode::FORBIDDEN, self.to_string()),
            JunctoError::MeetingNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            JunctoError::MeetingFull => (StatusCode::CONFLICT, self.to_string()),
            JunctoError::InvalidAccessCode => (StatusCode::UNAUTHORIZED, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
```

### Error Recovery Strategies

1. **Connection Failures**: Automatic reconnection with exponential backoff
2. **Media Server Failures**: Failover to backup media server
3. **Signaling Failures**: Queue messages and retry
4. **Recording Failures**: Notify host and continue meeting
5. **Database Failures**: Use Redis cache for critical operations


## Testing Strategy

### Unit Tests

**Frontend**:
- WebRTC client connection logic
- Signaling message handling
- State management
- UI component rendering
- Media device handling

**Backend**:
- Meeting CRUD operations
- Participant management
- Permission validation
- Signaling message routing
- Recording lifecycle

### Integration Tests

1. **Authentication Flow**: Test Authenc integration for login and token validation
2. **Meeting Lifecycle**: Create, join, leave, end meeting flow
3. **Media Streaming**: WebRTC connection establishment and media flow
4. **Recording**: Start, stop, and playback recording
5. **Chat**: Send and receive messages
6. **Notifications**: Meeting invitations and reminders
7. **Document Sharing**: Integration with document service

### End-to-End Tests

1. **Complete Meeting Flow**: User creates meeting, invites participants, conducts meeting, ends meeting
2. **Screen Sharing**: Participant shares screen and others view it
3. **Breakout Rooms**: Host creates breakout rooms, participants move between rooms
4. **Recording Playback**: Record meeting and play back recording
5. **Mobile Experience**: Join meeting from mobile browser

### Performance Tests

1. **Load Testing**: 100 concurrent meetings with 10 participants each
2. **Stress Testing**: Gradually increase load until system degrades
3. **Latency Testing**: Measure end-to-end latency for media and signaling
4. **Bandwidth Testing**: Test adaptive bitrate under various network conditions
5. **Failover Testing**: Simulate media server failure and measure recovery time

### Security Tests

1. **Authentication Bypass**: Attempt to join meeting without valid token
2. **Authorization Escalation**: Attempt to perform host actions as participant
3. **Injection Attacks**: Test SQL injection, XSS in chat messages
4. **Encryption Validation**: Verify DTLS-SRTP encryption is active
5. **Rate Limiting**: Test API rate limits and DDoS protection

## Integration Points

### 1. Authenc Integration

**Authentication Flow**:
```mermaid
sequenceDiagram
    participant User
    participant Juncto
    participant Authenc

    User->>Juncto: Access Juncto
    Juncto->>Authenc: Redirect to login
    User->>Authenc: Enter credentials
    Authenc->>Authenc: Validate credentials
    Authenc->>Juncto: Return access token
    Juncto->>Authenc: Validate token
    Authenc->>Juncto: Return user info
    Juncto->>User: Show dashboard
```

**API Endpoints Used**:
- `POST /oauth2/authorize` - Initiate OAuth2 flow
- `POST /oauth2/token` - Exchange code for token
- `GET /api/v1/userinfo` - Get user profile
- `POST /api/v1/permissions/check` - Verify permissions

**Permissions Required**:
- `juncto.meeting.create` - Create meetings
- `juncto.meeting.join` - Join meetings
- `juncto.meeting.host` - Host meetings
- `juncto.recording.access` - Access recordings
- `juncto.admin` - Administrative functions

### 2. Secreton Integration

**Use Cases**:
1. Store TURN server credentials
2. Store recording encryption keys
3. Store API keys for external services
4. Store TLS certificates

**API Endpoints Used**:
- `POST /api/v1/secrets` - Store secret
- `GET /api/v1/secrets/{id}` - Retrieve secret
- `DELETE /api/v1/secrets/{id}` - Delete secret
- `POST /api/v1/secrets/{id}/rotate` - Rotate secret

**Example Usage**:
```rust
// Store recording encryption key
let key_id = secreton_client
    .store_secret(Secret {
        name: format!("recording_key_{}", recording_id),
        value: encryption_key,
        metadata: json!({
            "recording_id": recording_id,
            "created_at": Utc::now(),
        }),
    })
    .await?;

// Retrieve key for playback
let secret = secreton_client
    .get_secret(&key_id)
    .await?;
let encryption_key = secret.value;
```

### 3. Portal Integration

**Module Federation Configuration**:
```toml
# Trunk.toml for antarmuka/juncto
[[hooks]]
stage = "post_build"
command = "sh"
command_arguments = ["-c", "wasm-bindgen --target web --out-dir dist/pkg dist/juncto.wasm"]

[serve]
port = 8084
address = "0.0.0.0"

[build]
public_url = "/juncto/"
dist = "dist"
```

**Portal Navigation Entry**:
```rust
// In antarmuka/portal/src/app.rs
NavigationItem {
    label: "Juncto",
    icon: "video-camera",
    path: "/juncto",
    permission: "juncto.access",
    module_url: "/juncto/pkg/juncto.js",
}
```

### 4. Notification Service Integration

**Notification Types**:
1. Meeting invitation
2. Meeting reminder (15 minutes before)
3. Meeting started
4. Recording available
5. Meeting cancelled/rescheduled

**API Endpoints Used**:
- `POST /api/v1/notifications/send` - Send notification
- `POST /api/v1/notifications/bulk` - Send bulk notifications

**Example Usage**:
```rust
notification_client
    .send_notification(Notification {
        recipient_user_ids: vec![user_id],
        title: "Meeting Invitation".to_string(),
        body: format!("You're invited to: {}", meeting.title),
        notification_type: NotificationType::MeetingInvitation,
        action_url: Some(format!("/juncto/join/{}", meeting.id)),
        priority: Priority::Normal,
    })
    .await?;
```

### 5. Document Service Integration

**Use Cases**:
1. Share documents during meetings
2. Attach documents to meeting records
3. Collaborative document viewing

**API Endpoints Used**:
- `GET /api/v1/documents` - List documents
- `GET /api/v1/documents/{id}` - Get document
- `POST /api/v1/documents/{id}/share` - Generate share link

## Deployment Architecture

### Container Structure

```yaml
# docker-compose.juncto.yml
version: '3.8'

services:
  juncto-signaling:
    image: simpelv2/juncto-signaling:latest
    environment:
      - DATABASE_URL=postgresql://user:pass@postgres:5432/juncto
      - REDIS_URL=redis://redis:6379
      - AUTHENC_URL=http://authenc:8080
      - SECRETON_URL=http://secreton:8081
    ports:
      - "8090:8090"
    depends_on:
      - postgres
      - redis
      - authenc
      - secreton
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '2'
          memory: 2G

  juncto-media:
    image: simpelv2/juncto-media:latest
    environment:
      - REDIS_URL=redis://redis:6379
      - RTC_MIN_PORT=40000
      - RTC_MAX_PORT=49999
    network_mode: host
    deploy:
      replicas: 2
      resources:
        limits:
          cpus: '4'
          memory: 4G

  juncto-recording:
    image: simpelv2/juncto-recording:latest
    environment:
      - S3_ENDPOINT=http://minio:9000
      - S3_BUCKET=juncto-recordings
      - SECRETON_URL=http://secreton:8081
    volumes:
      - recording-temp:/tmp/recordings
    deploy:
      replicas: 2
      resources:
        limits:
          cpus: '2'
          memory: 4G

  coturn:
    image: coturn/coturn:latest
    network_mode: host
    volumes:
      - ./config/turnserver.conf:/etc/coturn/turnserver.conf
    command: ["-c", "/etc/coturn/turnserver.conf"]

volumes:
  recording-temp:
```

### Kubernetes Deployment

```yaml
# infra/k8s/juncto-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: juncto-signaling
  namespace: simpelv2
spec:
  replicas: 3
  selector:
    matchLabels:
      app: juncto-signaling
  template:
    metadata:
      labels:
        app: juncto-signaling
    spec:
      containers:
      - name: signaling
        image: simpelv2/juncto-signaling:latest
        ports:
        - containerPort: 8090
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: juncto-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: juncto-secrets
              key: redis-url
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 8090
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8090
          initialDelaySeconds: 5
          periodSeconds: 5

---
apiVersion: v1
kind: Service
metadata:
  name: juncto-signaling
  namespace: simpelv2
spec:
  selector:
    app: juncto-signaling
  ports:
  - protocol: TCP
    port: 8090
    targetPort: 8090
  type: ClusterIP

---
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: juncto-media
  namespace: simpelv2
spec:
  selector:
    matchLabels:
      app: juncto-media
  template:
    metadata:
      labels:
        app: juncto-media
    spec:
      hostNetwork: true
      containers:
      - name: media
        image: simpelv2/juncto-media:latest
        env:
        - name: RTC_MIN_PORT
          value: "40000"
        - name: RTC_MAX_PORT
          value: "49999"
        resources:
          requests:
            cpu: 2000m
            memory: 2Gi
          limits:
            cpu: 4000m
            memory: 4Gi
```

### Nginx Configuration

```nginx
# infra/nginx/conf.d/juncto.conf
upstream juncto_signaling {
    least_conn;
    server juncto-signaling-1:8090;
    server juncto-signaling-2:8090;
    server juncto-signaling-3:8090;
}

server {
    listen 443 ssl http2;
    server_name simpel.kejaksaan.go.id;

    # Juncto frontend
    location /juncto/ {
        alias /var/www/juncto/;
        try_files $uri $uri/ /juncto/index.html;

        # Cache static assets
        location ~* \.(js|css|wasm|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }
    }

    # Juncto API
    location /api/juncto/ {
        proxy_pass http://juncto_signaling/;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket timeout
        proxy_read_timeout 3600s;
        proxy_send_timeout 3600s;
    }
}
```

## Monitoring and Observability

### Metrics

**Application Metrics** (Prometheus):
```rust
lazy_static! {
    static ref ACTIVE_MEETINGS: IntGauge = register_int_gauge!(
        "juncto_active_meetings",
        "Number of active meetings"
    ).unwrap();

    static ref ACTIVE_PARTICIPANTS: IntGauge = register_int_gauge!(
        "juncto_active_participants",
        "Number of active participants"
    ).unwrap();

    static ref MEETING_DURATION: Histogram = register_histogram!(
        "juncto_meeting_duration_seconds",
        "Meeting duration in seconds"
    ).unwrap();

    static ref MEDIA_QUALITY: Histogram = register_histogram!(
        "juncto_media_quality_score",
        "Media quality score (0-100)"
    ).unwrap();

    static ref CONNECTION_FAILURES: Counter = register_counter!(
        "juncto_connection_failures_total",
        "Total number of connection failures"
    ).unwrap();
}
```

**System Metrics**:
- CPU usage per service
- Memory usage per service
- Network bandwidth usage
- Disk I/O for recordings
- WebSocket connection count
- WebRTC peer connection count

### Logging

**Structured Logging** (tracing):
```rust
#[instrument(skip(self))]
async fn join_meeting(&self, meeting_id: Uuid, user_id: Uuid) -> Result<JoinResponse> {
    info!(
        meeting_id = %meeting_id,
        user_id = %user_id,
        "User attempting to join meeting"
    );

    // ... implementation

    info!(
        meeting_id = %meeting_id,
        user_id = %user_id,
        participant_count = participants.len(),
        "User successfully joined meeting"
    );

    Ok(response)
}
```

**Log Levels**:
- ERROR: System failures, unrecoverable errors
- WARN: Degraded performance, recoverable errors
- INFO: Important events (join, leave, recording start/stop)
- DEBUG: Detailed flow information
- TRACE: Very detailed debugging information

### Alerting

**Critical Alerts**:
1. Media server down
2. Database connection lost
3. Redis connection lost
4. High error rate (>5%)
5. High latency (>500ms p95)

**Warning Alerts**:
1. High CPU usage (>80%)
2. High memory usage (>80%)
3. High participant count (>80% capacity)
4. Recording failures
5. Slow database queries (>1s)

## Security Considerations

### Authentication & Authorization

1. **OAuth2/OIDC**: All users authenticate via Authenc
2. **JWT Tokens**: Short-lived access tokens (15 minutes)
3. **Refresh Tokens**: Long-lived refresh tokens (7 days)
4. **Permission-Based Access**: Fine-grained permissions for all operations
5. **Meeting Access Codes**: Optional additional security layer

### Encryption

1. **Transport Encryption**: TLS 1.3 for all HTTP/WebSocket connections
2. **Media Encryption**: DTLS-SRTP for all WebRTC streams
3. **Recording Encryption**: AES-256-GCM for stored recordings
4. **Database Encryption**: Encrypted columns for sensitive data

### Network Security

1. **TURN Authentication**: Time-limited credentials for TURN server
2. **ICE Candidate Filtering**: Only allow specific IP ranges
3. **Rate Limiting**: API rate limits per user and IP
4. **DDoS Protection**: Cloudflare or similar protection
5. **Firewall Rules**: Restrict access to media server ports

### Data Privacy

1. **GDPR Compliance**: User consent for recording
2. **Data Retention**: Automatic deletion of old recordings
3. **Access Logs**: Audit trail for all data access
4. **Anonymization**: Option to anonymize participant data
5. **Right to Deletion**: Users can request data deletion

## Performance Optimization

### Frontend Optimizations

1. **Code Splitting**: Lazy load components
2. **WASM Optimization**: Use `opt-level = "z"` for size
3. **Asset Compression**: Gzip/Brotli compression
4. **CDN**: Serve static assets from CDN
5. **Service Worker**: Cache static assets offline

### Backend Optimizations

1. **Connection Pooling**: Database and Redis connection pools
2. **Caching**: Redis cache for frequently accessed data
3. **Batch Operations**: Batch database writes
4. **Async I/O**: Tokio async runtime
5. **Load Balancing**: Distribute load across multiple instances

### Media Optimizations

1. **Adaptive Bitrate**: Adjust quality based on bandwidth
2. **Simulcast**: Send multiple quality streams
3. **SFU Architecture**: No transcoding overhead
4. **Codec Selection**: VP8/VP9 for video, Opus for audio
5. **Bandwidth Estimation**: Google Congestion Control

## Alternative Solutions Considered

### 1. Media Server Options

**Option A: mediasoup (Chosen)**
- Pros: High performance, SFU architecture, Rust bindings available
- Cons: Complex setup, requires deep WebRTC knowledge

**Option B: Janus Gateway**
- Pros: Mature, well-documented, plugin architecture
- Cons: C-based, harder to integrate with Rust

**Option C: Custom SFU**
- Pros: Full control, optimized for our use case
- Cons: High development effort, maintenance burden

**Decision**: Use mediasoup for production-ready SFU with Rust integration

### 2. Signaling Protocol

**Option A: WebSocket (Chosen)**
- Pros: Real-time, bidirectional, widely supported
- Cons: Requires persistent connections

**Option B: HTTP Long Polling**
- Pros: Works everywhere, no special server requirements
- Cons: Higher latency, more overhead

**Option C: Server-Sent Events**
- Pros: Simple, built-in browser support
- Cons: Unidirectional, requires separate channel for client-to-server

**Decision**: WebSocket for low-latency bidirectional communication

### 3. Recording Architecture

**Option A: Server-Side Recording (Chosen)**
- Pros: Reliable, consistent quality, no client overhead
- Cons: Server resources required

**Option B: Client-Side Recording**
- Pros: No server resources, works offline
- Cons: Unreliable, quality varies, privacy concerns

**Option C: Hybrid Approach**
- Pros: Fallback option, flexibility
- Cons: Complex implementation

**Decision**: Server-side recording for reliability and consistency

## Future Enhancements

### Phase 2 Features

1. **AI-Powered Features**:
   - Real-time transcription
   - Automatic meeting summaries
   - Speaker identification
   - Sentiment analysis

2. **Advanced Collaboration**:
   - Whiteboard integration
   - Collaborative document editing
   - Polls and surveys
   - Q&A sessions

3. **Enhanced Analytics**:
   - Engagement metrics
   - Attention tracking
   - Speaking time analysis
   - Meeting effectiveness scores

4. **Mobile Apps**:
   - Native iOS app
   - Native Android app
   - Better mobile experience

5. **Integration Expansion**:
   - Calendar integration (Google, Outlook)
   - Email integration
   - Slack/Teams notifications
   - Third-party app integrations

### Scalability Roadmap

1. **Multi-Region Deployment**: Deploy media servers in multiple regions
2. **Edge Computing**: Process media closer to users
3. **CDN Integration**: Distribute recordings via CDN
4. **Database Sharding**: Shard database by organization
5. **Microservices Split**: Split monolith into smaller services
