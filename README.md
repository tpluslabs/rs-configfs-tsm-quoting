# Quote Generation Through the TSM ABI

Before the TSM ABI was for getting measurements thorugh configfs, but it has been extended to also have an attestation agent get the TDX Quote data. We're going to use this in situations where the driver won't support `<TDG.VP.VMCALL<GetQuote>` iotcl calls and if the quote generation service is not connected through vsock.

> Note: this only includes the tdx quote for the guest os. It doesn't deal with the vTPM TD to get the combined attestation.

## Test it out

Clone the repository, build the `test-quote` binary and:

```
sudo ./target/release/test-quote $(head -c 64 /dev/urandom | xxd -p | tr -d '\n') > quote
```

## Usage

The API is pretty straightforward:

```rust
// construct the client and request. `in_blob` is the TD report data.
let client = make_client().unwrap();
let request = Request {
    in_blob: bytes,
    privilege: None,
    get_aux_blob: false
};

let mut report = report::create(client, request).unwrap();
let result = report.get().unwrap().out_blob;
```
