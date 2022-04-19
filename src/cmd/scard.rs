use crate::cmd::{Parse};
use crate::tikv::errors::AsyncResult;
use crate::tikv::set::SetCommandCtx;
use crate::{Connection, Frame};
use crate::config::{is_use_txn_api};
use crate::utils::{resp_err};

use tracing::{debug, instrument};

#[derive(Debug)]
pub struct Scard {
    key: String,
}

impl Scard {
    pub fn new(key: &str) -> Scard {
        Scard {
            key: key.to_string(),
        }
    }

    /// Get the key
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn set_key(&mut self, key: &str) {
        self.key = key.to_owned();
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Scard> {
        let key = parse.next_string()?;
        Ok(Scard::new(&key))
    }

    #[instrument(skip(self, dst))]
    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        
        let response = self.scard().await?;
        debug!(?response);
        dst.write_frame(&response).await?;

        Ok(())
    }

    async fn scard(&self) -> AsyncResult<Frame> {
        if is_use_txn_api() {
            SetCommandCtx::new(None).do_async_txnkv_scard(&self.key).await
        } else {
            Ok(resp_err("not supported yet"))
        }
    }
}
