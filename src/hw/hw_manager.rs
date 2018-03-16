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

use bf::BFLayer;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, RustcEncodable)]
#[allow(non_snake_case)]
pub struct Port {
    pub Number: u32,
    pub Speed: u16,
    pub AutoNegDisabled: bool,
    pub FECDisabled: bool,
}

pub struct HWManager {
    configured_ports: Vec<u32>,
    dev_to_chassis_port_map: HashMap<u32, u32>,
}

#[derive(Clone, Debug)]
pub struct PortStats {
    pub packets_in: u64,
    pub packets_out: u64,
    pub octets_in: u64,
    pub octets_out: u64,
    pub packets_dropped_buffer_full: u64,
}

lazy_static! {
    static ref MANAGER: Mutex<HWManager> = Mutex::new(
        HWManager{configured_ports: Vec::with_capacity(1024), dev_to_chassis_port_map: HashMap::new()});
}

impl HWManager {
    pub fn init() {
        BFLayer::init_device();
    }

    pub fn configure_ports(dev_port_definitions: &Vec<Port>) {
        let mut dev_ports = Vec::new();
        for dev_port in dev_port_definitions {
            dev_ports.push(dev_port.Number as u32);
        }
        MANAGER.lock().unwrap().configured_ports = dev_ports.clone();

        for dev_port in dev_port_definitions {
            BFLayer::configure_port(dev_port.Number, dev_port.Speed, dev_port.AutoNegDisabled, dev_port.FECDisabled);
        }
    }

    pub fn get_configured_dev_ports() -> Vec<u32> {
        MANAGER.lock().unwrap().configured_ports.clone()
    }

    pub fn convert_chassis_port_to_dev_port(chassis_port: &u32) -> u32 {
        let dev_port = BFLayer::convert_chassis_port_to_dev_port(chassis_port.clone());
        MANAGER.lock().unwrap().dev_to_chassis_port_map.insert(dev_port, chassis_port.clone());
        dev_port
    }

    pub fn convert_dev_port_to_chassis_port(dev_port: &u32) -> u32 {
        match MANAGER.lock().unwrap().dev_to_chassis_port_map.get(&dev_port) {
            Some(chassis_port) => chassis_port.clone(),
            None => 0,
        }
    }

    pub fn get_stats_for_port(dev_port: u32) -> PortStats {
        let result = PortStats {
            packets_in: BFLayer::get_stat_packets_in(dev_port as i32),
            packets_out: BFLayer::get_stat_packets_out(dev_port as i32),
            octets_in: BFLayer::get_stat_octets_in(dev_port as i32),
            octets_out: BFLayer::get_stat_octets_out(dev_port as i32),
            packets_dropped_buffer_full: BFLayer::get_stat_packets_dropped_buffer_full(dev_port as i32),
        };

        result
    }
}
