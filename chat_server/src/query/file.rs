use async_graphql::Object;
use anyhow::Result;

use crate::error::AppError;

#[derive(Default)]
pub(crate) struct FileQuery;

#[Object]
impl FileQuery {
    async fn get_sts(&self) -> Result<String, AppError> {
        Ok("STS".to_string())
    }
}
