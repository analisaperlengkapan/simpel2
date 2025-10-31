# Requirements Document - Juncto Video Conferencing System

## Introduction

Juncto adalah sistem video conferencing yang dirancang khusus untuk Kejaksaan Agung RI, terintegrasi penuh dengan ekosistem SIMPelv2. Sistem ini menyediakan komunikasi video real-time yang aman untuk rapat internal, sidang virtual, pelatihan jarak jauh, dan kolaborasi antar divisi dengan standar keamanan tinggi yang sesuai untuk institusi pemerintah.

Juncto dibangun dengan arsitektur microfrontend (Leptos + WebAssembly) dan microservices (Rust + Axum), menggunakan WebRTC untuk komunikasi peer-to-peer dan media server untuk skalabilitas, dengan enkripsi end-to-end dan integrasi seamless dengan Authenc (IAM) dan Secreton (secret management).

## Glossary

- **Juncto System**: Sistem video conferencing yang terdiri dari frontend microfrontend dan backend microservice
- **Meeting Room**: Ruang virtual untuk video conference dengan ID unik
- **Participant**: Pengguna yang bergabung dalam meeting room
- **Host**: Pengguna yang membuat dan mengelola meeting room
- **Media Server**: Server yang menangani routing dan processing media streams (SFU - Selective Forwarding Unit)
- **Signaling Server**: Server yang menangani WebRTC signaling dan room management
- **WebRTC**: Web Real-Time Communication protocol untuk peer-to-peer media streaming
- **SFU**: Selective Forwarding Unit - arsitektur media server yang meneruskan streams tanpa transcoding
- **TURN Server**: Traversal Using Relays around NAT - server relay untuk koneksi yang tidak bisa peer-to-peer
- **STUN Server**: Session Traversal Utilities for NAT - server untuk NAT traversal discovery
- **Recording Service**: Layanan untuk merekam meeting sessions
- **Portal Integration**: Integrasi dengan antarmuka/portal untuk akses terpusat
- **Authenc Integration**: Integrasi dengan sistem IAM untuk autentikasi dan autorisasi
- **Secreton Integration**: Integrasi dengan secret management untuk credential dan key management

## Requirements

### Requirement 1: User Authentication and Authorization

**User Story:** As a government employee, I want to authenticate using my existing SIMPelv2 credentials so that I can securely access video conferencing without managing separate accounts

#### Acceptance Criteria

1. WHEN a user accesses Juncto, THE Juncto System SHALL authenticate the user via Authenc using OAuth2/OIDC flow
2. WHEN authentication succeeds, THE Juncto System SHALL retrieve user profile including name, division, role, and permissions from Authenc
3. WHEN a user attempts to create a meeting, THE Juncto System SHALL verify the user has "meeting.create" permission via Authenc
4. WHEN a user attempts to join a meeting, THE Juncto System SHALL verify the user has "meeting.join" permission via Authenc
5. WHEN a user session expires, THE Juncto System SHALL redirect the user to Authenc for re-authentication

### Requirement 2: Meeting Room Management

**User Story:** As a meeting host, I want to create and manage meeting rooms with various settings so that I can control the meeting environment according to my needs

#### Acceptance Criteria

1. WHEN a host creates a meeting, THE Juncto System SHALL generate a unique meeting ID and return a shareable meeting URL
2. WHEN creating a meeting, THE Juncto System SHALL allow the host to configure settings including title, description, scheduled time, duration, maximum participants, waiting room, recording, and access control
3. WHEN a host enables waiting room, THE Juncto System SHALL hold participants in a waiting area until the host admits them
4. WHEN a host enables recording, THE Juncto System SHALL store the recording configuration in Secreton for secure access
5. WHEN a meeting is scheduled, THE Juncto System SHALL send notifications to invited participants via the notification service
6. WHEN a host deletes a meeting, THE Juncto System SHALL revoke access and notify all invited participants

### Requirement 3: Real-Time Video and Audio Communication

**User Story:** As a participant, I want to communicate with other participants using high-quality video and audio so that I can effectively collaborate in virtual meetings

#### Acceptance Criteria

