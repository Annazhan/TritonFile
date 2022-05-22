pub mod client_fs;
use crate::client_fs::binstore::BinStore;
pub type TribResult<T> = Result<T, Box<(dyn Error + Send + Sync)>>;


#[allow(unused_variables)]
pub async fn new_bin_client(backs: Vec<String>) -> TribResult<Box<dyn BinStorage>> {
    Ok(Box::new(binstore::BinStore::new(backs)))
}

