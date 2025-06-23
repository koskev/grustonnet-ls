package main

import (
	"encoding/json"
	"testing"

	"github.com/google/go-jsonnet"
	"github.com/google/go-jsonnet/ast"
	"github.com/stretchr/testify/assert"
)

func TestSerialize(t *testing.T) {
	node, err := jsonnet.SnippetToAST("", "[5]")
	assert.NoError(t, err)

	nodeJson, err := json.MarshalIndent(node, "", "  ")
	assert.NoError(t, err)

	newNode, err := Unmarshal_ast[ast.Node](string(nodeJson))
	assert.NoError(t, err)
	newNodeJson, err := json.MarshalIndent(newNode, "", "  ")
	assert.NoError(t, err)

	assert.Equal(t, string(nodeJson), string(newNodeJson))
}