1. WHEN a participant joins a meeting, THE Juncto System SHALL establish WebRTC peer connections via the Media Server using SFU architecture
2. WHEN media streams are transmitted, THE Juncto System SHALL encrypt all audio and video data using DTLS-SRTP
3. WHEN network conditions change, THE Juncto System SHALL adapt video quality dynamically to maintain connection stability
4. WHEN a participant enables their camera, THE Juncto System SHALL transmit video at resolutions up to 1080p with adaptive bitrate
5. WHEN a participant enables their microphone, THE Juncto System SHALL transmit audio with echo cancellation and noise suppression
6. WHEN direct peer connection fails, THE Juncto System SHALL fallback to TURN relay server for media transmission

### Requirement 4: Screen Sharing and Presentation

**User Story:** As a presenter, I want to share my screen or specific application windows so that I can present documents, slides, or demonstrations to meeting participants

#### Acceptance Criteria

1. WHEN a participant initiates screen sharing, THE Juncto System SHALL request screen capture permission from the browser
2. WHEN screen sharing is active, THE Juncto System SHALL transmit the screen stream at up to 30 fps with resolution up to 1920x1080
3. WHEN multiple participants attempt screen sharing, THE Juncto System SHALL allow only one active screen share at a time unless host permits multiple
4. WHEN screen sharing includes audio, THE Juncto System SHALL capture and transmit system audio along with the screen stream
5. WHEN a participant stops screen sharing, THE Juncto System SHALL immediately terminate the screen stream and notify all participants

### Requirement 5: Meeting Recording and Playback

**User Story:** As a meeting host, I want to record meetings and access recordings later so that I can review discussions and share with absent participants

#### Acceptance Criteria

1. WHEN a host starts recording, THE Juncto System SHALL notify all participants that recording is in progress
2. WHEN recording is active, THE Recording Service SHALL capture all audio, video, and screen sharing streams into a single MP4 file
3. WHEN recording completes, THE Juncto System SHALL store the recording file in encrypted storage with access credentials managed by Secreton
4. WHEN a user requests recording playback, THE Juncto System SHALL verify the user has permission to access the recording via Authenc
5. WHEN a recording is deleted, THE Juncto System SHALL permanently remove the file and revoke all access credentials

### Requirement 6: Chat and Messaging

**User Story:** As a participant, I want to send text messages during meetings so that I can share links, ask questions, or communicate without interrupting speakers

#### Acceptance Criteria

1. WHEN a participant sends a chat message, THE Juncto System SHALL deliver the message to all participants in real-time via WebSocket
2. WHEN a participant sends a private message, THE Juncto System SHALL deliver the message only to the specified recipient
3. WHEN a participant shares a file via chat, THE Juncto System SHALL upload the file to secure storage and share a download link
4. WHEN a meeting ends, THE Juncto System SHALL optionally save the chat transcript if enabled by the host
5. WHEN a participant joins late, THE Juncto System SHALL optionally show previous chat messages if enabled by the host

### Requirement 7: Participant Management and Controls

**User Story:** As a meeting host, I want to manage participants and control their permissions so that I can maintain order and security in meetings

#### Acceptance Criteria

1. WHEN a host views the participant list, THE Juncto System SHALL display all participants with their name, division, connection status, and current media state
2. WHEN a host mutes a participant, THE Juncto System SHALL disable the participant's microphone and prevent them from unmuting without host permission
3. WHEN a host removes a participant, THE Juncto System SHALL disconnect the participant and optionally prevent them from rejoining
4. WHEN a host promotes a participant to co-host, THE Juncto System SHALL grant the participant host-level permissions
5. WHEN a host locks the meeting, THE Juncto System SHALL prevent new participants from joining

### Requirement 8: Portal Integration

**User Story:** As a SIMPelv2 user, I want to access Juncto from the main portal so that I have a unified experience across all system modules

#### Acceptance Criteria

1. WHEN a user navigates to the portal, THE Portal System SHALL display Juncto as an available module in the navigation menu
2. WHEN a user clicks the Juncto module, THE Portal System SHALL load the Juncto microfrontend using module federation
3. WHEN Juncto loads, THE Juncto System SHALL use the existing portal authentication session without requiring re-login
4. WHEN a user receives a meeting invitation, THE Portal System SHALL display a notification with a direct link to join the meeting
5. WHEN a user creates a meeting from another module, THE Portal System SHALL allow embedding Juncto meeting creation via shared components

### Requirement 9: Notification and Calendar Integration

**User Story:** As a user, I want to receive notifications about upcoming meetings and integrate with my calendar so that I don't miss important meetings

