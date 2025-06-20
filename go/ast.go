package main

import (
	"encoding/json"
	"fmt"

	"github.com/google/go-jsonnet"
)

type GoAst struct{}

func init() {
	GenerateASTImpl = GoAst{}
}

func (GoAst) get_ast(filename *string) string {
	node, _, _ := jsonnet.MakeVM().ImportAST("", *filename)
	nodeJson, _ := json.Marshal(node)
	fmt.Printf("%+s", nodeJson)
	return string(nodeJson)
}

func (GoAst) get_ast_snippet(snippet *string) string {
	node, _ := jsonnet.SnippetToAST("", *snippet)
	nodeJson, _ := json.Marshal(node)
	fmt.Printf("%+s", nodeJson)
	return string(nodeJson)
}
