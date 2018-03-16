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

use hyper::Client;
use hyper::header::Connection;
use std::sync::Mutex;
use std::thread;

pub struct LabelingManager {
    labeling_on: bool,
}

lazy_static! {
    static ref MANAGER: Mutex<LabelingManager> = Mutex::new(LabelingManager{labeling_on: false});
}

lazy_static! {
    static ref CLIENT: Client = Client::new();
}

impl LabelingManager {
    pub fn run(labeling_on: bool) {
        MANAGER.lock().unwrap().labeling_on = labeling_on;
        println!("LabelingManager running: {}", labeling_on);
    }

    pub fn label_reset() {
        if MANAGER.lock().unwrap().labeling_on {
            let data = "label,type=reset,ingress=999,egress=999 data=\"reset\"".to_string();
            send_label(data);
        }
    }

    pub fn label_reset_ingress_egress(ingress: u32, egress: u32) {
        if MANAGER.lock().unwrap().labeling_on {
            let data = format!{"label,type=reset,ingress={},egress={} data=\"reset\"", ingress, egress};
            send_label(data);
        }
    }

    pub fn label_divert(divert_type: String, ingress: u32, egress: u32, address: String) {
        if MANAGER.lock().unwrap().labeling_on {
            let data = format!{"label,type=divert,ingress={},egress={},divert-type={} data=\"{}\"", ingress, egress, divert_type, address};
            send_label(data);
        }
    }
}

fn send_label(data: String) {
    let address = format!{"http://localhost:8086/write?db=telegraf"};
    let data_internal = data.clone();

    let _ = thread::Builder::new().name("label reset".to_string()).spawn(move || {
        let _ = CLIENT.post(&address).header(Connection::close()).body(&data_internal).send();
        println!("Labeled: {}", data);
    });
}
