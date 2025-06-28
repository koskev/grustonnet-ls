package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"reflect"

	"github.com/google/go-jsonnet"
	"github.com/google/go-jsonnet/ast"
	"github.com/google/go-jsonnet/formatter"
	"github.com/google/go-jsonnet/linter"
)

type GoAst struct {
}

func init() {
	ASTBridgeImpl = GoAst{}
}
func (GoAst) version() string {
	return jsonnet.Version()
}

func get_vm(params *EvaluateParams) *jsonnet.VM {
	vm := jsonnet.MakeVM()
	for _, val := range params.ext_vars {
		vm.ExtVar(val.name, val.value)
	}
	for _, val := range params.ext_code {
		vm.ExtCode(val.name, val.value)
	}
	importer := &jsonnet.FileImporter{JPaths: params.jpaths}
	vm.Importer(importer)
	return vm
}

func (GoAst) get_ast(filename *string) ASTInfo {
	info := ASTInfo{}
	node, _, err := jsonnet.MakeVM().ImportAST("", *filename)
	if err != nil {
		info.error_data = err.Error()
		return info
	}
	nodeJson, err := json.Marshal(node)
	if err != nil {
		info.error_data = err.Error()
		return info
	}
	info.ast_data = string(nodeJson)
	//fmt.Printf("%+s", nodeJson)
	return info
}

func (GoAst) get_ast_snippet(source_file *string, snippet *string) ASTInfo {
	info := ASTInfo{}
	node, err := jsonnet.SnippetToAST(*source_file, *snippet)
	if err != nil {
		// Since go is stupid we are not able to get the underlying error type and thus are forced to just use the string
		info.error_data = err.Error()
		return info
	}
	nodeJson, err := json.Marshal(tagged_marshal(node))
	if err != nil {
		info.error_data = err.Error()
		return info
	}
	info.ast_data = string(nodeJson)
	return info
}

func (GoAst) import_ast(source_file *string, filename *string, params *EvaluateParams) ASTInfo {
	info := ASTInfo{}
	vm := get_vm(params)
	node, _, err := vm.ImportAST(*source_file, *filename)
	if err != nil {
		// Since go is stupid we are not able to get the underlying error type and thus are forced to just use the string
		info.error_data = err.Error()
		return info
	}
	//fmt.Printf("PATHS: %+v", params.jpaths)
	nodeJson, err := json.Marshal(tagged_marshal(node))
	if err != nil {
		info.error_data = err.Error()
		return info
	}
	info.ast_data = string(nodeJson)
	return info
}

func (GoAst) evaluate_ast(astString *string, params *EvaluateParams) ASTInfo {
	info := ASTInfo{}
	node, err := Unmarshal_ast[ast.Node](*astString)
	if err != nil {
		info.error_data = err.Error()
		return info
	}

	vm := get_vm(params)
	res, err := vm.Evaluate(node)
	if err != nil {
		info.error_data = err.Error()
		return info
	}

	info.ast_data = res
	return info
}

func (GoAst) evaluate_snippet(filename *string, snippet *string, params *EvaluateParams) ASTInfo {
	info := ASTInfo{}
	vm := get_vm(params)
	res, err := vm.EvaluateAnonymousSnippet(*filename, *snippet)
	if err != nil {
		info.error_data = err.Error()
		return info
	}

	info.ast_data = res
	return info

}

func (GoAst) lint_snippet(filename *string, snippet *string, params *EvaluateParams) ASTInfo {
	info := ASTInfo{}
	vm := get_vm(params)
	buf := &bytes.Buffer{}
	hasErr := linter.LintSnippet(vm, buf, []linter.Snippet{
		{FileName: *filename, Code: *snippet},
	})
	if hasErr {
		info.error_data = buf.String()
	} else {
		info.ast_data = buf.String()
	}
	return info
}

func (GoAst) format_snippet(filename *string, snippet *string, options *FormatOptions) ASTInfo {
	formatter_options := formatter.DefaultOptions()
	formatter_options.CommentStyle = formatter.CommentStyle(options.comment_style)
	formatter_options.Indent = int(options.indent)
	formatter_options.MaxBlankLines = int(options.max_blank_lines)
	formatter_options.StringStyle = formatter.StringStyle(options.string_style)
	formatter_options.CommentStyle = formatter.CommentStyle(options.comment_style)
	formatter_options.PrettyFieldNames = options.pretty_field_names
	formatter_options.PadArrays = options.pad_arrays
	formatter_options.PadObjects = options.pad_objects
	formatter_options.SortImports = options.sort_imports
	formatter_options.UseImplicitPlus = options.use_implicit_plus
	formatter_options.StripEverything = options.strip_everything
	formatter_options.StripComments = options.strip_comments
	formatter_options.StripAllButComments = options.strip_all_but_comments

	formatted, err := formatter.Format(*filename, *snippet, formatter_options)

	info := ASTInfo{}
	if err != nil {
		info.error_data = err.Error()
	} else {
		info.ast_data = formatted
	}

	return info
}

