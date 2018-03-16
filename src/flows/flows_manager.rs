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

use flows::FlowsLayer;
use hw::HWManager;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq, RustcEncodable)]
pub struct Flow {
    pub src_addr: String,
    pub src_addr_int: u32,
    pub src_port: u16,
    pub dst_addr: String,
    pub dst_addr_int: u32,
    pub dst_port: u16,
    pub ipv4_protocol: u8,
    pub hash1: u16,
    pub hash2: u16,
}

pub struct FlowsManager {
    session_handler: u32,
    flowmanager_running: bool,
    max_number_of_flows: u16,
    current_number_of_flows: u16,
    learned_flows: Vec<Flow>,
}

lazy_static! {
    static ref MANAGER: Mutex<FlowsManager> = Mutex::new(
        FlowsManager{
            session_handler: 0,
            flowmanager_running: false,
            max_number_of_flows: 100,
            current_number_of_flows: 0,
            learned_flows: Vec::new(),
        });
}

impl FlowsManager {
    pub fn init() {
        let mut manager = MANAGER.lock().unwrap();
        manager.session_handler = FlowsLayer::init();
        FlowsLayer::setup_tables(manager.session_handler);
        FlowsLayer::register_callback_function(manager.session_handler);
    }

    pub fn set_flow_learning_for_time_window(chassis_port_ingress: u32, max_number_of_flows: u32, time_window_in_seconds: u32) {
        let mut manager = MANAGER.lock().unwrap();
        if manager.flowmanager_running {
            return;
        };

        manager.flowmanager_running = true;
        manager.learned_flows = Vec::with_capacity(max_number_of_flows as usize);
        manager.max_number_of_flows = max_number_of_flows as u16;

        let _ = thread::Builder::new().name("flows_learner".to_string()).spawn(move || {
            thread::sleep(Duration::from_secs(time_window_in_seconds as u64));
            FlowsManager::reset_flows_table();
            FlowsManager::reset_learned_flows();
            MANAGER.lock().unwrap().flowmanager_running = false;
            println!("Flows Learning turned off");
        });

        let dev_port_ingress = HWManager::convert_chassis_port_to_dev_port(&chassis_port_ingress);
        FlowsLayer::set_flows(manager.session_handler, dev_port_ingress as u16);

        println!("Flows Learning turned on on {}", chassis_port_ingress);
    }

    pub fn start_flow_learning(chassis_port_ingress: u32, max_number_of_flows: u16) {
        let mut manager = MANAGER.lock().unwrap();
        if !manager.flowmanager_running {
            manager.flowmanager_running = true;
            manager.learned_flows = Vec::with_capacity(max_number_of_flows as usize);
            manager.max_number_of_flows = max_number_of_flows;
        };

        let dev_port_ingress = HWManager::convert_chassis_port_to_dev_port(&chassis_port_ingress);
        FlowsLayer::set_flows(manager.session_handler, dev_port_ingress as u16);

        println!("Flows Learning turned on on {}", chassis_port_ingress);
    }

    pub fn stop_flow_learning() {
        FlowsManager::reset_flows_table();
        FlowsManager::reset_learned_flows();
        MANAGER.lock().unwrap().flowmanager_running = false;
        println!("Flows Learning turned off");
    }

    pub fn get_learned_flows() -> Vec<Flow> {
        MANAGER.lock().unwrap().learned_flows.clone()
    }

    pub fn get_learned_flows_and_reset() -> Vec<Flow> {
        let mut manager = MANAGER.lock().unwrap();
        let learned_flows = manager.learned_flows.clone();
        manager.learned_flows = Vec::with_capacity(manager.max_number_of_flows as usize);
        manager.current_number_of_flows = 0;
        for flow_to_delete in learned_flows.clone() {
            FlowsLayer::reset_bloomfilters(manager.session_handler, &vec![flow_to_delete.hash1, flow_to_delete.hash2]);
        }
        learned_flows
    }

    pub fn get_current_number_of_flows() -> u16 {
        MANAGER.lock().unwrap().current_number_of_flows
    }

    pub fn reset_learned_flows() {
        let mut manager = MANAGER.lock().unwrap();
        for flow_to_delete in manager.learned_flows.clone() {
            FlowsLayer::reset_bloomfilters(manager.session_handler, &vec![flow_to_delete.hash1, flow_to_delete.hash2]);
        }
        manager.current_number_of_flows = 0;
    }

    pub fn add_learned_flow(flow: Flow) {
        let mut manager = MANAGER.lock().unwrap();
        manager.current_number_of_flows = manager.current_number_of_flows + 1;
        if manager.current_number_of_flows <= manager.max_number_of_flows {
            manager.learned_flows.push(flow.clone());
        } else {
            println!("Dropped learned flow, number is limited to {}", manager.max_number_of_flows);
        }
    }

    pub fn reset_flows_table() {
        FlowsLayer::reset_flows_table(MANAGER.lock().unwrap().session_handler);
    }
}
