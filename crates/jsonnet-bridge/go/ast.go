package main

import (
	"bytes"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"reflect"

	"github.com/google/go-jsonnet"
	"github.com/google/go-jsonnet/ast"
	"github.com/google/go-jsonnet/formatter"
	"github.com/google/go-jsonnet/linter"
	"github.com/vmihailenco/msgpack/v5"
)

const (
	Binary          = iota
	Array           = iota
	LiteralNumber   = iota
	LiteralString   = iota
	LiteralBoolean  = iota
	LiteralNull     = iota
	Local           = iota
	Function        = iota
	Apply           = iota
	DesugaredObject = iota
	Index           = iota
	Var             = iota
	Import          = iota
	ImportStr       = iota
	ImportBin       = iota
	Conditional     = iota
	Error           = iota
	Unary           = iota
	InSuper         = iota

	SelfNode   = iota
	SuperIndex = iota
	Dollar     = iota
	// Leftover nodes. Most likely something is broken
	Other = iota
)

type GoAst struct{}

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

func (GoAst) get_ast(filename *string) (info ASTInfo) {
	info = ASTInfo{}
	defer func() {
		if r := recover(); r != nil {
			info.error_data = fmt.Sprintf("GO Error: %v", r)
		}
	}()
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
	info.ast_data = nodeJson
	// fmt.Printf("%+s", nodeJson)
	return info
}

func (GoAst) get_ast_snippet(source_file *string, snippet *string) (info ASTInfo) {
	info = ASTInfo{}
	defer func() {
		if r := recover(); r != nil {
			info.error_data = fmt.Sprintf("GO Error: %v", r)
		}
	}()
	node, err := jsonnet.SnippetToAST(*source_file, *snippet)
	if err != nil {
		// Since go is stupid we are not able to get the underlying error type and thus are forced to just use the string
		info.error_data = err.Error()
		return info
	}
	nodeJson, err := msgpack.Marshal(tagged_marshal(node))
	if err != nil {
		info.error_data = err.Error()
		return info
	}
	info.ast_data = nodeJson
	return info
}

func (GoAst) get_ast_snippet_binary(source_file *string, snippet *string) (info ASTInfo) {
	info = ASTInfo{}
	defer func() {
		if r := recover(); r != nil {
			info.error_data = fmt.Sprintf("GO Error: %v", r)
		}
	}()
	node, err := jsonnet.SnippetToAST(*source_file, *snippet)
	if err != nil {
		// Since go is stupid we are not able to get the underlying error type and thus are forced to just use the string
		info.error_data = err.Error()
		return info
	}
	encoder := JsonnetEncoder{}
	encoder.encode_bincode(node)
	info.ast_data = encoder.buf.Bytes()
	return info
}

func (GoAst) import_ast(source_file *string, filename *string, params *EvaluateParams) (info ASTInfo) {
	info = ASTInfo{}
	defer func() {
		if r := recover(); r != nil {
			info.error_data = fmt.Sprintf("GO Error: %v", r)
		}
	}()
	vm := get_vm(params)
	node, _, err := vm.ImportAST(*source_file, *filename)
	if err != nil {
		// Since go is stupid we are not able to get the underlying error type and thus are forced to just use the string
		info.error_data = err.Error()
		return info
	}
	// fmt.Printf("PATHS: %+v", params.jpaths)
	nodeJson, err := msgpack.Marshal(tagged_marshal(node))
	if err != nil {
		info.error_data = err.Error()
		return info
	}
	info.ast_data = nodeJson
	return info
}

func (GoAst) evaluate_ast(astString *string, params *EvaluateParams) (info ASTInfo) {
	info = ASTInfo{}
	defer func() {
		if r := recover(); r != nil {
			info.error_data = fmt.Sprintf("GO Error: %v", r)
		}
	}()
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

	info.ast_data = []uint8(res)
	return info
}