func get_type(node ast.Node) string {
	return fmt.Sprintf("%T", node)
}

func ignore_error(val any, err error) any {
	return val
}

func to_json_map(val any) map[string]any {
	data, _ := json.Marshal(val)
	var m map[string]any
	_ = json.Unmarshal(data, &m)
	return m
}

func to_json_map_arr[T any](vals []T) []map[string]any {
	var retval []map[string]any
	for _, val := range vals {
		data, _ := json.Marshal(val)
		var m map[string]any
		_ = json.Unmarshal(data, &m)
		retval = append(retval, m)
	}
	return retval
}

func tagged_marshal(val any) map[string]any {
	if val == nil {
		return map[string]any{}
	}
	data := map[string]any{}
	reflect_val := reflect.ValueOf(val)
	if reflect_val.Kind() == reflect.Pointer {
		reflect_val = reflect_val.Elem()
	}
	reflect_type := reflect.TypeOf(val)
	if reflect_type.Kind() == reflect.Pointer {
		reflect_type = reflect_type.Elem()
	}
	data["Type"] = reflect_type.Name()
	for i := range reflect_type.NumField() {
		field_type := reflect_type.Field(i)
		field_val := reflect_val.FieldByName(field_type.Name)
		switch field_val.Kind() {
		case reflect.Slice:
			slice_data := []any{}
			for j := range field_val.Len() {
				if field_val.Index(j).Kind() == reflect.Struct {
					slice_data = append(slice_data, tagged_marshal(field_val.Index(j).Interface()))
				} else {
					slice_data = append(slice_data, field_val.Index(j).Interface())
				}
			}
			data[field_type.Name] = slice_data

		case reflect.Struct, reflect.Interface:
			if field_val.Interface() != nil {
				data[field_type.Name] = tagged_marshal(field_val.Interface())
			}
		case reflect.Pointer:
			if field_val.Elem().Kind() == reflect.Struct {
				data[field_type.Name] = tagged_marshal(field_val.Elem().Interface())
				break
			}
			fallthrough
		default:
			data[field_type.Name] = field_val.Interface()
		}
	}
	return data

}

func Marshal_ast(root_node ast.Node) map[string]any {
	node_map := map[string]any{}
	node_map["Type"] = get_type(root_node)
	node_map["Fodder"] = to_json_map(root_node.OpenFodder())
	node_map["Ctx"] = to_json_map(root_node.Context())
	node_map["FreeVars"] = to_json_map(root_node.FreeVariables())
	node_map["LocRange"] = to_json_map(root_node.Loc())
	// TODO: use reflect magic
	switch current_node := root_node.(type) {
	case *ast.Binary:
		node_map["Left"] = Marshal_ast(current_node.Left)
		node_map["Right"] = Marshal_ast(current_node.Right)
		node_map["OpFodder"] = to_json_map(current_node.Fodder)
	case *ast.Local:
		node_map["Type"] = get_type(current_node)
		node_map["Binds"] = to_json_map_arr(current_node.Binds)
		node_map["Body"] = Marshal_ast(current_node.Body)
	case *ast.Self:
		node_map[get_type(current_node)] = map[string]any{}
	case *ast.Import:
		node_map["File"] = Marshal_ast(current_node.File)
	case *ast.LiteralString:
		node_map["Value"] = current_node.Value
		node_map["BlockIndent"] = current_node.BlockIndent
		node_map["BlockTermIndent"] = current_node.BlockTermIndent
		node_map["Kind"] = current_node.Kind
	case *ast.DesugaredObject:
		asserts := []map[string]any{}
		for _, assert_node := range current_node.Asserts {
			asserts = append(asserts, Marshal_ast(assert_node))
		}
		node_map["Asserts"] = asserts
	}

	return node_map
}

