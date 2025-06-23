package main

import (
	"encoding/json"
	"fmt"
	"reflect"

	"github.com/google/go-jsonnet"
	"github.com/google/go-jsonnet/ast"
)

type GoAst struct{}

func init() {
	GenerateASTImpl = GoAst{}
}

func (GoAst) get_ast(filename *string) string {
	node, _, _ := jsonnet.MakeVM().ImportAST("", *filename)
	nodeJson, _ := json.Marshal(node)
	//fmt.Printf("%+s", nodeJson)
	return string(nodeJson)
}

func (GoAst) get_ast_snippet(snippet *string) string {
	node, err := jsonnet.SnippetToAST("", *snippet)
	if err != nil {
		fmt.Printf("### Unmarshal ERROR in snippet: %+v", err)
		return fmt.Sprintf("%v", err)
	}
	nodeJson, err := json.Marshal(node)
	if err != nil {
		fmt.Printf("### Unmarshal ERROR in snippet: %+v", err)
		return fmt.Sprintf("%v", err)
	}
	return string(nodeJson)
}

func (GoAst) evaluate_ast(astString *string, ext_vars *[]ExtValue, ext_code *[]ExtValue) string {
	node, err := Unmarshal_ast[ast.Node](*astString)
	if err != nil {
		fmt.Printf("### Unmarshal ERROR: %+v", err)
		return ""
	}
	vm := jsonnet.MakeVM()
	for _, val := range *ext_vars {
		vm.ExtVar(val.name, val.value)
	}
	for _, val := range *ext_code {
		vm.ExtCode(val.name, val.value)
	}

	res, err := vm.Evaluate(node)
	if err != nil {
		return fmt.Sprintf("%v", err)
	}

	return res
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
		return ret, fmt.Errorf("unmarshal unhandled to %T: %w", ret, err)
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
