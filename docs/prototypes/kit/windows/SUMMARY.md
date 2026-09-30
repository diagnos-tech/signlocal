# windows

Scripts of the Windows proof with software keys (CNG and CAPI).

- `ci-windows.ps1` — creates test certificates, runs `list` and `sign` in every acquisition mode, and checks the results and that nothing prompted
- `invoke-bounded.ps1` — runs a program with a time limit, so a native call that waits for a dialog fails with its name instead of hanging the job
- `make-test-certs.ps1` — creates the CNG, CAPI, and legacy A1 test certificates and removes them at the end
