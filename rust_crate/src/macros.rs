#![allow(clippy::crate_in_macro_def)]

#[macro_export]
/// Writes the interface code
/// needed to communicate with Dart.
/// This should be used once, and only once,
/// at the root of the `hub` crate.
macro_rules! write_interface {
    () => {
        #[cfg(not(target_family = "wasm"))]
        mod interface_os {
            use crate::tokio::runtime::Builder;
            use crate::tokio::runtime::Runtime;
            use rinf::externs::os_thread_local::ThreadLocal;
            use std::cell::RefCell;
            use std::panic::catch_unwind;
            use std::sync::OnceLock;

            // We use `os_thread_local` so that when the program fails
            // and the main thread exits unexpectedly,
            // the whole async tokio runtime can disappear as well.
            type TokioRuntime = OnceLock<ThreadLocal<RefCell<Option<Runtime>>>>;
            static TOKIO_RUNTIME: TokioRuntime = OnceLock::new();

            #[no_mangle]
            pub extern "C" fn start_rust_logic_extern() {
                let _ = catch_unwind(|| {
                    // Enable backtrace output for panics.
                    #[cfg(debug_assertions)]
                    {
                        use rinf::debug_print;
                        use rinf::externs::backtrace::Backtrace;
                        std::panic::set_hook(Box::new(|panic_info| {
                            let backtrace = Backtrace::new();
                            debug_print!("A panic occurred in Rust.\n{panic_info}\n{backtrace:?}");
                        }));
                    }

                    // Run the main function.
                    let tokio_runtime = Builder::new_multi_thread().enable_all().build().unwrap();
                    tokio_runtime.spawn(crate::main());
                    let os_cell =
                        TOKIO_RUNTIME.get_or_init(|| ThreadLocal::new(|| RefCell::new(None)));
                    os_cell.with(move |cell| {
                        // If there was already a tokio runtime previously,
                        // most likely due to Dart's hot restart,
                        // its tasks as well as itself will be terminated,
                        // being replaced with the new one.
                        cell.replace(Some(tokio_runtime));
                    });
                });
            }

            #[no_mangle]
            pub extern "C" fn stop_rust_logic_extern() {
                let _ = catch_unwind(|| {
                    let os_cell =
                        TOKIO_RUNTIME.get_or_init(|| ThreadLocal::new(|| RefCell::new(None)));
                    os_cell.with(move |cell| {
                        // If there was already a tokio runtime previously,
                        // most likely due to Dart's hot restart,
                        // its tasks as well as itself will be terminated,
                        // being replaced with the new one.
                        cell.replace(None);
                    });
                });
            }

            #[no_mangle]
            pub extern "C" fn send_dart_signal_extern(
                message_id: i64,
                message_pointer: *const u8,
                message_size: usize,
                blob_valid: bool,
                blob_pointer: *const u8,
                blob_size: usize,
            ) {
                let message_bytes = unsafe {
                    Vec::from_raw_parts(message_pointer as *mut u8, message_size, message_size)
                };
                let blob = if blob_valid {
                    unsafe {
                        Some(Vec::from_raw_parts(
                            blob_pointer as *mut u8,
                            blob_size,
                            blob_size,
                        ))
                    }
                } else {
                    None
                };
                let _ = catch_unwind(|| {
                    crate::messages::generated::handle_dart_signal(
                        message_id as i32,
                        message_bytes,
                        blob,
                    );
                });
            }
        }

        #[cfg(target_family = "wasm")]
        mod interface_web {
            use crate::tokio;
            use std::panic::catch_unwind;
            use wasm_bindgen::prelude::wasm_bindgen;

            #[wasm_bindgen]
            pub fn start_rust_logic_extern() {
                let _ = catch_unwind(|| {
                    // Add kind description for panics.
                    #[cfg(debug_assertions)]
                    {
                        use rinf::debug_print;
                        std::panic::set_hook(Box::new(|panic_info| {
                            debug_print!("A panic occurred in Rust.\n{panic_info}");
                        }));
                    }

                    // Run the main function.
                    tokio::spawn(crate::main());
                });
            }

            #[wasm_bindgen]
            pub fn send_dart_signal_extern(
                message_id: i32,
                message_bytes: &[u8],
                blob_valid: bool,
                blob_bytes: &[u8],
            ) {
                let message_bytes = message_bytes.to_vec();
                let blob = if blob_valid {
                    Some(blob_bytes.to_vec())
                } else {
                    None
                };
                let _ = catch_unwind(|| {
                    crate::messages::generated::handle_dart_signal(message_id, message_bytes, blob);
                });
            }
        }
    };
}

/// Delegates the printing operation to Flutter,
/// which excels at handling various platforms
/// including web and mobile emulators.
/// When debugging, using this macro is recommended over `println!()`,
/// as it seamlessly adapts to different environments.
/// Note that this macro does nothing in release mode.
#[macro_export]
macro_rules! debug_print {
    ( $( $t:tt )* ) => {
        let rust_report = format!( $( $t )* );
        #[cfg(debug_assertions)]
        rinf::send_rust_signal(
            -1, // This is a special message ID for Rust reports
            Vec::new(),
            Some(rust_report.into_bytes()),
        );
        #[cfg(not(debug_assertions))]
        let _ = rust_report;
    }
}
