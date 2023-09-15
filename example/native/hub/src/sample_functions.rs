//! This module is only for demonstration purposes.
//! You might want to remove this module in production.

use crate::bridge::api::{RustOperation, RustRequest, RustResponse, RustSignal};
use crate::bridge::send_rust_signal;
use crate::messages::counter_number::{SampleSchema, SampleSchemaArgs};

pub async fn handle_counter_number(rust_request: RustRequest) -> RustResponse {
    use crate::messages::counter_number::{ReadRequest, ReadResponse, ReadResponseArgs};

    match rust_request.operation {
        RustOperation::Create => RustResponse::default(),
        RustOperation::Read => {
            // We import message structs in this match condition
            // because schema will differ by the operation type.

            // Decode raw bytes into a Rust message object.
            let request_message = flatbuffers::root::<ReadRequest>(&rust_request.bytes).unwrap();

            // Perform a simple calculation.
            let after_value: i32 = sample_crate::add_seven(request_message.before_number());

            // Return the response that will be sent to Dart.
            let mut builder = flatbuffers::FlatBufferBuilder::new();
            let dummy_two = Some(SampleSchema::create(
                &mut builder,
                &SampleSchemaArgs {
                    sample_field_one: false,
                    sample_field_two: false,
                },
            ));
            let dummy_three =
                Some(builder.create_vector(request_message.dummy_three().unwrap().safe_slice()));
            let response_message = ReadResponse::create(
                &mut builder,
                &ReadResponseArgs {
                    after_number: after_value,
                    dummy_one: request_message.dummy_one(),
                    dummy_two,
                    dummy_three,
                },
            );
            RustResponse {
                successful: true,
                bytes: response_message.to_be_bytes().to_vec(),
            }
        }
        RustOperation::Update => RustResponse::default(),
        RustOperation::Delete => RustResponse::default(),
    }
}

pub async fn handle_sample_resource(rust_request: RustRequest) -> RustResponse {
    match rust_request.operation {
        RustOperation::Create => RustResponse::default(),
        RustOperation::Read => RustResponse::default(),
        RustOperation::Update => RustResponse::default(),
        RustOperation::Delete => RustResponse::default(),
    }
}

pub async fn stream_mandelbrot() {
    use crate::messages::mandelbrot::ID;

    let mut scale: f64 = 1.0;
    let mut interval = crate::time::interval(std::time::Duration::from_millis(50));

    loop {
        interval.tick().await;

        scale *= 0.95;
        if scale < 1e-7 {
            scale = 1.0
        };

        let calculated = sample_crate::mandelbrot(
            sample_crate::Size {
                width: 64,
                height: 64,
            },
            sample_crate::Point {
                x: 0.360,
                y: -0.641,
            },
            scale,
            4,
        );

        if let Ok(mandelbrot) = calculated {
            // Stream the signal to Dart.
            let rust_signal = RustSignal {
                resource: ID,
                bytes: mandelbrot,
            };
            send_rust_signal(rust_signal);
        }
    }
}
