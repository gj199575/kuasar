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

use std::os::unix::io::RawFd;

use sandbox_derive::CmdLineParams;

use crate::{device::Transport, network::NetType};

pub const VIRTIO_NET_DRIVER: &str = "virtio-net";

#[derive(CmdLineParams, Debug, Clone)]
#[params("netdev", "device")]
pub struct VirtioNetDevice {
    #[property(param = "netdev", ignore_key)]
    pub(crate) r#type: NetType,
    #[property(ignore)]
    pub(crate) transport: Transport,
    #[property(param = "device", ignore_key)]
    pub(crate) driver: String,
    #[property(param = "netdev")]
    #[property(param = "device", key = "netdev")]
    pub(crate) id: String,
    #[property(param = "device", key = "id")]
    pub(crate) device_id: String,
    #[property(
        param = "netdev",
        predicate = "self.vhost",
        generator = "crate::utils::bool_to_on_off"
    )]
    pub(crate) vhost: bool,
    #[property(
        param = "netdev",
        predicate = "self.vhost",
        generator = "crate::utils::vec_to_string"
    )]
    pub(crate) vhostfds: Vec<i32>,
    #[property(
        param = "netdev",
        predicate = "self.fds.len()>0",
        generator = "crate::utils::vec_to_string"
    )]
    #[property(
        param = "device",
        key = "vectors",
        predicate = "self.fds.len()>0 && self.is_pci()",
        generator = "crate::utils::fds_to_vectors"
    )]
    pub(crate) fds: Vec<i32>,
    #[property(param = "netdev", predicate = "self.fds.len()<=0")]
    pub(crate) ifname: Option<String>,
    #[property(param = "device", key = "bus")]
    pub(crate) pci_bus: Option<String>,
    #[property(param = "device", key = "addr")]
    pub(crate) addr: String,
    #[property(param = "netdev")]
    pub(crate) script: Option<String>,
    #[property(param = "netdev", key = "downscript")]
    pub(crate) down_script: Option<String>,
    #[property(param = "device", key = "mac")]
    pub(crate) mac_address: String,
    #[property(
        param = "device",
        key = "mq",
        generator = "crate::utils::bool_to_on_off"
    )]
    pub(crate) multi_queue: bool,
    #[property(param = "device")]
    pub(crate) disable_modern: Option<bool>,
    #[property(param = "device")]
    pub(crate) romfile: Option<String>,
    #[property(param = "netdev")]
    pub(crate) queues: Option<i32>,
}

impl VirtioNetDevice {
    fn is_pci(&self) -> bool {
        matches!(self.transport, Transport::Pci)
    }
}

impl_device_no_bus!(VirtioNetDevice);
impl_set_get_device_addr!(VirtioNetDevice);

impl VirtioNetDevice {
    pub fn new(
        id: &str,
        name: Option<String>,
        mac_address: &str,
        transport: Transport,
        fds: Vec<RawFd>,
        enable_vhost: bool,
        vhostfds: Vec<RawFd>,
        bus: Option<String>,
    ) -> Self {
        let driver = transport.to_driver(VIRTIO_NET_DRIVER);
        let multi_queue = !fds.is_empty();
        Self {
            r#type: NetType::Tap,
            transport,
            driver,
            id: id.to_string(),
            device_id: format!("virtio-net-{}", id),
            vhost: enable_vhost,
            vhostfds,
            fds,
            ifname: name,
            pci_bus: bus,
            addr: "".to_string(),
            script: None,
            down_script: None,
            mac_address: mac_address.to_string(),
            multi_queue,
            disable_modern: None,
            romfile: None,
            queues: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        network::NetType,
        param::ToCmdLineParams,
        stratovirt::{
            devices::{virtio_net::VirtioNetDevice, DEFAULT_PCIE_BUS},
            Transport,
        },
    };

    #[test]
    fn test_virtio_net_params_disable_vhost() {
        let device = VirtioNetDevice {
            r#type: NetType::Tap,
            transport: Transport::Pci,
            driver: "virtio-net-pci".to_string(),
            id: "intf-tap0".to_string(),
            device_id: "virtio-net-intf-tap0".to_string(),
            vhost: false,
            vhostfds: vec![],
            fds: vec![],
            ifname: Some("tap0".to_string()),
            pci_bus: Some(DEFAULT_PCIE_BUS.to_string()),
            addr: "0x0f".to_string(),
            script: None,
            down_script: None,
            mac_address: "a1:b2:c3:d5:f4".to_string(),
            multi_queue: true,
            disable_modern: None,
            romfile: None,
            queues: None,
        };
        let params = device.to_cmdline_params("-");

        println!("params: {:?}", params);

        assert!(params
            .iter()
            .position(|x| x == "tap,id=intf-tap0,ifname=tap0")
            .is_some());
        assert!(params
            .iter()
            .position(|x| x == "virtio-net-pci,netdev=intf-tap0,id=virtio-net-intf-tap0,bus=pcie.0,addr=0x0f,mac=a1:b2:c3:d5:f4,mq=on")
            .is_some());
    }
}
