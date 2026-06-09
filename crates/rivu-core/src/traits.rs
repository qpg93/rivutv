use crate::error::Result;
use crate::models::{ApiResult, PlayInfo};

#[async_trait::async_trait]
pub trait Spider: Send + Sync {
    async fn home(&self) -> Result<ApiResult>;
    async fn category(&self, tid: &str, pg: i32, filter: bool, extend: &str) -> Result<ApiResult>;
    async fn detail(&self, ids: &[String]) -> Result<ApiResult>;
    async fn play(&self, flag: &str, id: &str) -> Result<PlayInfo>;
    async fn search(&self, keyword: &str, pg: i32) -> Result<ApiResult>;
}

#[async_trait::async_trait]
pub trait Player: Send + Sync {
    async fn play(&self, info: &PlayInfo) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    fn is_running(&self) -> bool;
}
