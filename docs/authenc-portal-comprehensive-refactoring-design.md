# Design Document: Authenc & Portal Comprehensive Refactoring

## Overview

This document outlines a comprehensive architectural refactoring of the SIMPelv2 authentication and portal system. The refactoring addresses three major areas:

1. **Authenc Multi-Crate Architecture**: Breaking down the monolithic authenc service (70+ service modules, 989-line app.rs) into a modular multi-crate architecture following Rust best practices
2. **Portal Service Architecture Decision**: Evaluating whether to merge layanan-portal into authenc or maintain separation with cleaner boundaries
3. **Portal Microfrontend Rebuild**: Complete redesign of the portal microfrontend with modern UI/UX and proper architecture

The goal is to create a maintainable, scalable, production-ready authentication and portal system that follows enterprise-grade patterns while maintaining backward compatibility and all existing security features.

## Main Algorithm/Workflow

```mermaid
sequenceDiagram
    participant Dev as Developer
    participant Old as Current Monolith
    participant New as New Multi-Crate
    participant Portal as Portal Service
    participant MF as Portal Microfrontend

    Dev->>Old: Analyze current structure
    Old-->>Dev: 70+ services, 989-line app.rs

    Dev->>New: Design multi-crate architecture
    New-->>Dev: authenc-core, authenc-api, authenc-grpc, etc.

    Dev->>Portal: Evaluate merge vs separate
    Portal-->>Dev: Decision: Keep separate with clean boundaries

    Dev->>MF: Rebuild with modern architecture
    MF-->>Dev: Clean component structure, proper state management

    Dev->>New: Implement migration strategy
    New-->>Old: Gradual migration, maintain compatibility
```

## Architecture

### Current State Analysis

```mermaid
graph TB
    subgraph "Current Monolithic Authenc"
        APP[app.rs<br/>989 lines<br/>50+ Arc fields]
        SVC[src/services/<br/>70+ modules]
        HDL[src/handlers/<br/>40+ handlers]
        DB[src/database/<br/>operations]
        GRPC[src/grpc/<br/>services]

        APP --> SVC
        APP --> HDL
        APP --> DB
        APP --> GRPC
    end

    subgraph "Current Portal Service"
        PS[layanan-portal]
        PSH[handlers/]
        PSM[middleware/]
        PSS[services/]

        PS --> PSH
        PS --> PSM
        PS --> PSS
    end

    subgraph "Current Portal Microfrontend"
        PMF[antarmuka/portal]
        PMFP[pages/]
        PMFC[components/]
        PMFF[features/]

        PMF --> PMFP
        PMF --> PMFC
        PMF --> PMFF
    end

    PMF -->|REST API| PS
    PS -->|gRPC| APP

    style APP fill:#ff9999
    style SVC fill:#ff9999
    style PS fill:#ffcc99
    style PMF fill:#99ccff
```
