//! Unit + handler tests for the Analisis Kebutuhan feature.

use super::handlers::{create_analisis, get_all_analisis, get_analisis_by_id};
use super::models::{AnalisisKebutuhan, CreateAnalisisRequest};
use super::repository::MockAnalisisRepository;
use super::services::AnalisisService;
use crate::shared::middleware::Claims;
use crate::shared::pagination::PaginationQuery;
use axum::Json;
use axum::extract::{Path, Query, State};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

fn sample(id: Uuid, judul: &str, uid: Option<Uuid>) -> AnalisisKebutuhan {
    AnalisisKebutuhan {
        id,
        judul: judul.to_string(),
        kategori: "TIK".to_string(),
        deskripsi: None,
        prioritas: "tinggi".to_string(),
        status: "draft".to_string(),
        estimasi_biaya: None,
        justifikasi: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: uid,
        updated_by: uid,
    }
}

fn mock_claims() -> Claims {
    Claims {
        nama: Some("Test User".to_string()),
        jabatan: Some("Admin".to_string()),
        name: Some("Test User".to_string()),
        nip: Some("123456789".to_string()),
        satker_code: Some("001".to_string()),
        ..Claims::with_roles(Uuid::new_v4(), "testuser", ["admin"])
    }
}

#[tokio::test]
async fn test_service_create_analisis() {
    let mut repo = MockAnalisisRepository::new();
    let user_id = Uuid::new_v4();
    repo.expect_create_analisis()
        .with(
            mockall::predicate::always(),
            mockall::predicate::eq(Some(user_id)),
        )
        .times(1)
        .returning(|req, uid| Ok(sample(Uuid::new_v4(), &req.judul, uid)));

    let service = AnalisisService::new(Arc::new(repo));
    let req = CreateAnalisisRequest {
        judul: "New Analisis".to_string(),
        kategori: "TIK".to_string(),
        deskripsi: None,
        prioritas: "tinggi".to_string(),
        estimasi_biaya: None,
        justifikasi: None,
    };
    let result = service.create_analisis(req, Some(user_id)).await.unwrap();
    assert_eq!(result.judul, "New Analisis");
}

#[tokio::test]
async fn test_handler_get_all_analisis() {
    let mut repo = MockAnalisisRepository::new();
    repo.expect_get_all_analisis()
        .with(mockall::predicate::eq(1), mockall::predicate::eq(20))
        .times(1)
        .returning(|_, _| Ok((vec![sample(Uuid::new_v4(), "Analisis Server", None)], 1)));

    let state = State(AnalisisService::new(Arc::new(repo)));
    let pagination = Query(PaginationQuery {
        page: 1,
        per_page: 20,
        category: None,
    });
    let result = get_all_analisis(state, pagination, mock_claims()).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.0.data.len(), 1);
    assert_eq!(response.0.data[0].judul, "Analisis Server");
}

#[tokio::test]
async fn test_handler_create_analisis() {
    let mut repo = MockAnalisisRepository::new();
    let user_id = Uuid::new_v4();
    repo.expect_create_analisis()
        .with(mockall::predicate::always(), mockall::predicate::always())
        .times(1)
        .returning(move |req, uid| Ok(sample(Uuid::new_v4(), &req.judul, uid)));

    let state = State(AnalisisService::new(Arc::new(repo)));
    let mut claims = mock_claims();
    claims.user_id = user_id;
    let request = Json(CreateAnalisisRequest {
        judul: "Handler Analisis".to_string(),
        kategori: "UMUM".to_string(),
        deskripsi: None,
        prioritas: "sedang".to_string(),
        estimasi_biaya: None,
        justifikasi: None,
    });

    let result = create_analisis(state, claims, request).await;

    assert!(result.is_ok());
    let (status, json) = result.unwrap();
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(json.0.data.judul, "Handler Analisis");
}

#[tokio::test]
async fn test_handler_get_analisis_by_id() {
    let mut repo = MockAnalisisRepository::new();
    let id = Uuid::new_v4();
    repo.expect_get_analisis_by_id()
        .with(mockall::predicate::eq(id))
        .times(1)
        .returning(move |_| Ok(sample(id, "Test Analisis", None)));

    let state = State(AnalisisService::new(Arc::new(repo)));
    let result = get_analisis_by_id(state, Path(id), mock_claims()).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().0.data.judul, "Test Analisis");
}
