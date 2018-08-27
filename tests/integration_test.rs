// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

extern crate ffi_utils;
extern crate rand;
extern crate system_uri;
#[macro_use]
extern crate unwrap;

use rand::Rng;
#[cfg(target_os = "linux")]
use std::process::Command;

#[cfg(not(target_os = "macos"))]
use system_uri::open;
use system_uri::{install, App};

#[cfg(target_os = "linux")]
fn clean_string(input: &str) -> String {
    input.replace(".", "").replace("/", "").to_ascii_lowercase()
}

#[cfg(target_os = "linux")]
fn check(scheme: &str, vendor: &str, name: &str) {
    let scheme = clean_string(scheme);
    let vendor = clean_string(vendor);
    let name = clean_string(name);

    if std::env::var("TRAVIS").is_err() {
        println!("opening {}:test", scheme);
        unwrap!(open(format!("{}://test", scheme)));
    } else {
        // in travis we can only check the configuration, but not open
        // as we don't actually have a desktop
        println!("in travis");

        let output = Command::new("xdg-mime")
            .arg("query")
            .arg("default")
            .arg(format!("x-scheme-handler/{}", scheme))
            .output()
            .expect("xdg-mime failed");

        assert!(output.status.success());

        let scheme_handler_name = format!("{}-{}.desktop\n", vendor, name);

        assert_eq!(
            String::from_utf8_lossy(output.stdout.as_slice()),
            scheme_handler_name.to_owned()
        );
    }
}

#[cfg(target_os = "macos")]
fn check(_: &str, _: &str, _: &str) {
    // unfortunately registration won't work in mac unless we have a bundle
    assert!(true);
}

#[cfg(target_os = "windows")]
fn check(scheme: &str, _: &str, _: &str) {
    let _ = unwrap!(open(format!("{}:test", scheme)));
}

fn gen_rand_schema() -> String {
    let mut rng = rand::thread_rng();
    format!("testschema-ABC-{}", rng.gen::<u32>())
}

#[test]
fn install_and_check() {
    let vendor = String::from("MaidSafe");
    let app_name = String::from("Example1");
    let schema = gen_rand_schema();
    let exec = String::from(unwrap!(unwrap!(std::env::current_exe()).to_str()));
    println!("{:} for {}", exec, schema);
    let app = App::new(
        "net.maidsafe.example".to_string(),
        vendor.clone(),
        app_name.clone(),
        exec,
        None,
    );

    assert!(install(&app, &[schema.clone()]).is_ok());
    check(&schema, &vendor, &app_name);
}

#[test]
fn exec_multiple_args() {
    let vendor = String::from("MaidSafe");
    let app_name = String::from("Example2");
    let schema = gen_rand_schema();
    let mut exec = String::from(unwrap!(unwrap!(std::env::current_exe()).to_str()));
    exec.push_str(" arg1 arg2");
    println!("{:} for {}", exec, schema);
    let app = App::new(
        "net.maidsafe.example".to_string(),
        vendor.clone(),
        app_name.clone(),
        exec,
        None,
    );

    assert!(install(&app, &[schema.clone()]).is_ok());
    check(&schema, &vendor, &app_name);
}

#[test]
fn ffi_install_and_check() {
    use ffi_utils::test_utils::call_0;
    use std::ffi::CString;
    use std::os::raw::c_char;
    use system_uri::ffi::install as ffi_install;

    let exec = unwrap!(unwrap!(std::env::current_exe()).to_str()).to_owned();
    // let's copy the executable to a path
    // with a white space to test it's supported
    let exec_with_white_space = format!("{} after_white_space", exec);
    assert!(unwrap!(std::fs::copy(exec.clone(), exec_with_white_space.clone())) > 0);

    let schema = gen_rand_schema();
    let schema_cstr = unwrap!(CString::new(schema.clone()));
    println!("{:} for {}", exec_with_white_space, schema);
    let vendor = "MaidSafe";
    let app_name = "Example3";
    let bundle_str = unwrap!(CString::new("net.maidsafe.example"));
    let vendor_str = unwrap!(CString::new(vendor));
    let app_name_str = unwrap!(CString::new(app_name));
    let icon = unwrap!(CString::new(""));
    let mut args_vec: Vec<*const c_char> = vec![];
    let arg1 = unwrap!(CString::new(exec_with_white_space));
    let arg2 = unwrap!(CString::new("arg2"));
    args_vec.push(arg1.as_ptr());
    args_vec.push(arg2.as_ptr());

    unsafe {
        unwrap!(call_0(|user_data, callback| ffi_install(
            bundle_str.as_ptr(),
            vendor_str.as_ptr(),
            app_name_str.as_ptr(),
            args_vec.as_ptr() as *const *const c_char,
            args_vec.len(),
            icon.as_ptr(),
            schema_cstr.as_ptr(),
            user_data,
            callback,
        )))
    };
    check(&schema, vendor, app_name);
}