#### Acceptance Criteria

1. WHEN a meeting is scheduled, THE Juncto System SHALL send email notifications to all invited participants via the notification service
2. WHEN a meeting starts, THE Juncto System SHALL send real-time notifications to invited participants who haven't joined
3. WHEN a meeting is rescheduled, THE Juncto System SHALL send update notifications to all participants
4. WHEN a user schedules a meeting, THE Juncto System SHALL provide an iCalendar (.ics) file for calendar integration
5. WHEN a meeting reminder is due, THE Juncto System SHALL send reminder notifications 15 minutes before the scheduled start time

### Requirement 10: Security and Compliance

**User Story:** As a security officer, I want all video conferencing to meet government security standards so that sensitive discussions remain confidential

#### Acceptance Criteria

1. WHEN media streams are transmitted, THE Juncto System SHALL encrypt all data using DTLS-SRTP with minimum AES-128-GCM cipher
2. WHEN signaling messages are exchanged, THE Juncto System SHALL use TLS 1.3 with certificate pinning for all WebSocket connections
3. WHEN a meeting room is created, THE Juncto System SHALL generate a cryptographically secure meeting ID using CSPRNG
4. WHEN storing recordings, THE Juncto System SHALL encrypt files at rest using AES-256-GCM with keys managed by Secreton
5. WHEN audit logging is required, THE Juncto System SHALL log all meeting events including join, leave, recording start/stop, and permission changes

### Requirement 11: Performance and Scalability

**User Story:** As a system administrator, I want the system to handle multiple concurrent meetings efficiently so that all users have a smooth experience

#### Acceptance Criteria

1. WHEN the system is under load, THE Media Server SHALL support at least 100 concurrent meetings with 10 participants each
2. WHEN a participant joins a meeting, THE Juncto System SHALL establish media connections within 3 seconds under normal network conditions
3. WHEN network latency exceeds 200ms, THE Juncto System SHALL display a connection quality indicator to participants
4. WHEN CPU usage exceeds 80%, THE Media Server SHALL scale horizontally by adding additional server instances
5. WHEN a media server fails, THE Juncto System SHALL automatically reconnect participants to a healthy server instance

### Requirement 12: Accessibility and User Experience

**User Story:** As a user with accessibility needs, I want the interface to be accessible so that I can participate in meetings effectively

#### Acceptance Criteria

1. WHEN a user navigates the interface, THE Juncto System SHALL support full keyboard navigation for all controls
2. WHEN screen reader is active, THE Juncto System SHALL provide ARIA labels and announcements for all interactive elements
3. WHEN a participant speaks, THE Juncto System SHALL optionally display live captions using speech-to-text
4. WHEN color is used to convey information, THE Juncto System SHALL provide alternative indicators for colorblind users
5. WHEN a user adjusts settings, THE Juncto System SHALL persist preferences including theme, layout, and accessibility options

### Requirement 13: Mobile and Cross-Platform Support

**User Story:** As a mobile user, I want to join meetings from my smartphone or tablet so that I can participate when away from my desk

#### Acceptance Criteria

1. WHEN a user accesses Juncto from a mobile browser, THE Juncto System SHALL provide a responsive interface optimized for touch input
2. WHEN a mobile user joins a meeting, THE Juncto System SHALL support video and audio with mobile-optimized codecs (VP8/VP9, Opus)
3. WHEN a mobile device rotates, THE Juncto System SHALL adapt the layout to portrait or landscape orientation
4. WHEN mobile network switches between WiFi and cellular, THE Juncto System SHALL maintain the connection using ICE restart
5. WHEN battery is low, THE Juncto System SHALL optionally disable video to conserve power

### Requirement 14: Analytics and Monitoring

**User Story:** As an administrator, I want to monitor system usage and performance so that I can optimize resources and troubleshoot issues

#### Acceptance Criteria

1. WHEN meetings occur, THE Juncto System SHALL collect metrics including participant count, duration, media quality, and connection failures
2. WHEN system health is queried, THE Juncto System SHALL expose Prometheus metrics for monitoring via the /metrics endpoint
3. WHEN errors occur, THE Juncto System SHALL log detailed error information with correlation IDs for troubleshooting
4. WHEN generating reports, THE Juncto System SHALL provide usage statistics including total meetings, total participants, and average duration
5. WHEN performance degrades, THE Juncto System SHALL send alerts to administrators via the notification service