func (GoAst) evaluate_snippet(filename *string, snippet *string, params *EvaluateParams) (info ASTInfo) {
	info = ASTInfo{}
	defer func() {
		if r := recover(); r != nil {
			info.error_data = fmt.Sprintf("GO Error: %v", r)
		}
	}()
	vm := get_vm(params)
	res, err := vm.EvaluateAnonymousSnippet(*filename, *snippet)
	if err != nil {
		info.error_data = err.Error()
		return info
	}

	info.ast_data = []uint8(res)
	return info
}

func (GoAst) lint_snippet(filename *string, snippet *string, params *EvaluateParams) (info ASTInfo) {
	info = ASTInfo{}

	defer func() {
		if r := recover(); r != nil {
			info.error_data = fmt.Sprintf("GO Error: %v", r)
		}
	}()

	vm := get_vm(params)
	buf := &bytes.Buffer{}
	hasErr := linter.LintSnippet(vm, buf, []linter.Snippet{
		{FileName: *filename, Code: *snippet},
	})
	if hasErr {
		info.error_data = buf.String()
	} else {
		info.ast_data = buf.Bytes()
	}
	return info
}

func (GoAst) format_snippet(filename *string, snippet *string, options *FormatOptions) ASTInfo {
	// It seems like go is very opinionated about unkeyed fields.
	// However, they are just plain wrong and unkeyed fields are way better in this case since we ensure all fields are set
	// If Go would support proper keyed fields (like in Rust) we could switch
	formatter_options := formatter.Options{
		int(options.indent),
		int(options.max_blank_lines),
		formatter.StringStyle(options.string_style),
		formatter.CommentStyle(options.comment_style),
		options.pretty_field_names,
		options.pad_arrays,
		options.pad_objects,
		options.sort_imports,
		options.use_implicit_plus,
		options.strip_everything,
		options.strip_comments,
		options.strip_all_but_comments,
	}

	formatted, err := formatter.Format(*filename, *snippet, formatter_options)

	info := ASTInfo{}
	if err != nil {
		info.error_data = err.Error()
	} else {
		info.ast_data = []uint8(formatted)
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

var BYTE_ORDER = binary.LittleEndian

type JsonnetEncoder struct {
	buf bytes.Buffer
}

func NewJsonnetEncoder() *JsonnetEncoder {
	return &JsonnetEncoder{}
}

var count = 0

func (self *JsonnetEncoder) write(data any) {
	// fmt.Printf("Writing %v (%T)\n", data, data)
	_ = binary.Write(&self.buf, BYTE_ORDER, data)
}

func (self *JsonnetEncoder) encode_string_option(data string) *JsonnetEncoder {
	l := uint64(len(data))
	if l == 0 {
		self.write(uint8(0))
	} else {
		self.write(uint8(1))
		self.write(uint64(len(data)))
		self.write([]byte(data))
	}
	return self
}

func (self *JsonnetEncoder) encode_string(data string) *JsonnetEncoder {
	self.write(uint64(len(data)))
	if len(data) > 0 {
		self.write([]byte(data))
	}
	return self
}

func (self *JsonnetEncoder) encode_bincode_val(val reflect.Value) *JsonnetEncoder {
	switch val.Kind() {
	case reflect.Int:
		self.write(int32(val.Int()))
	case reflect.Slice:
		self.write(uint64(val.Len()))
		for i := range val.Len() {
			slice_val := val.Index(i)
			self.encode_bincode_val(slice_val)
		}
	case reflect.Pointer:
		if val.IsNil() ||
			// Filter out Source as it contains a pointer to the whole file
			reflect.TypeOf(val.Elem().Interface()) == reflect.TypeFor[ast.Source]() {
			self.write(int8(0))
		} else {
			self.write(int8(1))
			self.encode_bincode(val.Elem().Interface())
		}
	case reflect.String:
		self.encode_string(val.String())
	case reflect.Struct, reflect.Interface:
		if val.Interface() != nil {
			self.encode_bincode(val.Interface())
		}
	case reflect.Bool:
		if val.Bool() {
			self.write(int8(1))
		} else {
			self.write(int8(0))
		}
	default:
		panic(fmt.Sprintf("Unknown kind! %v %v", val.Kind(), val))
	}
	return self
}

func (self *JsonnetEncoder) encode_option(val any) {
	if val == nil {
		self.write(uint8(0))
	} else {
		self.write(uint8(1))
		self.encode_bincode(val)
	}
}

func (self *JsonnetEncoder) encode_bincode(val any) *JsonnetEncoder {
	reflect_val := reflect.ValueOf(val)
	if reflect_val.Kind() == reflect.Pointer {
		reflect_val = reflect_val.Elem()
	}
	reflect_type := reflect.TypeOf(val)
	if reflect_type.Kind() == reflect.Pointer {
		reflect_type = reflect_type.Elem()
	}
	if reflect_type.Kind() != reflect.Struct {
		return self.encode_bincode_val(reflect_val)
	}
	// Since the AST objects have the NodeBase at different positions (WHY!!?!) we have to handle each case of the ast
	switch currNode := reflect_val.Interface().(type) {
	case ast.Binary:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Binary))
	case ast.Array:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Array))
	case ast.LiteralNumber:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(LiteralNumber))
	case ast.LiteralString:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(LiteralString))
	case ast.LiteralBoolean:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(LiteralBoolean))
	case ast.LiteralNull:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(LiteralNull))
	case ast.Local:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Local))
		self.encode_bincode(currNode.Binds)
		self.encode_option(currNode.Body)
		return self
	case ast.Function:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Function))
	case ast.Apply:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Apply))
	case ast.DesugaredObject:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(DesugaredObject))
	case ast.Index:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Index))
		self.encode_bincode(currNode.Target)
		self.encode_bincode(currNode.Index)
		self.encode_bincode(currNode.RightBracketFodder)
		self.encode_bincode(currNode.LeftBracketFodder)
		if currNode.Id == nil {
			self.write(uint8(0))
		} else {
			self.write(uint8(1))
			self.encode_string(string(*currNode.Id))
		}
		return self
	case ast.Var:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Var))
		self.encode_string_option(string(currNode.Id))
		return self
	case ast.Import:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Import))
		fileNode := ast.LiteralString{}
		if currNode.File != nil {
			fileNode = *currNode.File
		}
		self.encode_bincode(fileNode)
		return self
	case ast.ImportStr:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(ImportStr))
		fileNode := ast.LiteralString{}
		if currNode.File != nil {
			fileNode = *currNode.File
		}
		self.encode_bincode(fileNode)
		return self
	case ast.ImportBin:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(ImportBin))
		fileNode := ast.LiteralString{}
		if currNode.File != nil {
			fileNode = *currNode.File
		}
		self.encode_bincode(fileNode)
		return self
	case ast.Conditional:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Conditional))
	case ast.Error:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Error))
	case ast.Unary:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Unary))
	case ast.InSuper:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(InSuper))
	case ast.Self:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(SelfNode))
	case ast.SuperIndex:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(SuperIndex))
	case ast.Dollar:
		self.encode_base_node(currNode.NodeBase)
		self.write(uint32(Dollar))
	case ast.LocalBind:
		self.encode_bincode(currNode.VarFodder)
		self.encode_option(currNode.Body)
		self.encode_bincode(currNode.EqFodder)
		self.encode_bincode(currNode.Variable)
		self.encode_bincode(currNode.CloseFodder)
		// XXX: if we vast fun to any it is not nil. WHY?!?!
		if currNode.Fun == nil {
			self.write(uint8(0))
		} else {
			self.write(uint8(1))
			self.encode_bincode(currNode.Fun)
		}
		self.encode_bincode(currNode.LocRange)
		return self
	case ast.Parameter:
		self.encode_bincode(currNode.NameFodder)
		self.encode_bincode(currNode.Name)
		self.encode_bincode(currNode.CommaFodder)
		self.encode_bincode(currNode.EqFodder)
		self.encode_option(currNode.DefaultArg)
		self.encode_bincode(currNode.LocRange)
		return self
	}
	for i := range reflect_type.NumField() {
		field_type := reflect_type.Field(i)
		if field_type.Anonymous {
			// Skip NodeBase to have it at the same position every time
			continue
		}
		field_val := reflect_val.FieldByName(field_type.Name)
		self.encode_bincode_val(field_val)
	}

	return self
}

