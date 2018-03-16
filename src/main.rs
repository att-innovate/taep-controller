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

extern crate taep_controller;
extern crate yaml_rust;

use std::env;
use std::fs::File;
use std::io::Read;
use std::sync::{Condvar, Mutex};
use taep_controller::api::APIManager;
use taep_controller::bf::BFManager;
use taep_controller::flows::FlowsManager;
use taep_controller::hhd::HHDManager;
use taep_controller::hw::{HWManager, Port};
use taep_controller::l2::{Connection, L2Manager};
use taep_controller::label::LabelingManager;
use taep_controller::metrics::MetricsCollector;
use yaml_rust::{Yaml, YamlLoader};

fn main() {
    let mut config_file = "./config/config.yml".to_string();

    let args: Vec<_> = env::args().collect();
    if args.len() == 2 {
        config_file = args[1].clone();
    }

    println!("Configuration read from: {}", config_file);
    let config = read_config_file(&config_file);

    let bf_config_file = read_bf_config_file(&config);
    let bf_bin_path = read_bf_bin_path(&config);
    let ports = read_ports(&config);
    let connections = read_connections(&config);
    let enable_labeling = read_enable_labeling(&config);
    let api_port = read_api_port(&config);

    println!("bf config file: {}", bf_config_file);
    println!("bf bin path: {}", bf_bin_path);

    BFManager::run(bf_config_file, bf_bin_path);

    APIManager::run(api_port);
    LabelingManager::run(enable_labeling);
    LabelingManager::label_reset();

    HWManager::init();

    let mut dev_ports = Vec::new();
    for port in ports {
        let definition = Port {
            Number: HWManager::convert_chassis_port_to_dev_port(&port.Number),
            Speed: port.Speed,
            AutoNegDisabled: port.AutoNegDisabled,
            FECDisabled: port.FECDisabled,
        };
        dev_ports.push(definition);
    }

    HWManager::configure_ports(&dev_ports);

    for connection in connections {
        if connection.Type == "unidirectional" {
            L2Manager::configure_from_port_to_port_forwarding(
                HWManager::convert_chassis_port_to_dev_port(&connection.From),
                HWManager::convert_chassis_port_to_dev_port(&connection.To),
            );
        } else {
            L2Manager::configure_port_to_port_forwarding(
                HWManager::convert_chassis_port_to_dev_port(&connection.From),
                HWManager::convert_chassis_port_to_dev_port(&connection.To),
            );
        }
    }

    HHDManager::init(read_max_number_of_flows(&config), read_analysis_window_in_seconds(&config));

    FlowsManager::init();

    let poll_interval_in_seconds: u16 = 5;
    MetricsCollector::run(poll_interval_in_seconds);

    // wait forever
    let condvar = Condvar::new();
    let lock = Mutex::new(());
    let _ = condvar.wait(lock.lock().unwrap());
}

fn read_config_file(config_file: &str) -> Yaml {
    let mut file = match File::open(config_file) {
        Ok(file) => file,
        Err(err) => panic!(err.to_string()),
    };

    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let config = YamlLoader::load_from_str(&content).unwrap();
    // Multi document support, doc is a yaml::Yaml
    config[0].clone()
}

fn read_ports(config: &Yaml) -> Vec<Port> {
    let mut result = Vec::new();

    match config["ports"].is_badvalue() {
        true => {}
        false => {
            let ports = config["ports"].as_vec().unwrap();
            for port in ports {
                let speed = port["speed"].as_i64().unwrap();
                let definition = Port {
                    Number: port["number"].as_i64().unwrap() as u32,
                    Speed: speed as u16,
                    AutoNegDisabled: match port["autoneg-disabled"].is_badvalue() {
                        true => false,
                        false => port["autoneg-disabled"].as_bool().unwrap(),
                    },
                    FECDisabled: match port["fec-disabled"].is_badvalue() {
                        true => match speed {
                            100 => false,
                            _ => true,
                        },
                        false => port["fec-disabled"].as_bool().unwrap(),
                    },
                };
                result.push(definition);
            }
        }
    }

    result.clone()
}

fn read_connections(config: &Yaml) -> Vec<Connection> {
    let mut result = Vec::new();

    match config["connections"].is_badvalue() {
        true => {}
        false => {
            let connections = config["connections"].as_vec().unwrap();
            for connection in connections {
                let definition = Connection {
                    From: connection["from"].as_i64().unwrap() as u32,
                    To: connection["to"].as_i64().unwrap() as u32,
                    Type: connection["type"].as_str().unwrap().to_string(),
                };
                result.push(definition);
            }
        }
    }

    result.clone()
}

fn read_bf_bin_path(config: &Yaml) -> String {
    match config["bf-bin-path"].is_badvalue() {
        true => "/root/bf-sde/install".to_string(),
        false => config["bf-bin-path"].as_str().unwrap().to_string(),
    }
}

fn read_bf_config_file(config: &Yaml) -> String {
    match config["bf-config-file"].is_badvalue() {
        true => "/root/taep_controller/p4/l2_switching.conf".to_string(),
        false => config["bf-config-file"].as_str().unwrap().to_string(),
    }
}

fn read_enable_labeling(config: &Yaml) -> bool {
    match config["enable-labeling"].is_badvalue() {
        true => false,
        false => config["enable-labeling"].as_bool().unwrap(),
    }
}

fn read_api_port(config: &Yaml) -> u16 {
    (match config["api_port"].is_badvalue() {
        true => 8100,
        false => config["api_port"].as_i64().unwrap(),
    }) as u16
}

fn read_analysis_window_in_seconds(config: &Yaml) -> u16 {
    (match config["hhd"].is_badvalue() {
        true => 30,
        false => config["hhd"]["analysis-window-in-seconds"].as_i64().unwrap(),
    }) as u16
}

fn read_max_number_of_flows(config: &Yaml) -> u16 {
    (match config["hhd"].is_badvalue() {
        true => 100,
        false => match config["hhd"]["max-number-of-flows"].is_badvalue() {
            true => 100,
            false => config["hhd"]["max-number-of-flows"].as_i64().unwrap(),
        },
    }) as u16
}
