// The MIT License (MIT)
//
// Copyright (c) 2018 AT&T. All other rights reserved.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use hw::HWManager;
use l2::L2Layer;
use label::LabelingManager;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug)]
pub enum DivertType {
    IPSrc,
    IPDest,
}

#[derive(Clone, RustcEncodable)]
#[allow(non_snake_case)]
pub struct Connection {
    pub From: u32,
    pub To: u32,
    pub Type: String,
}

pub struct L2Manager {
    session_handler: u32,
}

lazy_static! {
    static ref MANAGER: Mutex<L2Manager> = Mutex::new(L2Manager{session_handler: 0});
}

impl L2Manager {
    pub fn init() {
        MANAGER.lock().unwrap().session_handler = L2Layer::init();
    }

    pub fn configure_from_port_to_port_forwarding(dev_port_1: u32, dev_port_2: u32) {
        L2Layer::configure_from_port_to_port(MANAGER.lock().unwrap().session_handler, dev_port_1 as u16, dev_port_2 as u16);
    }

    pub fn configure_port_to_port_forwarding(dev_port_1: u32, dev_port_2: u32) {
        L2Layer::configure_from_port_to_port(MANAGER.lock().unwrap().session_handler, dev_port_1 as u16, dev_port_2 as u16);
        L2Layer::configure_from_port_to_port(MANAGER.lock().unwrap().session_handler, dev_port_2 as u16, dev_port_1 as u16);
    }

    pub fn set_divert(
        divert_type: DivertType,
        chassis_port_ingress: u32,
        chassis_port_egress: u32,
        ip_address: &String,
        ip_prefix_length: u16,
        high_priority: bool,
    ) {
        let dev_port_ingress = HWManager::convert_chassis_port_to_dev_port(&chassis_port_ingress);
        let dev_port_egress = HWManager::convert_chassis_port_to_dev_port(&chassis_port_egress);
        let ip_address_as_int = convert_ip_address_to_int(ip_address);
        let mask = convert_prefix_to_mask(ip_prefix_length as u32);

        println!("Detour {:?} {} {} {} {}", divert_type, dev_port_ingress, dev_port_egress, ip_address_as_int, mask);

        L2Layer::set_divert(
            MANAGER.lock().unwrap().session_handler,
            divert_type.clone(),
            dev_port_ingress as u16,
            dev_port_egress as u16,
            ip_address_as_int,
            mask,
            high_priority,
        );

        LabelingManager::label_divert(
            format!{"{:?}", divert_type},
            chassis_port_ingress,
            chassis_port_egress,
            format!{"{}/{}", ip_address, ip_prefix_length},
        );
    }

    pub fn reset_divert_table() {
        L2Layer::reset_divert_table(MANAGER.lock().unwrap().session_handler);
        LabelingManager::label_reset();
    }

    pub fn reset_divert_for_ingress_egress_port(chassis_port_ingress: u32, chassis_port_egress: u32) {
        let dev_port_ingress = HWManager::convert_chassis_port_to_dev_port(&chassis_port_ingress);
        let dev_port_egress = HWManager::convert_chassis_port_to_dev_port(&chassis_port_egress);

        L2Layer::reset_divert_for_ingress_egress_port(
            MANAGER.lock().unwrap().session_handler,
            dev_port_ingress as u16,
            dev_port_egress as u16,
        );

        LabelingManager::label_reset_ingress_egress(chassis_port_ingress, chassis_port_egress);
    }
}

fn convert_prefix_to_mask(prefix: u32) -> u32 {
    if prefix == 32 {
        u32::max_value()
    } else {
        (2u32.pow(prefix) - 1 << (32 - prefix))
    }
}

fn convert_ip_address_to_int(ip_address: &String) -> u32 {
    let addr = Ipv4Addr::from_str(ip_address).unwrap();
    let octets = addr.octets();
    let (a, b, c, d) = (octets[0] as u32, octets[1] as u32, octets[2] as u32, octets[3] as u32);

    (a << 24) | (b << 16) | (c << 8) | d
}
