use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use hyper::{body::to_bytes, client::Client, Body, Method, Request, Response};
use log::{debug, error};

use error::Error;

/// Expose the JNI interface for android below
#[cfg(target_os = "android")]
pub mod android;

mod error;

const SERVER: &str = "http://192.168.1.5:3000/";

#[no_mangle]
pub extern "C" fn rust_greeting(to: *const c_char) -> *mut c_char {
    debug!("rust_greeting");
    let c_str = unsafe { CStr::from_ptr(to) };
    let recipient = c_str.to_str().unwrap_or("there");

    CString::new("Hello ".to_owned() + recipient)
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn rust_greeting_free(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}

#[no_mangle]
pub extern "C" fn rust_send_token(token: *const c_char) -> bool {
    debug!("Sending new token");
    let c_str = unsafe { CStr::from_ptr(token) };
    let token = c_str.to_str().unwrap_or("");
    if token.is_empty() {
        return false;
    }

    send_request(Method::POST, "/api/v1/token", token)
        .map(|_| true)
        .unwrap_or(false)
}

fn send_request<T>(method: Method, uri: &str, value: T) -> Result<Response<Body>, Error>
where
    Body: From<T>,
{
    // Build the async runtime
    let rt = tokio::runtime::Runtime::new().unwrap();
    // Init to handle HTTPS
    let https = hyper_rustls::HttpsConnector::with_native_roots();
    let client = Client::builder().build(https);

    // Build the request
    let req = Request::builder()
        .method(method)
        .uri(format!("{}{}", SERVER, uri))
        .body(Body::from(value))
        .expect("request builder");

    // Block waiting async code to execute
    rt.block_on(async {
        // Launch request
        let mut res = client.request(req).await?;

        // Retrieve the status
        let status = res.status();
        // Retrieve the response body in Vec<u8>
        let bytes = to_bytes(res.body_mut()).await?.to_vec();

        // If it's an error, convert the body to string and return the error
        if status.as_u16() > 399 {
            let body = unsafe { String::from_utf8_unchecked(bytes) };
            let err = Err((status, body.as_str()).into());
            error!("An error occurred: {:?}", err);
            return err;
        }

        // Convert back the response bytes into body and store it inside the response
        let body = bytes.into();
        *res.body_mut() = body;

        Ok(res)
    })
}
