# Quote Generation Through the TSM ABI

Before the TSM ABI was for getting measurements thorugh configfs, but it has been extended to also have an attestation agent get the TDX Quote data. We're going to use this in situations where the driver won't support `<TDG.VP.VMCALL<GetQuote>` iotcl calls and if the quote generation service is not connected through vsock.
