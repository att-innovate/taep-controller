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

pub mod L2Layer {
    include!("../../gen-stub/bindings-taep.rs");

    use l2::DivertType;
    use std::mem;
    use std::os::raw::c_int;

    pub fn init() -> u32 {
        unsafe {
            let session_handler: *mut p4_pd_sess_hdl_t = malloc(mem::size_of::<p4_pd_sess_hdl_t>()) as *mut p4_pd_sess_hdl_t;
            let status: p4_pd_status_t;

            println!("Get session handle");
            status = p4_pd_client_init(session_handler);
            println!("Status {}", status);
            println!("Session {}", *session_handler);

            *session_handler
        }
    }

    pub fn configure_from_port_to_port(session_handler: u32, port_1: u16, port_2: u16) {
        unsafe {
            let match_spec: *mut p4_pd_l2_switching_forward_match_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_forward_match_spec_t>()) as *mut p4_pd_l2_switching_forward_match_spec_t;
            let action_spec: *mut p4_pd_l2_switching_set_egr_action_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_set_egr_action_spec_t>()) as *mut p4_pd_l2_switching_set_egr_action_spec_t;
            let entry_hdl: *mut p4_pd_entry_hdl_t = malloc(mem::size_of::<p4_pd_entry_hdl_t>()) as *mut p4_pd_entry_hdl_t;

            (*match_spec).ig_intr_md_ingress_port = port_1;
            (*action_spec).action_egress_spec = port_2;

