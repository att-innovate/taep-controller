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

use flows::FlowsManager;
use hhd::HHDManager;
use iron::{Iron, IronResult, Request, Response};
use iron::mime::{Mime, SubLevel, TopLevel};
use iron::status;
use l2::{DivertType, L2Manager};
use metrics::MetricsCollector;
use router::Router;
use rustc_serialize::json;
use std::io::Read;
use std::sync::Mutex;
use std::thread;

pub struct APIManager {
    listening_port: u16,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Divert {
    pub port_ingress: u32,
    pub port_egress: u32,
    pub ip_address: String,
    pub ip_prefix_length: u16,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Flows {
    pub port_ingress: u32,
    pub max_number_of_flows: u32,
    pub time_window_in_seconds: u32,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Hhd {
    pub port_ingress: u32,
}

#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct HhdDivert {
    pub port_ingress: u32,
    pub port_ingress_divert: u32,
    pub divert_ingress: u32,
    pub divert_egress: u32,
}

#[derive(Clone, Debug, RustcEncodable)]
struct SimpleResponse {
    result: String,
}

lazy_static! {
    static ref MANAGER: Mutex<APIManager> = Mutex::new(APIManager{listening_port: 0});
}

impl APIManager {
    pub fn run(listening_port: u16) {
        MANAGER.lock().unwrap().listening_port = listening_port;
        println!("APIManager running on port {}", listening_port);
        let _ = thread::Builder::new().name("api-manager".to_string()).spawn(move || loop {
            let mut router = Router::new();

            router.get("/admin/ping", handle_ping, "pingpong");
            router.get("/metrics", handle_get_metrics, "get metrics");
            router.post("/divert/dest", handle_set_divert_dest, "post divert dest");
            router.post("/divert/src", handle_set_divert_src, "post divert src");
            router.patch("/divert/dest", handle_patch_divert_dest, "patch divert dest");
            router.patch("/divert/src", handle_patch_divert_src, "patch divert src");
            router.delete("/divert", handle_reset_divert, "reset divert");
            router.get("/flows", handle_get_learned_flows, "get flows");
            router.post("/flows", handle_set_flow_learning, "post flows");
            router.post("/hhd", handle_set_hhd, "post hhd");
            router.post("/hhd/dest", handle_set_hhd_divert_dest, "post hhd divert dest");
            router.post("/hhd/src", handle_set_hhd_divert_src, "post hhd divert src");
            router.delete("/hhd", handle_reset_hhd, "reset hhd");
            Iron::new(router).http(format!("0.0.0.0:{}", listening_port)).unwrap();
        });
    }
}

fn handle_ping(_request: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "pong")))
}

fn handle_get_metrics(_request: &mut Request) -> IronResult<Response> {
    let result = MetricsCollector::get_port_stats();
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&result).unwrap())))
}

fn handle_set_divert_dest(request: &mut Request) -> IronResult<Response> {
    handle_set_divert(request, DivertType::IPDest)
}

fn handle_set_divert_src(request: &mut Request) -> IronResult<Response> {
    handle_set_divert(request, DivertType::IPSrc)
}

fn handle_set_divert(request: &mut Request, divert_type: DivertType) -> IronResult<Response> {
    let mut body = String::new();
    request.body.read_to_string(&mut body).unwrap();
    let divert: Divert = json::decode(&body).unwrap();

    println!("Set {:?}", divert);

    L2Manager::set_divert(divert_type, divert.port_ingress, divert.port_egress, &divert.ip_address, divert.ip_prefix_length, false);

    let response = SimpleResponse {
        result: "done".to_string(),
    };
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&response).unwrap())))
}

fn handle_patch_divert_dest(request: &mut Request) -> IronResult<Response> {
    handle_patch_divert(request, DivertType::IPDest)
}

fn handle_patch_divert_src(request: &mut Request) -> IronResult<Response> {
    handle_patch_divert(request, DivertType::IPSrc)
}

fn handle_patch_divert(request: &mut Request, divert_type: DivertType) -> IronResult<Response> {
    let mut body = String::new();
    request.body.read_to_string(&mut body).unwrap();
    let divert: Divert = json::decode(&body).unwrap();

    println!("Patch {:?}", divert);

    L2Manager::reset_divert_for_ingress_egress_port(divert.port_ingress, divert.port_egress);

    L2Manager::set_divert(divert_type, divert.port_ingress, divert.port_egress, &divert.ip_address, divert.ip_prefix_length, false);

    let response = SimpleResponse {
        result: "done".to_string(),
    };
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&response).unwrap())))
}

fn handle_reset_divert(_request: &mut Request) -> IronResult<Response> {
    println!("Reset Divert Tables");

    L2Manager::reset_divert_table();

    let response = SimpleResponse {
        result: "done".to_string(),
    };
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&response).unwrap())))
}

fn handle_get_learned_flows(_request: &mut Request) -> IronResult<Response> {
    let result = FlowsManager::get_learned_flows();
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&result).unwrap())))
}

fn handle_set_flow_learning(request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    request.body.read_to_string(&mut body).unwrap();
    let flows: Flows = json::decode(&body).unwrap();

    println!("Set {:?}", flows);

    FlowsManager::set_flow_learning_for_time_window(flows.port_ingress, flows.max_number_of_flows, flows.time_window_in_seconds);

    let response = SimpleResponse {
        result: "done".to_string(),
    };
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&response).unwrap())))
}

fn handle_set_hhd(request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    request.body.read_to_string(&mut body).unwrap();
    let hhd: Hhd = json::decode(&body).unwrap();

    println!("{:?}", hhd);

    HHDManager::set_hhd(hhd.port_ingress);

    let response = SimpleResponse {
        result: "done".to_string(),
    };
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&response).unwrap())))
}

fn handle_set_hhd_divert_dest(request: &mut Request) -> IronResult<Response> {
    handle_set_hhd_divert(request, DivertType::IPDest)
}

fn handle_set_hhd_divert_src(request: &mut Request) -> IronResult<Response> {
    handle_set_hhd_divert(request, DivertType::IPSrc)
}

fn handle_set_hhd_divert(request: &mut Request, divert_type: DivertType) -> IronResult<Response> {
    let mut body = String::new();
    request.body.read_to_string(&mut body).unwrap();
    let hhd: HhdDivert = json::decode(&body).unwrap();

    println!("{:?}", hhd);

    HHDManager::set_hhd(hhd.port_ingress);
    HHDManager::set_hhd(hhd.port_ingress_divert);
    HHDManager::run_hhd_divert(hhd.divert_ingress, hhd.divert_egress, divert_type);

    let response = SimpleResponse {
        result: "done".to_string(),
    };
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&response).unwrap())))
}

fn handle_reset_hhd(_request: &mut Request) -> IronResult<Response> {
    println!("Reset HHD Table");

    HHDManager::reset_hhd();

    let response = SimpleResponse {
        result: "done".to_string(),
    };
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());
    Ok(Response::with((content_type, status::Ok, json::encode(&response).unwrap())))
}
