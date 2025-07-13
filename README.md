# **G**o **R**ust Js**onnet** **L**anguage **S**erver
This is a jsonnet language server using the `go-jsonnet` implementation to generate the AST and evaluate jsonnet code

## TODO
* Implement remaining features to be on par with my go version of the server
    * Signature Help
    * Complete parameters
    * Inlay hints for indices
    * More settings
    * For loop completion
    * Activate all tests
    * More tests
    * Fix super not working if it never had an index
    * Conditionals
    * Improve AST fixing
* Clean up code (waaaaay to many clones)

## Known Issues

* Jsonnet bug: If you import `foo.libsonnet` and there is also a `foo.libsonnet` in the current working directory, evaluating the snippet will result in a diagnostic error
 * To reproduce `cat mydir/bar.jsonnet | jsonnet --jpath mydir -`
