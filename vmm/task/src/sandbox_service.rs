/*
Copyright 2022 The Kuasar Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::sync::Arc;

use async_trait::async_trait;
use containerd_shim::{error::Result, TtrpcContext, TtrpcResult};
use tokio::sync::Mutex;
use vmm_common::{
    api,
    api::{empty::Empty, sandbox::*},
};

use crate::netlink::Handle;

pub struct SandboxService {
    pub handle: Arc<Mutex<Handle>>,
}

impl SandboxService {
    pub fn new() -> Result<Self> {
        let handle = Handle::new()?;
        Ok(Self {
            handle: Arc::new(Mutex::new(handle)),
        })
    }

    pub(crate) async fn handle_localhost(&self) -> Result<()> {
        self.handle.lock().await.enable_lo().await
    }
}

#[async_trait]
impl api::sandbox_ttrpc::SandboxService for SandboxService {
    async fn update_interfaces(
        &self,
        _ctx: &TtrpcContext,
        req: UpdateInterfacesRequest,
    ) -> TtrpcResult<Empty> {
        self.handle
            .lock()
            .await
            .update_interfaces(req.interfaces)
            .await?;
        Ok(Empty::new())
    }

    async fn update_routes(
        &self,
        _ctx: &TtrpcContext,
        req: UpdateRoutesRequest,
    ) -> TtrpcResult<Empty> {
        self.handle.lock().await.update_routes(req.routes).await?;
        Ok(Empty::new())
    }

    async fn check(&self, _ctx: &TtrpcContext, _req: CheckRequest) -> TtrpcResult<Empty> {
        Ok(Empty::new())
    }

    async fn exec_vm_process(
        &self,
        _ctx: &TtrpcContext,
        _req: ExecVMProcessRequest,
    ) -> TtrpcResult<ExecVMProcessResponse> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(
            ::ttrpc::Code::NOT_FOUND,
            "/grpc.SandboxService/ExecVMProcess is not supported".to_string(),
        )))
    }

    async fn sync_clock(
        &self,
        _ctx: &TtrpcContext,
        _req: SyncClockPacket,
    ) -> TtrpcResult<SyncClockPacket> {
        Err(::ttrpc::Error::RpcStatus(::ttrpc::get_status(
            ::ttrpc::Code::NOT_FOUND,
            "/grpc.SandboxService/SyncClock is not supported".to_string(),
        )))
    }
}
