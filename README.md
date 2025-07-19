# **G**o **R**ust Js**onnet** **L**anguage **S**erver
This is a jsonnet language server using the `go-jsonnet` implementation to generate the AST and evaluate jsonnet code

## Roadmap

* [-] Completion
    * [x] Global completion
    * [x] Index completion
    * [-] Complete "everything"
        * [ ] Find the remaining cases where completion does not work/tries to complete the wrong node
    * [x] Stdlib
    * [-] Advanced Stdlib completion
        * [x] extVar completion
        * [ ] Return values
        * [ ] Function parameters e.g. std.map
    * [ ] Complete Loops
    * [x] All jsonnet imports
    * [x] self
    * [x] super
        * [ ]  Fix super not working if it never had an index
    * [-] Keyword completion
        * [ ] Only complete if valid
        * [ ] Add missing keywords
    * [-] Conditionals
        * [ ] Actually evaluate the condition
    * [x] Default parameters
    * [-] Builder pattern
        * [ ] Check extremely complex patterns
    * [ ] Array access
    * [ ] Unused function arguments
* [x] Semantic tokens
* [-] Inlay Hints
    * [x] Function parameters
    * [ ] Indices
* [x] Goto definition
    * Can go everywhere we can complete
* [x] Find reference
    * Can find references for everything we can goto
* [x] Rename
    * Can rename everything we can find the reference
* [ ] Signature Help
* [ ] Docsonnet support
    * [ ] Handle the stdlib the same as docsonnet?
* [-] AST repair
* [ ] Code actions
* [ ] Improve performance
    * [ ] Test rust2go mem
    * [ ] More multithreading
* [ ] More tests
    * [ ] Fix ignored tests
* [ ] Code cleanup

## Known Issues

* Jsonnet bug: If you import `foo.libsonnet` and there is also a `foo.libsonnet` in the current working directory, evaluating the snippet will result in a diagnostic error
 * To reproduce `cat mydir/bar.jsonnet | jsonnet --jpath mydir -`
* Jsonnet bug:
 * If there is a circular dependency go-jsonnet emits a strange error