### Requirement 15: Breakout Rooms

**User Story:** As a meeting host, I want to create breakout rooms so that participants can have smaller group discussions

#### Acceptance Criteria

1. WHEN a host creates breakout rooms, THE Juncto System SHALL allow configuring the number of rooms and assignment method (manual or automatic)
2. WHEN breakout rooms open, THE Juncto System SHALL move participants to their assigned rooms and establish new media connections
3. WHEN a participant is in a breakout room, THE Juncto System SHALL allow the host to broadcast messages to all rooms
4. WHEN breakout rooms close, THE Juncto System SHALL return all participants to the main meeting room
5. WHEN a participant requests help in a breakout room, THE Juncto System SHALL notify the host and allow them to join that room

### Requirement 16: Virtual Backgrounds and Video Effects

**User Story:** As a participant, I want to use virtual backgrounds and video effects so that I can maintain privacy and professionalism

#### Acceptance Criteria

1. WHEN a participant enables virtual background, THE Juncto System SHALL apply background replacement using WebAssembly-based segmentation
2. WHEN processing video, THE Juncto System SHALL support blur effect with adjustable intensity
3. WHEN custom backgrounds are uploaded, THE Juncto System SHALL validate file type and size before applying
4. WHEN video effects are active, THE Juncto System SHALL process frames at minimum 15 fps to maintain quality
5. WHEN device performance is insufficient, THE Juncto System SHALL disable video effects and notify the user

### Requirement 17: Waiting Room and Lobby

**User Story:** As a meeting host, I want to control who enters my meeting so that I can prevent unauthorized access

#### Acceptance Criteria

1. WHEN waiting room is enabled, THE Juncto System SHALL hold joining participants in a lobby until admitted by the host
2. WHEN a participant is in the waiting room, THE Juncto System SHALL display their name and division to the host
3. WHEN a host admits a participant, THE Juncto System SHALL immediately connect them to the meeting room
4. WHEN a host denies entry, THE Juncto System SHALL disconnect the participant and optionally block them from rejoining
5. WHEN a host is not present, THE Juncto System SHALL automatically admit participants if configured to do so

### Requirement 18: Meeting Templates and Presets

**User Story:** As a frequent meeting organizer, I want to save meeting configurations as templates so that I can quickly create similar meetings

#### Acceptance Criteria

1. WHEN a host creates a template, THE Juncto System SHALL save all meeting settings including duration, permissions, and features
2. WHEN a host uses a template, THE Juncto System SHALL pre-fill the meeting creation form with template values
3. WHEN a template is updated, THE Juncto System SHALL not affect existing meetings created from that template
4. WHEN a host deletes a template, THE Juncto System SHALL confirm the action and remove the template permanently
5. WHEN templates are listed, THE Juncto System SHALL display template name, description, and last used date

### Requirement 19: Integration with Document Management

**User Story:** As a participant, I want to share documents from the SIMPelv2 document system so that I can reference official files during meetings

#### Acceptance Criteria

1. WHEN a participant shares a document, THE Juncto System SHALL integrate with layanan/shared/dokumen to browse available files
2. WHEN a document is selected, THE Juncto System SHALL display the document in a shared viewer visible to all participants
3. WHEN viewing a document, THE Juncto System SHALL support annotations and pointer tools for collaboration
4. WHEN a document is shared, THE Juncto System SHALL verify the participant has permission to access the document via Authenc
5. WHEN a meeting ends, THE Juncto System SHALL optionally attach shared documents to the meeting record

### Requirement 20: Emergency Broadcast and Priority Meetings

**User Story:** As an administrator, I want to initiate emergency broadcasts so that I can communicate urgent information to all users

#### Acceptance Criteria

1. WHEN an emergency broadcast is initiated, THE Juncto System SHALL create a priority meeting and send immediate notifications to all users
2. WHEN a priority meeting starts, THE Juncto System SHALL display a prominent alert in the portal for all logged-in users
3. WHEN users join an emergency meeting, THE Juncto System SHALL bypass waiting room and admission controls
4. WHEN an emergency broadcast is active, THE Juncto System SHALL automatically record the session for compliance
5. WHEN the emergency ends, THE Juncto System SHALL archive the meeting with special classification for audit purposes
