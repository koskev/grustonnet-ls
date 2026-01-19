JSONNET_VERSION:="0.21.0"

vendor_stdlib:
    curl -fsSLO https://raw.githubusercontent.com/google/jsonnet/v{{JSONNET_VERSION}}/doc/_stdlib_gen/stdlib-content.jsonnet
    jsonnet -J . -J crates/grustonnet-ls-lib crates/grustonnet-ls-lib/stdlib.jsonnet | jq . > crates/grustonnet-ls-lib/stdlib.json
