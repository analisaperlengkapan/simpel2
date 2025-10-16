//! Storage backend implementations

// Core storage backends (essential)
pub mod file;
pub mod postgres;
pub mod raft;
pub mod redis;

// Cloud storage backends (commonly used)
pub mod cassandra;
pub mod cockroachdb;
pub mod consul;
pub mod dynamodb;
pub mod etcd;
pub mod mongodb;
pub mod mysql;
pub mod s3;

// Azure and GCS backends
pub mod azure_blob;
pub mod gcs;

// Export core backends
pub use file::FileBackend;
pub use postgres::PostgresBackend;
pub use raft::{RaftConfig, RaftStorageBackend};
pub use redis::RedisBackend;

// Export cloud backends
pub use cassandra::{CassandraConfig, CassandraStorage};
pub use cockroachdb::{CockroachDBConfig, CockroachDBStorage};
pub use consul::{ConsulStorage, ConsulStorageConfig};
pub use dynamodb::{DynamoDBStorage, DynamoDBStorageConfig};
pub use etcd::{EtcdStorage, EtcdStorageConfig};
pub use mongodb::{MongoDBConfig, MongoDBStorage};
pub use mysql::{MySQLStorage, MySQLStorageConfig};
pub use s3::{S3Storage, S3StorageConfig};

// Export Azure and GCS backends
pub use azure_blob::{AzureBlobConfig, AzureBlobStorage};
pub use gcs::{GcsConfig, GoogleCloudStorage};
