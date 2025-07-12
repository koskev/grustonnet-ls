# **G**o **R**ust Js**onnet** **L**anguage **S**erver
This is a jsonnet language server using the `go-jsonnet` implementation to generate the AST and evaluate jsonnet code


## Known Issues

* Jsonnet bug: If you import `foo.libsonnet` and there is also a `foo.libsonnet` in the current working directory, evaluating the snippet will result in a diagnostic error
 * To reproduce `cat mydir/bar.jsonnet | jsonnet --jpath mydir -`
