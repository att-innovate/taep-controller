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

pub mod HHDLayer {
    include!("../../gen-stub/bindings-taep.rs");

    use std::mem;

    pub fn init() -> u32 {
        unsafe {
            let session_handler: *mut p4_pd_sess_hdl_t = malloc(mem::size_of::<p4_pd_sess_hdl_t>()) as *mut p4_pd_sess_hdl_t;
            let _status = p4_pd_client_init(session_handler);

            *session_handler
        }
    }

    pub fn set_hhd(session_handler: u32, dev_port_ingress: u16) {
        unsafe {
            let match_spec: *mut p4_pd_l2_switching_feature_match_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_feature_match_spec_t>()) as *mut p4_pd_l2_switching_feature_match_spec_t;
            let action_spec: *mut p4_pd_l2_switching_feature_enable_action_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_feature_enable_action_spec_t>())
                    as *mut p4_pd_l2_switching_feature_enable_action_spec_t;
            let entry_hdl: *mut p4_pd_entry_hdl_t = malloc(mem::size_of::<p4_pd_entry_hdl_t>()) as *mut p4_pd_entry_hdl_t;

            (*match_spec).ig_intr_md_ingress_port = dev_port_ingress;
            (*action_spec).action_hhd = 1;
            (*action_spec).action_flows = 0;

            p4_pd_l2_switching_feature_table_add_with_feature_enable(
                session_handler,
                resolve_dev_target(),
                match_spec,
                action_spec,
                entry_hdl,
            );
            println!("Enabled HHD {}", *entry_hdl);
        }
    }

    pub fn reset_hhd_table(session_handler: u32) {
        unsafe {
            let mut status: p4_pd_status_t = 0;
            while status == 0 {
                let index: *mut i32 = malloc(mem::size_of::<i32>()) as *mut i32;
                status = p4_pd_l2_switching_feature_get_first_entry_handle(session_handler, resolve_dev_target(), index);

                // todo: we will have to verify that it is the HHD entry
                // assuming that there will be more features
                if status == 0 {
                    p4_pd_l2_switching_feature_table_delete(session_handler, 0 as u8, *index as p4_pd_entry_hdl_t);
                    println!("Delete HHD Setting, Handle {}", *index);
                }
            }
        }
    }

    pub fn retrieve_current_packet_count(session_handler: u32, hashes: &Vec<u16>) -> u64 {
        let mut result: u64;

        unsafe {
            let counter_value: *mut p4_pd_counter_value_t = malloc(mem::size_of::<p4_pd_counter_value_t>()) as *mut p4_pd_counter_value_t;

            p4_pd_l2_switching_counter_read_count_hhd_hash_1(
                session_handler,
                resolve_dev_target(),
                hashes[0] as i32,
                COUNTER_READ_HW_SYNC as i32,
                counter_value,
            );
            result = (*counter_value).packets;

            p4_pd_l2_switching_counter_read_count_hhd_hash_2(
                session_handler,
                resolve_dev_target(),
                hashes[1] as i32,
                COUNTER_READ_HW_SYNC as i32,
                counter_value,
            );
            result = result + (*counter_value).packets;
        }

        result
    }

    pub fn retrieve_smallest_packet_count(session_handler: u32, hashes: &Vec<u16>) -> u64 {
        let mut result: u64;

        unsafe {
            let counter_value: *mut p4_pd_counter_value_t = malloc(mem::size_of::<p4_pd_counter_value_t>()) as *mut p4_pd_counter_value_t;

            p4_pd_l2_switching_counter_read_count_hhd_hash_1(
                session_handler,
                resolve_dev_target(),
                hashes[0] as i32,
                COUNTER_READ_HW_SYNC as i32,
                counter_value,
            );
            result = (*counter_value).packets;

            p4_pd_l2_switching_counter_read_count_hhd_hash_2(
                session_handler,
                resolve_dev_target(),
                hashes[1] as i32,
                COUNTER_READ_HW_SYNC as i32,
                counter_value,
            );
            if (*counter_value).packets < result {
                result = (*counter_value).packets;
            }
        }

        result
    }

    pub fn reset_counters(session_handler: u32, hashes: &Vec<u16>) {
        unsafe {
            p4_pd_l2_switching_counter_write_count_hhd_hash_1(
                session_handler,
                resolve_dev_target(),
                hashes[0] as i32,
                p4_pd_counter_value {
                    packets: 0 as u64,
                    bytes: 0 as u64,
                },
            );

            p4_pd_l2_switching_counter_write_count_hhd_hash_2(
                session_handler,
                resolve_dev_target(),
                hashes[1] as i32,
                p4_pd_counter_value {
                    packets: 0 as u64,
                    bytes: 0 as u64,
                },
            );
        }
    }

    fn resolve_dev_target() -> p4_pd_dev_target_t {
        p4_pd_dev_target_t {
            device_id: 0 as i32,
            dev_pipe_id: DEV_PIPE_ALL as u16,
        }
    }
}
