use client::{make_client, report::{self, Request}};

// You can use teenonce=$(head -c 64 /dev/urandom | xxd -p | tr -d '\n') to generate a
// random hex report.

fn main() {
    let report_data: String = std::env::args().nth(1).expect("Please provide report data");
    let bytes = hex::decode(&report_data).expect("Report data must be hex format");
    
    let client = make_client().unwrap();
    let request = Request {
        in_blob: bytes,
        privilege: None,
        get_aux_blob: false
    };

    let mut report = report::create(client, request).unwrap();
    let result = report.get().unwrap().out_blob;

    println!("{:?}", result)
}
