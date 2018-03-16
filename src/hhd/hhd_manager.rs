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

use flows::{Flow, FlowsManager};
use hhd::HHDLayer;
use hw::HWManager;
use l2::{DivertType, L2Manager};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

pub struct HHDManager {
    session_handler: u32,
    max_number_of_flows: u16,
    last_heavy_flow: Option<Flow>,
    divert_on: bool,
    divert_type: DivertType,
    divert_ingress_port: u32,
    divert_egress_port: u32,
}

lazy_static! {
    static ref MANAGER: Mutex<HHDManager> = Mutex::new(
        HHDManager{
            session_handler: 0,
            max_number_of_flows: 100,
            last_heavy_flow: None,
            divert_on: false,
            divert_type: DivertType::IPSrc,
            divert_ingress_port: 999,
            divert_egress_port: 999,
        });
}

impl HHDManager {
    pub fn init(max_number_of_flows: u16, analysis_window_in_seconds: u16) {
        println!("HHD Max Number of Flow set to {}", max_number_of_flows);
        let mut manager = MANAGER.lock().unwrap();
        manager.session_handler = HHDLayer::init();
        manager.max_number_of_flows = max_number_of_flows;

        println!("HHD Analysis Window {}s", analysis_window_in_seconds);
        let _ = thread::Builder::new().name("hhd-picker".to_string()).spawn(move || loop {
            pick_hhd();
            thread::sleep(Duration::from_secs(analysis_window_in_seconds as u64));
        });
    }

    pub fn reset_hhd() {
        HHDManager::reset_hhd_table();
        FlowsManager::stop_flow_learning();
        HHDManager::reset_counters();
    }

    pub fn set_hhd(chassis_port_ingress: u32) {
        let manager = MANAGER.lock().unwrap();

        FlowsManager::start_flow_learning(chassis_port_ingress, manager.max_number_of_flows);

        let dev_port_ingress = HWManager::convert_chassis_port_to_dev_port(&chassis_port_ingress);
        HHDLayer::set_hhd(manager.session_handler, dev_port_ingress as u16);

        println!("HHD turned on on {}", chassis_port_ingress);
    }

    pub fn run_hhd_divert(divert_ingress: u32, divert_egress: u32, divert_type: DivertType) {
        let mut manager = MANAGER.lock().unwrap();
        manager.divert_on = true;
        manager.divert_type = divert_type;
        manager.divert_ingress_port = divert_ingress;
        manager.divert_egress_port = divert_egress;

        println!("HHD auto divert turned on on {} -> {}", divert_ingress, divert_egress);
    }

    pub fn reset_counters() {
        let session_handler = MANAGER.lock().unwrap().session_handler;
        for flow_to_delete in FlowsManager::get_learned_flows() {
            HHDLayer::reset_counters(session_handler, &vec![flow_to_delete.hash1, flow_to_delete.hash2]);
        }

        MANAGER.lock().unwrap().last_heavy_flow = None;
    }

    pub fn reset_hhd_table() {
        let mut manager = MANAGER.lock().unwrap();
        HHDLayer::reset_hhd_table(manager.session_handler);
        if manager.divert_on {
            manager.divert_on = false;
            L2Manager::reset_divert_for_ingress_egress_port(manager.divert_ingress_port, manager.divert_egress_port);
        };
    }
}

fn pick_hhd() {
    if !MANAGER.lock().unwrap().divert_on {
        return;
    };

    let learned_flows = FlowsManager::get_learned_flows();
    let session_handler = MANAGER.lock().unwrap().session_handler;

    let mut largest_result = 0;
    let mut largest_flow: Option<Flow> = None;

    for learned_flow in learned_flows {
        let result = HHDLayer::retrieve_smallest_packet_count(session_handler, &vec![learned_flow.hash1, learned_flow.hash2]);

        if result > largest_result {
            largest_result = result;
            largest_flow = Some(learned_flow.clone());
        }
        // println!("Flow: {:?} packet counts: {}", learned_flow, result);
    }

    if largest_flow.is_some() {
        println!(
            "Number of flows seen: {}, max set to: {}",
            FlowsManager::get_current_number_of_flows(),
            MANAGER.lock().unwrap().max_number_of_flows
        );

        let flow = largest_flow.unwrap();
        println!("Largest flow from {:?} to {:?}, details {:?}", flow.src_addr, flow.dst_addr, flow);

        if MANAGER.lock().unwrap().divert_on {
            let divert_ingress_port = MANAGER.lock().unwrap().divert_ingress_port;
            let divert_egress_port = MANAGER.lock().unwrap().divert_egress_port;
            let divert_type = MANAGER.lock().unwrap().divert_type.clone();
            let last_heavy_flow = MANAGER.lock().unwrap().last_heavy_flow.clone();

            if last_heavy_flow.is_none() || (last_heavy_flow.is_some() && last_heavy_flow.unwrap() != flow) {
                L2Manager::reset_divert_for_ingress_egress_port(divert_ingress_port, divert_egress_port);

                L2Manager::set_divert(
                    divert_type,
                    divert_ingress_port,
                    divert_egress_port,
                    &match divert_type {
                        DivertType::IPDest => flow.dst_addr.clone(),
                        DivertType::IPSrc => flow.src_addr.clone(),
                    },
                    32,
                    true,
                );
            }

            MANAGER.lock().unwrap().last_heavy_flow = Some(flow);
        }
    };

    let learned_flows = FlowsManager::get_learned_flows_and_reset();

    for flow_to_delete in learned_flows {
        HHDLayer::reset_counters(session_handler, &vec![flow_to_delete.hash1, flow_to_delete.hash2]);
    }
}