// Since Go is missing the ability to unmarshal interfaces (like the enum untagged in rust). We need to implement this manually...
func Unmarshal_ast[BASE any](astString string) (BASE, error) {
	var ret BASE
	dataMap, err := string_to_json(astString)
	if err != nil {
		return ret, fmt.Errorf("converting %s to json: %w", astString, err)
	}
	if len(dataMap) == 0 {
		return ret, fmt.Errorf("data map is 0! Unable to match anything! astString: %s", astString)
	}
	types_to_test := []reflect.Type{
		reflect.TypeFor[ast.CommaSeparatedExpr](),
		reflect.TypeFor[ast.Array](),
		reflect.TypeFor[ast.LiteralNumber](),
		reflect.TypeFor[ast.Local](),
		reflect.TypeFor[ast.Var](),
	}
	// WTF GO!!?
	nodeType := reflect.TypeOf((*ast.Node)(nil)).Elem()

	// Check for every field in data map if it is present in the struct
	// If yes: Marshal with specific type and do a recursive call for ast.Node types
	// If no: check the next type
typeLoop:
	for _, test_type := range types_to_test {
		for dataName := range dataMap {
			_, hasField := test_type.FieldByName(dataName)
			if !hasField {
				continue typeLoop
			}
		}

		// Field has all fields that are in DataName
		node := reflect.New(test_type)

		for dataName, value := range dataMap {
			field, _ := test_type.FieldByName(dataName)
			fieldValue := reflect.Indirect(node).FieldByName(dataName)

			if field.Type.Implements(nodeType) {
				nodeStr, _ := json.Marshal(value)
				childNode, err := Unmarshal_ast[ast.Node](string(nodeStr))
				if err != nil {
					return ret, fmt.Errorf("getting child node: %w", err)
				}
				if field.Type.Kind() == reflect.Interface {
					fmt.Printf("TYPE: %v", field.Type)
					fieldValue.Set(reflect.ValueOf(childNode))
				} else {
					fieldValue.Set(reflect.ValueOf(&childNode))
				}
			} else {
				// FIXME: There **IS** a better way. What I want to do: Manually Unmarshal Node. And use the default unmarshal for everything else
				switch field.Type {
				// Types with node
				case reflect.TypeFor[ast.CommaSeparatedExpr]():
					byteVal, _ := json.Marshal(value)
					val, err := Unmarshal_ast[ast.CommaSeparatedExpr](string(byteVal))
					if err != nil {
						return ret, fmt.Errorf("marshalling ast comma: %w", err)
					}
					fieldValue.Set(reflect.ValueOf(val))
				case reflect.TypeFor[ast.Context]():
					set_value[ast.Context](value, &fieldValue)
				case reflect.TypeFor[ast.Fodder]():
					set_value[ast.Fodder](value, &fieldValue)
				case reflect.TypeFor[ast.Array]():
					set_value[ast.Array](value, &fieldValue)
				case reflect.TypeFor[ast.LocationRange]():
					set_value[ast.LocationRange](value, &fieldValue)
				case reflect.TypeFor[ast.Identifier]():
					set_value[ast.Identifier](value, &fieldValue)
				case reflect.TypeFor[ast.Identifiers]():
					set_value[ast.Identifiers](value, &fieldValue)
				case reflect.TypeFor[bool]():
					set_value[bool](value, &fieldValue)
				case reflect.TypeFor[ast.CommaSeparatedExpr]():
					set_value[ast.CommaSeparatedExpr](value, &fieldValue)
				case reflect.TypeFor[ast.Local]():
					set_value[ast.Local](value, &fieldValue)
				case reflect.TypeFor[ast.Var]():
					set_value[ast.Var](value, &fieldValue)
				case reflect.TypeFor[[]ast.CommaSeparatedExpr]():
					// Get JSON array
					result := []any{}
					byteVal, _ := json.Marshal(value)
					err := json.Unmarshal(byteVal, &result)
					if err != nil {
						return ret, fmt.Errorf("unmarshalling comma arr: %w", err)
					}

					// For every value in the array: Recursive call and add to array.
					var exprs []ast.CommaSeparatedExpr
					for _, elem := range result {
						objectStr, err := json.Marshal(elem)
						if err != nil {
							return ret, fmt.Errorf("marshalling comma arr: %w", err)
						}
						expr, err := Unmarshal_ast[ast.CommaSeparatedExpr](string(objectStr))
						if err != nil {
							return ret, fmt.Errorf("unmarshal ast comma arr: %w", err)
						}
						exprs = append(exprs, expr)
					}
					fieldValue.Set(reflect.ValueOf(exprs))
				case reflect.TypeFor[string]():
					set_value[string](value, &fieldValue)

				default:

				}
			}

		}
		if reflect.TypeFor[BASE]().Kind() == reflect.Interface {
			return node.Interface().(BASE), nil
		} else {
			return reflect.Indirect(node).Interface().(BASE), nil
		}

	}

	err = json.Unmarshal([]byte(astString), &ret)
	if err != nil {
		return ret, fmt.Errorf("unmarshal unhandled to %v %s: %w", reflect.TypeFor[BASE](), astString, err)
	}
	return ret, nil
}

func set_value[T any](value any, fieldValue *reflect.Value) {
	var fieldType T
	byteVal, _ := json.Marshal(value)
	_ = json.Unmarshal(byteVal, &fieldType)
	fieldValue.Set(reflect.ValueOf(fieldType))
}

func string_to_json(json_string string) (map[string]any, error) {
	result := map[string]any{}
	err := json.Unmarshal([]byte(json_string), &result)
	return result, err
}