func (self *JsonnetEncoder) encode_base_node(node ast.NodeBase) *JsonnetEncoder {
	self.encode_bincode(node.Fodder)
	if node.Ctx == nil || true {
		self.encode_string("")
	} else {
		self.encode_string(*node.Ctx)
	}
	self.encode_bincode(node.FreeVars)
	self.encode_bincode(node.LocRange)
	return self
}

func (GoAst) get_test_objects() []TestData {
	emptyCtx := ""
	return []TestData{
		{
			name: "fodder",
			data: NewJsonnetEncoder().encode_bincode(ast.Fodder{
				{
					Comment: []string{
						"one",
						"two",
					},
					Kind:   1,
					Blanks: 2,
					Indent: 3,
				},
			}).buf.Bytes(),
		},
		{
			name: "location",
			data: NewJsonnetEncoder().encode_bincode(ast.Location{
				Line:   5,
				Column: 19,
			}).buf.Bytes(),
		},
		{
			name: "locrange",
			data: NewJsonnetEncoder().encode_bincode(ast.LocationRange{
				FileName: "test",
				Begin: ast.Location{
					Line:   1,
					Column: 2,
				},
				End: ast.Location{
					Line:   3,
					Column: 4,
				},
			}).buf.Bytes(),
		},
		{
			name: "self",
			data: NewJsonnetEncoder().encode_bincode(ast.Self{}).buf.Bytes(),
		},
		{
			name: "apply",
			data: NewJsonnetEncoder().encode_bincode(ast.Apply{
				Target: &ast.Self{},
			}).buf.Bytes(),
		},
		{
			name: "array",
			data: NewJsonnetEncoder().encode_bincode(ast.Array{}).buf.Bytes(),
		},
		{
			name: "local_self",
			data: NewJsonnetEncoder().encode_bincode(ast.Local{
				Body: &ast.Self{},
			}).buf.Bytes(),
		},
		{
			name: "local_empty",
			data: NewJsonnetEncoder().encode_bincode(ast.Local{}).buf.Bytes(),
		},
		{
			name: "node_base",
			data: NewJsonnetEncoder().encode_bincode(ast.NodeBase{
				Fodder:   ast.Fodder{},
				FreeVars: ast.Identifiers{},
				LocRange: ast.LocationRange{
					FileName: "",
					Begin: ast.Location{
						Line:   1,
						Column: 1,
					},
					End: ast.Location{
						Line:   1,
						Column: 3,
					},
				},
				Ctx: &emptyCtx,
			}).buf.Bytes(),
		},
	}
}

