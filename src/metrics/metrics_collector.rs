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
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::u16;

pub struct MetricsCollector {
    poll_interval_in_seconds: u16,
    recent_metrics: HashMap<u32, Metrics>,
}

#[derive(Clone, Debug, RustcEncodable)]
pub struct Metrics {
    pub chassis_port: u32,
    pub packets_in: u64,
    pub packets_out: u64,
    pub octets_in: u64,
    pub octets_out: u64,
    pub packets_dropped_buffer_full: u64,
}

lazy_static! {
    static ref COLLECTOR: Mutex<MetricsCollector> =
        Mutex::new(MetricsCollector{poll_interval_in_seconds: u16::MAX, recent_metrics: HashMap::new()});
}

impl MetricsCollector {
    pub fn run(poll_interval_in_seconds: u16) {
        COLLECTOR.lock().unwrap().poll_interval_in_seconds = poll_interval_in_seconds;

        let _ = thread::Builder::new().name("metrics-collector".to_string()).spawn(move || loop {
            collect_port_stats();
            thread::sleep(Duration::from_secs(poll_interval_in_seconds as u64));
        });
    }

    pub fn get_port_stats() -> Vec<Metrics> {
        let recent_metrics = COLLECTOR.lock().unwrap().recent_metrics.clone();
        let mut result: Vec<Metrics> = Vec::with_capacity(recent_metrics.len());
        for metrics in recent_metrics.values() {
            result.push(metrics.clone());
        }
        result
    }
}

fn collect_port_stats() {
    let configured_ports = HWManager::get_configured_dev_ports();
    for port in configured_ports {
        let chassis_port = HWManager::convert_dev_port_to_chassis_port(&port);
        let stats = HWManager::get_stats_for_port(port);

        let metrics = Metrics {
            chassis_port: chassis_port,
            packets_in: stats.packets_in,
            packets_out: stats.packets_out,
            octets_in: stats.octets_in,
            octets_out: stats.octets_out,
            packets_dropped_buffer_full: stats.packets_dropped_buffer_full,
        };

        COLLECTOR.lock().unwrap().recent_metrics.insert(chassis_port, metrics);
    }
}
