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

extern crate bindgen;

use std::path::Path;

static BF_SDE_PATH: &'static str = "/root/bf-sde";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Link dependencies
    println!("cargo:rustc-link-lib=pd");
    println!("cargo:rustc-link-lib=pdcli");
    println!("cargo:rustc-link-lib=bfutils");
    println!("cargo:rustc-link-lib=bfsys");
    println!("cargo:rustc-link-lib=bfdiags");
    println!("cargo:rustc-link-lib=pltfm_driver");
    println!("cargo:rustc-link-lib=pltfm_mgr");
    println!("cargo:rustc-link-lib=driver");
    println!("cargo:rustc-link-lib=bf_switchd_lib");
    println!("cargo:rustc-link-lib=tofinopdfixed_thrift");
    println!("cargo:rustc-link-search={}/install/lib/tofinopd/l2_switching", BF_SDE_PATH);
    println!("cargo:rustc-link-search={}/install/lib", BF_SDE_PATH);

    // Generating Stubs for required barefoot libraries
    //
    let mut clang_args = vec![format!("-I{}/install/include", BF_SDE_PATH)];
    let stub_path = Path::new("./gen-stub/");

    let bindings_bfswitch = bindgen::Builder::default()
        .header(stub_path.join("wrapper-bfswitch.h").to_str().unwrap())
        .clang_args(clang_args.into_iter())
        .generate()
        .expect("Unable to generate bindings");

    bindings_bfswitch.write_to_file(stub_path.join("bindings-bfswitch.rs")).expect("Couldn't write bindings!");

    // Generating Stubs for our taep libraries
    //
    clang_args = vec![format!("-I{}/install/include", BF_SDE_PATH)];

    let bindings_taep = bindgen::Builder::default()
        .header(stub_path.join("wrapper-taep.h").to_str().unwrap())
        .clang_args(clang_args.into_iter())
        .generate()
        .expect("Unable to generate bindings");

    bindings_taep.write_to_file(stub_path.join("bindings-taep.rs")).expect("Couldn't write bindings!");
}
