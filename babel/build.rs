fn main() {
    if let Err(e) = tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(
            &[
                // Backend API
                "babelsup.proto",
            ],
            &["../babel_api/proto"],
        )
    {
        eprintln!("Building protos failed with:\n{e}");
        std::process::exit(1);
    }
}