            p4_pd_l2_switching_forward_table_add_with_set_egr(session_handler, resolve_dev_target(), match_spec, action_spec, entry_hdl);
            println!("Added entry to Forwarding Table, Handle {}", *entry_hdl);
        }
    }

    pub fn set_divert(
        session_handler: u32,
        divert_type: DivertType,
        dev_port_ingress: u16,
        dev_port_egress: u16,
        ip_address: u32,
        mask: u32,
        high_priority: bool,
    ) {
        unsafe {
            let match_spec: *mut p4_pd_l2_switching_divert_match_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_divert_match_spec_t>()) as *mut p4_pd_l2_switching_divert_match_spec_t;
            let action_spec: *mut p4_pd_l2_switching_set_egr_action_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_set_egr_action_spec_t>()) as *mut p4_pd_l2_switching_set_egr_action_spec_t;
            let entry_hdl: *mut p4_pd_entry_hdl_t = malloc(mem::size_of::<p4_pd_entry_hdl_t>()) as *mut p4_pd_entry_hdl_t;
            let default_priority = 10 as c_int;
            let default_high_priority = 1 as c_int;

            (*match_spec).ig_intr_md_ingress_port = dev_port_ingress;

            match divert_type {
                DivertType::IPSrc => {
                    (*match_spec).ipv4_srcAddr = ip_address;
                    (*match_spec).ipv4_srcAddr_mask = mask;
                    (*match_spec).ipv4_dstAddr = 0;
                    (*match_spec).ipv4_dstAddr_mask = 0;
                }
                DivertType::IPDest => {
                    (*match_spec).ipv4_dstAddr = ip_address;
                    (*match_spec).ipv4_dstAddr_mask = mask;
                    (*match_spec).ipv4_srcAddr = 0;
                    (*match_spec).ipv4_srcAddr_mask = 0;
                }
            }

            (*action_spec).action_egress_spec = dev_port_egress;

            p4_pd_l2_switching_divert_table_add_with_set_egr(
                session_handler,
                resolve_dev_target(),
                match_spec,
                if high_priority {
                    default_high_priority
                } else {
                    default_priority
                },
                action_spec,
                entry_hdl,
            );
            println!("Added Detour Dest, Handle {}", *entry_hdl);
        }
    }

    pub fn reset_divert_table(session_handler: u32) {
        unsafe {
            let mut status: p4_pd_status_t = 0;
            while status == 0 {
                let handle: *mut i32 = malloc(mem::size_of::<i32>()) as *mut i32;
                status = p4_pd_l2_switching_divert_get_first_entry_handle(session_handler, resolve_dev_target(), handle);

                if status == 0 {
                    p4_pd_l2_switching_divert_table_delete(session_handler, 0 as u8, *handle as p4_pd_entry_hdl_t);
                    println!("Delete Divert Rule, Handle {}", *handle);
                }
            }
        }
    }

    pub fn reset_divert_for_ingress_egress_port(session_handler: u32, dev_port_ingress: u16, dev_port_egress: u16) {
        unsafe {
            let p4_pd_dev_target: *mut p4_pd_dev_target_t = malloc(mem::size_of::<p4_pd_dev_target_t>()) as *mut p4_pd_dev_target_t;
            (*p4_pd_dev_target).device_id = 0 as c_int;
            (*p4_pd_dev_target).dev_pipe_id = DEV_PIPE_ALL as u16;
            let entry_count: *mut u32 = malloc(mem::size_of::<u32>()) as *mut u32;
            p4_pd_l2_switching_divert_get_entry_count(session_handler, *p4_pd_dev_target, entry_count);

            if *entry_count == 0 {
                return;
            }

            let mut entry_handles = Vec::new();

            let handle: *mut i32 = malloc(mem::size_of::<i32>()) as *mut i32;
            let mut next_handle: *mut i32 = malloc(mem::size_of::<i32>()) as *mut i32;

            p4_pd_l2_switching_divert_get_first_entry_handle(session_handler, resolve_dev_target(), handle);
            entry_handles.push(*handle);

            let entry_count_to_retrieve = (*entry_count) - 1;

            if entry_count_to_retrieve > 0 {
                p4_pd_l2_switching_divert_get_next_entry_handles(
                    session_handler,
                    *p4_pd_dev_target,
                    *handle as u32,
                    entry_count_to_retrieve as c_int,
                    next_handle,
                );

                for _ in 0..entry_count_to_retrieve {
                    entry_handles.push(*next_handle);
                    next_handle = next_handle.offset(1);
                }
            }

            for _ in 0..(*entry_count) {
                delete_entry_if_match(session_handler, entry_handles.pop().unwrap() as u32, dev_port_ingress, dev_port_egress);
            }
        }
    }

    fn delete_entry_if_match(session_handler: u32, handle: p4_pd_entry_hdl_t, dev_port_ingress: u16, dev_port_egress: u16) -> bool {
        let mut entry_deleted = false;

        unsafe {
            let status: p4_pd_status_t;
            let match_spec: *mut p4_pd_l2_switching_divert_match_spec_t =
                malloc(mem::size_of::<p4_pd_l2_switching_divert_match_spec_t>()) as *mut p4_pd_l2_switching_divert_match_spec_t;
            let priority: *mut c_int = malloc(mem::size_of::<c_int>()) as *mut c_int;
            let action_spec: *mut p4_pd_l2_switching_action_specs_t =
                malloc(mem::size_of::<p4_pd_l2_switching_action_specs_t>()) as *mut p4_pd_l2_switching_action_specs_t;

            status = p4_pd_l2_switching_divert_get_entry(
                session_handler,
                0 as u8,
                handle as p4_pd_entry_hdl_t,
                true,
                match_spec,
                priority,
                action_spec,
            );

            if status == 0 {
                if dev_port_ingress == (*match_spec).ig_intr_md_ingress_port
                    && dev_port_egress == (*action_spec).u.p4_pd_l2_switching_set_egr.action_egress_spec
                {
                    p4_pd_l2_switching_divert_table_delete(session_handler, 0 as u8, handle as p4_pd_entry_hdl_t);
                    println!("Delete Divert Rule, Handle {}", handle);
                    entry_deleted = true;
                }
            }
        }

        entry_deleted
    }

    fn resolve_dev_target() -> p4_pd_dev_target_t {
        p4_pd_dev_target_t {
            device_id: 0 as i32,
            dev_pipe_id: DEV_PIPE_ALL as u16,
        }
    }
}
