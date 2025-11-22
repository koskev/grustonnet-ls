# **G**o **Rust** Js**onnet** **L**anguage **S**erver
This is a jsonnet language server using the `go-jsonnet` implementation to generate the AST and evaluate jsonnet code

## Roadmap

* [-] Completion
    * [x] Global completion
    * [x] Index completion
    * [x] Value preview
        * [x] Make the Object preview pretty
    * [-] Complete "everything"
        * [ ] Find the remaining cases where completion does not work/tries to complete the wrong node
    * [x] Stdlib
    * [-] Advanced Stdlib completion
        * [x] extVar completion
        * [ ] Return values
        * [ ] Function parameters e.g. std.map
    * [ ] Complete Loops
        * [ ] Use `std.map` for loops
    * [x] All jsonnet imports
        * [ ] Properly handle completion if "/" is already in the string
    * [x] self
    * [x] super
        * [ ]  Fix super not working if it never had an index
    * [-] Keyword completion
        * [ ] Only complete if valid
        * [ ] Add missing keywords
    * [-] Conditionals
        * [ ] Actually evaluate the condition
    * [x] Default parameters
    * [x] Builder pattern
        * [ ] Check extremely complex patterns
    * [x] Array access
    * [ ] Unused function arguments
* [x] Semantic tokens
* [-] Inlay Hints
    * [x] Function parameters
        * [ ] Only update if needed
    * [ ] Indices
    * [x] Name after long objects
* [x] Goto definition
    * [x] Goto file from import string
    * Can goto everything we can complete
* [x] Find reference
    * [ ] Import strings
    * Can find references for all identifiers we can goto
* [x] Rename
    * [ ] Rename imports if file is renamed
    * [ ] Rename file if import is renamed
    * Can rename all identifiers we can find the reference of
* [x] Signature Help
* [x] Docsonnet support
    * [x] How to handle the license issues? Docsonnet does not have an open source license
        * Just evaluate it
    * [ ] Handle the stdlib the same as docsonnet?
    * [ ] Signature help parameters
* [-] AST repair
* [x] Commands
    * [x] Evaluate file
* [ ] Missing LSP features
    * [ ] Code actions
    * [ ] Code Lense?
    * [ ] Hover
    * [ ] Document highlight
    * [ ] Document/Workspace symbols
    * [ ] Folding
    * [ ] Call hierachie
    * [ ] File operation support for automatic refactoring (like renaming imports)
* [ ] Improve performance
    * [ ] Test rust2go mem
    * [ ] More multithreading
* [ ] More tests
    * [ ] Fix ignored tests
* [ ] (Major) Code cleanup
    * Once the prototyping phase is over

## Known Issues

* (Go)-Jsonnet bugs
    * If you import `foo.libsonnet` and there is also a `foo.libsonnet` in the current working directory, evaluating the snippet will result in a diagnostic error
        * To reproduce `cat mydir/bar.jsonnet | jsonnet --jpath mydir -`
    * If there is a circular dependency go-jsonnet emits a strange error

## Jsonnet Quirks
* `tailstrict`
    * not part of the spec apart from the reserved keyword
    * no documentation at all
    * in `foo(myArg()) tailstrict` forces myArg to be evaluated before the body, even if it is unused
