---
"gozo-cli": patch
---

The npm launcher now runs the binary from `target/` when executed inside a source checkout and no longer fails with "Cannot find module 'gozo/package.json'" when it has to install the platform package.
