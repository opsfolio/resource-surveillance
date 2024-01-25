use std::{collections::HashMap, fmt, sync::Arc};

use async_trait::async_trait;
use pgwire::api::results::FieldInfo;
use tokio::sync::Mutex;

use crate::{error::UdiPgpResult, parser::stmt::UdiPgpStatment, Row};

#[async_trait]
pub trait SqlSupplier: ClonableSqlSupplier {
    fn name(&self) -> &str;
    async fn schema(&mut self, stmt: &mut UdiPgpStatment) -> UdiPgpResult<Vec<FieldInfo>>;
    async fn execute(&mut self, stmt: &UdiPgpStatment) -> UdiPgpResult<Vec<Vec<Row>>>;
}

pub type SqlSupplierType = Box<dyn SqlSupplier + Send + Sync>;
pub type SqlSupplierMap = HashMap<String, Arc<Mutex<SqlSupplierType>>>;

pub trait ClonableSqlSupplier {
    fn clone_box(&self) -> Box<dyn SqlSupplier + Send + Sync>;
}

impl<T> ClonableSqlSupplier for T
where
    T: 'static + SqlSupplier + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn SqlSupplier + Send + Sync> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn SqlSupplier + Send + Sync> {
    fn clone(&self) -> Box<dyn SqlSupplier + Send + Sync> {
        self.clone_box()
    }
}

impl fmt::Debug for dyn SqlSupplier + Send + Sync {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SqlSupplier").field(&self.name()).finish()
    }
}
