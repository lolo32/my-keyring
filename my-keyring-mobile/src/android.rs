use std::ffi::CString;

use android_logger::{Config, FilterBuilder};
use jni::{
    objects::{JClass, JString},
    sys::jstring,
    JNIEnv,
};
use log::Level;

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_eu_baysse_mykeyring_helper_RustGreeting_greeting(
    env: JNIEnv,
    _: JClass,
    java_pattern: JString,
) -> jstring {
    // Our Java companion code might pass-in "world" as a string, hence the name.
    let world = super::rust_greeting(
        env.get_string(java_pattern)
            .expect("invalid pattern string")
            .as_ptr(),
    );
    // Retake pointer so that we can use it below and allow memory to be freed when
    // it goes out of scope.
    let world_ptr = CString::from_raw(world);
    let output = env
        .new_string(world_ptr.to_str().unwrap())
        .expect("Couldn't create java string!");

    output.into_inner()

    // let recipient = CString::from(CStr::from_ptr(
    //     env.get_string(j_recipient).unwrap().as_ptr(),
    // ));

    // let output = env
    //     .new_string("Hello ".to_owned() + recipient.to_str().unwrap())
    //     .unwrap();
    // output.into_inner()
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_eu_baysse_mykeyring_helper_Init_initlib(_: JNIEnv, _: JClass) {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Trace)
            .with_tag("MyKeyring")
            .with_filter(FilterBuilder::new().parse("mykeyring").build()),
    );
}

#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_eu_baysse_mykeyring_helper_Push_sendToken(
    env: JNIEnv,
    _: JClass,
    java_pattern: JString,
) -> bool {
    // Our Java companion code might pass-in "world" as a string, hence the name.
    super::rust_send_token(
        env.get_string(java_pattern)
            .expect("invalid pattern string")
            .as_ptr(),
    )
}
