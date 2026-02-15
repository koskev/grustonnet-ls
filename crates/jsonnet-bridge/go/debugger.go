package main

import (
	"fmt"

	"github.com/google/go-jsonnet"
	"github.com/google/go-jsonnet/ast"
)

type GoDebugger struct{}

var (
	debugger  = jsonnet.MakeDebugger()
	last_node ast.Node
)

func init() {
	DebuggerBridgeImpl = GoDebugger{}
}

func copy_string(input *string) string {
	return fmt.Sprintf("%s", *input)
}

func (self GoDebugger) step_over() {
	debugger.ContinueUntilAfter(last_node)
}

func (self GoDebugger) step() {
	debugger.Step()
}

func (self GoDebugger) lookup_value(identifier *string) StringInfo {
	info, err := debugger.LookupValue(copy_string(identifier))
	err_str := ""
	if err != nil {
		err_str = err.Error()
	}
	return StringInfo{
		data:  info,
		error: err_str,
	}
}

func (self GoDebugger) list_vars() ASTInfo {
	encoder := JsonnetEncoder{}
	encoder.encode_bincode(debugger.ListVars())
	return ASTInfo{
		ast_data: encoder.buf.Bytes(),
	}
}

func (self GoDebugger) continue_debugger() {
	debugger.Continue()
}

func (GoDebugger) get_stack_trace() ASTInfo {
	info := ASTInfo{}
	encoder := JsonnetEncoder{}
	encoder.encode_bincode(debugger.StackTrace())
	info.ast_data = encoder.buf.Bytes()

	return info
}

func (self GoDebugger) wait_for_event() ASTInfo {
	info := ASTInfo{}
	event := <-debugger.Events()
	switch ev := event.(type) {
	case *jsonnet.DebugEventStop:
		last_node = ev.Current
	}
	encoder := JsonnetEncoder{}
	encoder.encode_bincode(event)
	info.ast_data = encoder.buf.Bytes()
	return info
}

func (self GoDebugger) launch(filename *string, content *string, params *EvaluateParams) {
	// We need to make a copy of the content since the launch will start a go routine
	content_copy := copy_string(content)
	filename_copy := copy_string(filename)
	for _, val := range params.ext_vars {
		debugger.GetVM().ExtVar(val.name, val.value)
	}
	for _, val := range params.ext_code {
		debugger.GetVM().ExtCode(val.name, val.value)
	}

	debugger.Launch(filename_copy, content_copy, []string{})
}

func (self GoDebugger) get_breakpoints() []string {
	return debugger.ActiveBreakpoints()
}

func (self GoDebugger) clear_breakpoints(file *string) {
	debugger.ClearBreakpoints(copy_string(file))
}

func (self GoDebugger) add_breakpoint(file *string, line *int64, column *int64) StringInfo {
	filename_copy := copy_string(file)
	val, err := debugger.SetBreakpoint(filename_copy, int(*line), int(*column))
	err_str := ""
	if err != nil {
		err_str = err.Error()
	}
	return StringInfo{
		val, err_str,
	}
}
