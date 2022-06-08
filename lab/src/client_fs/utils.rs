use std::thread::JoinHandle;

use tribbler::{error::TritonFileResult, storage::Storage};

use super::ops::FileOp;

pub async fn write_file_log(storage: Box<dyn Storage>, val: String, op: FileOp) -> TritonFileResult<()>{
    

    return Ok(())
}