func tagged_marshal(val any) map[string]any {
	if val == nil {
		return map[string]any{"T": "empty"}
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
	data["T"] = reflect_type.Name()
	for i := range reflect_type.NumField() {
		field_type := reflect_type.Field(i)
		field_val := reflect_val.FieldByName(field_type.Name)
		switch field_val.Kind() {
		case reflect.Slice:
			slice_data := []any{}
			for j := range field_val.Len() {
				switch field_val.Index(j).Kind() {
				case reflect.Struct, reflect.Interface:
					slice_data = append(slice_data, tagged_marshal(field_val.Index(j).Interface()))
				case reflect.Pointer:
					slice_data = append(slice_data, tagged_marshal(field_val.Elem().Index(j).Interface()))
				default:
					slice_data = append(slice_data, field_val.Index(j).Interface())
				}
			}
			data[field_type.Name] = slice_data

		case reflect.Struct, reflect.Interface:
			if field_val.Interface() != nil {
				data[field_type.Name] = tagged_marshal(field_val.Interface())
			}
		case reflect.Pointer:
			// XXX: Filter out *Source pointers as they contain the whole file. Which would mean every node has a copy of the whole file
			if field_val.Elem().Kind() == reflect.Struct &&
				reflect.TypeOf(field_val.Elem().Interface()) != reflect.TypeFor[ast.Source]() {
				data[field_type.Name] = tagged_marshal(field_val.Elem().Interface())
			}
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
