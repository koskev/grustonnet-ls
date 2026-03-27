module goJsonnetBridge

go 1.24.5

require (
	github.com/google/go-jsonnet v0.22.0
	github.com/ihciah/rust2go v0.0.0-20260314034108-6f06b0697c1b
	github.com/stretchr/testify v1.11.1
	github.com/vmihailenco/msgpack/v5 v5.4.1
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	github.com/vmihailenco/tagparser/v2 v2.0.0 // indirect
	go.yaml.in/yaml/v2 v2.4.3 // indirect
	golang.org/x/crypto v0.45.0 // indirect
	golang.org/x/sys v0.38.0 // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
	sigs.k8s.io/yaml v1.6.0 // indirect
)

replace github.com/google/go-jsonnet => github.com/koskev/go-jsonnet v0.0.0-20260327173611-b5123e53fd4d
