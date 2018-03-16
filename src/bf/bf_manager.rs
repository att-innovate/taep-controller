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
use std::sync::Mutex;

lazy_static! {
    static ref MANAGER: Mutex<BFManager> = Mutex::new(BFManager{is_running: false});
}

pub struct BFManager {
    is_running: bool,
}

impl BFManager {
    pub fn run(bf_config_file: String, bf_bin_path: String) {
        init_bf(bf_config_file, bf_bin_path);
    }
}

fn init_bf(bf_config_file: String, bf_bin_path: String) {
    BFLayer::init_bf(bf_config_file, bf_bin_path);
    MANAGER.lock().unwrap().is_running = true;
}
