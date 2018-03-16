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

pub mod FlowsLayer {
    include!("../../gen-stub/bindings-taep.rs");

    use flows::{Flow, FlowsManager};
    use std::mem;
    use std::net::Ipv4Addr;
    use std::os::raw::c_void;

    pub fn init() -> u32 {
        unsafe {
            let session_handler: *mut p4_pd_sess_hdl_t = malloc(mem::size_of::<p4_pd_sess_hdl_t>()) as *mut p4_pd_sess_hdl_t;
            let _status = p4_pd_client_init(session_handler);

            *session_handler
        }
    }

    pub fn setup_tables(session_handler: u32) {
        unsafe {
            let match_spec: *mut p4_pd_l2_switching_extract_flows_ports_match_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_extract_flows_ports_match_spec_t>())
                    as *mut p4_pd_l2_switching_extract_flows_ports_match_spec_t;
            let entry_hdl: *mut p4_pd_entry_hdl_t = malloc(mem::size_of::<p4_pd_entry_hdl_t>()) as *mut p4_pd_entry_hdl_t;

            (*match_spec).tcp_valid = 1;
            (*match_spec).udp_valid = 0;

            p4_pd_l2_switching_extract_flows_ports_table_add_with_get_flows_tcp_ports(
                session_handler,
                resolve_dev_target(),
                match_spec,
                entry_hdl,
            );

            (*match_spec).tcp_valid = 0;
            (*match_spec).udp_valid = 1;

            p4_pd_l2_switching_extract_flows_ports_table_add_with_get_flows_udp_ports(
                session_handler,
                resolve_dev_target(),
                match_spec,
                entry_hdl,
            );
        }
    }

    pub fn register_callback_function(session_handler: u32) {
        unsafe {
            let cb_fn_cookie: *mut c_void = malloc(mem::size_of::<c_void>()) as *mut c_void;
            p4_pd_l2_switching_ipv4_flows_tuple_plus_hash_register(session_handler, 0 as u8, Some(callback), cb_fn_cookie);

            // receive new flows at latest every 0.5s
            let learning_timeout: u_int32_t = 1 * 500000;
            p4_pd_l2_switching_set_learning_timeout(session_handler, 0 as u8, learning_timeout);

            println!("Callback function for flow learning registered");
        }
    }

    pub fn set_flows(session_handler: u32, dev_port_ingress: u16) {
        unsafe {
            let match_spec: *mut p4_pd_l2_switching_feature_match_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_feature_match_spec_t>()) as *mut p4_pd_l2_switching_feature_match_spec_t;
            let action_spec: *mut p4_pd_l2_switching_feature_enable_action_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_feature_enable_action_spec_t>())
                    as *mut p4_pd_l2_switching_feature_enable_action_spec_t;
            let entry_hdl: *mut p4_pd_entry_hdl_t = malloc(mem::size_of::<p4_pd_entry_hdl_t>()) as *mut p4_pd_entry_hdl_t;

            (*match_spec).ig_intr_md_ingress_port = dev_port_ingress;
            (*action_spec).action_hhd = 0;
            (*action_spec).action_flows = 1;

            p4_pd_l2_switching_feature_table_add_with_feature_enable(
                session_handler,
                resolve_dev_target(),
                match_spec,
                action_spec,
                entry_hdl,
            );
            println!("Enabled Flows {}", *entry_hdl);
        }
    }

    pub fn reset_flows_table(session_handler: u32) {
        unsafe {
            let mut status: p4_pd_status_t = 0;
            while status == 0 {
                let index: *mut i32 = malloc(mem::size_of::<i32>()) as *mut i32;
                status = p4_pd_l2_switching_feature_get_first_entry_handle(session_handler, resolve_dev_target(), index);

                // todo: we will have to verify that it is the HHD entry
                // assuming that there will be more features
                if status == 0 {
                    p4_pd_l2_switching_feature_table_delete(session_handler, 0 as u8, *index as p4_pd_entry_hdl_t);
                    println!("Delete Flows Setting, Handle {}", *index);
                }
            }
        }
    }

    unsafe extern "C" fn callback(
        sess_hdl: p4_pd_sess_hdl_t,
        msg: *mut p4_pd_l2_switching_ipv4_flows_tuple_plus_hash_digest_msg_t,
        _callback_fn_cookie: *mut c_void,
    ) -> u32 {
        let mut entry = (*msg).entries;

        for _ in 0..(*msg).num_entries {
            println!(
                "learned: {:?} {} {:?} {} {} {} {}",
                Ipv4Addr::from((*entry).ipv4_srcAddr),
                (*entry).md_flows_metadata_srcPort,
                Ipv4Addr::from((*entry).ipv4_dstAddr),
                (*entry).md_flows_metadata_dstPort,
                (*entry).ipv4_protocol,
                (*entry).md_flows_metadata_hash1,
                (*entry).md_flows_metadata_hash2,
            );

            // don't learn the ominous 0.0.0.0
            if (*entry).md_flows_metadata_srcPort != 0 {
                FlowsManager::add_learned_flow(Flow {
                    src_addr: Ipv4Addr::from((*entry).ipv4_srcAddr).to_string(),
                    src_addr_int: (*entry).ipv4_srcAddr,
                    src_port: (*entry).md_flows_metadata_srcPort,
                    dst_addr: Ipv4Addr::from((*entry).ipv4_dstAddr).to_string(),
                    dst_addr_int: (*entry).ipv4_dstAddr,
                    dst_port: (*entry).md_flows_metadata_dstPort,
                    ipv4_protocol: (*entry).ipv4_protocol,
                    hash1: (*entry).md_flows_metadata_hash1,
                    hash2: (*entry).md_flows_metadata_hash2,
                });
            };

            entry = entry.offset(1);
        }

        // acknowledge receive
        p4_pd_l2_switching_ipv4_flows_tuple_plus_hash_notify_ack(sess_hdl, msg);

        0 as u32
    }

    pub fn reset_bloomfilters(session_handler: u32, hashes: &Vec<u16>) {
        unsafe {
            let bloom_value: *mut u_int8_t = malloc(mem::size_of::<u_int8_t>()) as *mut u_int8_t;
            *bloom_value = 0;

            p4_pd_l2_switching_register_write_flows_bloom_filter_1(session_handler, resolve_dev_target(), hashes[0] as i32, bloom_value);

            p4_pd_l2_switching_register_write_flows_bloom_filter_2(session_handler, resolve_dev_target(), hashes[1] as i32, bloom_value);
        }
    }

    fn resolve_dev_target() -> p4_pd_dev_target_t {
        p4_pd_dev_target_t {
            device_id: 0 as i32,
            dev_pipe_id: DEV_PIPE_ALL as u16,
        }
    }
}
