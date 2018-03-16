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

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod BFLayer {
    include!("../../gen-stub/bindings-bfswitch.rs");

    use std::ffi::CString;
    use std::mem;

    pub fn init_bf(bf_config_file: String, bf_bin_path: String) {
        unsafe {
            let bf_switchd_context: *mut bf_switchd_context_t = malloc(mem::size_of::<bf_switchd_context_t>()) as *mut bf_switchd_context_t;
            let install_dir = CString::new(bf_bin_path).unwrap().into_raw();
            let conf_file = CString::new(bf_config_file).unwrap().into_raw();

            (*bf_switchd_context).install_dir = install_dir;
            (*bf_switchd_context).conf_file = conf_file;
            (*bf_switchd_context).running_in_background = true;
            (*bf_switchd_context).init_mode = bf_dev_init_mode_s::BF_DEV_INIT_COLD;

            bf_switchd_lib_init(bf_switchd_context);
            println!("bf_switchd init done");
        }
    }

    pub fn init_device() {
        unsafe {
            let poll_intervall_ms: u32 = 5000;
            let status = bf_pal_port_stats_poll_intvl_set(0, poll_intervall_ms);
            println!("stats polling interval set status {}", status);
        }
    }

    pub fn configure_port(port_number: u32, speed: u16, autoneg_disabled: bool, fec_disabled: bool) {
        unsafe {
            let mut status = bf_pal_port_add(
                0,
                port_number as i32,
                match speed {
                    10 => bf_port_speed_e::BF_SPEED_10G,
                    40 => bf_port_speed_e::BF_SPEED_40G,
                    100 => bf_port_speed_e::BF_SPEED_100G,
                    _ => {
                        println!("Port Speed not recognized: {} .. set to 100G ", speed);
                        bf_port_speed_e::BF_SPEED_100G
                    }
                },
                if fec_disabled {
                    bf_fec_type_e::BF_FEC_TYP_NONE
                } else {
                    bf_fec_type_e::BF_FEC_TYP_REED_SOLOMON
                },
            );

            println!("Port: {} added with fec disabled {}, with status: {}", port_number as i32, fec_disabled, status);

            if autoneg_disabled {
                status = bf_pal_port_autoneg_policy_set(0, port_number as i32, 2);
                println!("autoneg_policy disabled, status {}", status);
            } else {
                status = bf_pal_port_autoneg_policy_set(0, port_number as i32, 1);
                println!("autoneg_policy enabled, status {}", status);
            }

            status = bf_pal_port_enable(0, port_number as i32);
            println!("port enabled status {}", status);
        }
    }

    pub fn convert_chassis_port_to_dev_port(chassis_port: u32) -> u32 {
        let result: u32;
        unsafe {
            let dev_port: *mut bf_dev_port_t = malloc(mem::size_of::<bf_dev_port_t>()) as *mut bf_dev_port_t;

            let _status = bf_pal_fp_idx_to_dev_port_map(0, chassis_port, dev_port);

            result = *dev_port as u32;
        }
        result
    }

    pub fn get_stat_packets_in(dev_port: i32) -> u64 {
        let result: u64;
        unsafe {
            let packets_in: *mut u64 = malloc(mem::size_of::<u64>()) as *mut u64;
            let status = bf_pal_port_this_stat_get(0, dev_port, bf_rmon_counter_t::bf_mac_stat_FramesReceivedAll, packets_in);
            if status != 0 {
                println!("get_stat_octets_in error status {}", status);
            };

            result = *packets_in;
        }
        result
    }

    pub fn get_stat_packets_out(dev_port: i32) -> u64 {
        let result: u64;
        unsafe {
            let packets_out: *mut u64 = malloc(mem::size_of::<u64>()) as *mut u64;
            let status = bf_pal_port_this_stat_get(0, dev_port, bf_rmon_counter_t::bf_mac_stat_FramesTransmittedAll, packets_out);
            if status != 0 {
                println!("get_stat_packets_out error status {}", status);
            };

            result = *packets_out;
        }
        result
    }

    pub fn get_stat_octets_in(dev_port: i32) -> u64 {
        let result: u64;
        unsafe {
            let octets_in: *mut u64 = malloc(mem::size_of::<u64>()) as *mut u64;
            let status = bf_pal_port_this_stat_get(0, dev_port, bf_rmon_counter_t::bf_mac_stat_OctetsReceived, octets_in);
            if status != 0 {
                println!("get_stat_octets_in error status {}", status);
            };

            result = *octets_in;
        }
        result
    }

    pub fn get_stat_octets_out(dev_port: i32) -> u64 {
        let result: u64;
        unsafe {
            let octets_out: *mut u64 = malloc(mem::size_of::<u64>()) as *mut u64;
            let status = bf_pal_port_this_stat_get(0, dev_port, bf_rmon_counter_t::bf_mac_stat_OctetsTransmittedTotal, octets_out);
            if status != 0 {
                println!("get_stat_octets_out error status {}", status);
            };

            result = *octets_out;
        }
        result
    }

    pub fn get_stat_packets_dropped_buffer_full(dev_port: i32) -> u64 {
        let result: u64;
        unsafe {
            let packets_dropped_buffer_full: *mut u64 = malloc(mem::size_of::<u64>()) as *mut u64;
            let status =
                bf_pal_port_this_stat_get(0, dev_port, bf_rmon_counter_t::bf_mac_stat_FramesDroppedBufferFull, packets_dropped_buffer_full);
            if status != 0 {
                println!("get_stat_packets_dropped_buffer_full error status {}", status);
            };

            result = *packets_dropped_buffer_full;
        }
        result
    }
}
