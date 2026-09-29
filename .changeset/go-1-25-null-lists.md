---
"gozo-cli": patch
---

Fix parsing of `go mod edit -json`, `go work edit -json` and `go list -json` output from Go 1.25 and older, which print `null` for empty lists and made every project look like it had an invalid go.mod.
