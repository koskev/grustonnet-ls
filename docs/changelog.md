## [unreleased]

### 🚀 Features

- *(debug)* More trace logging
- *(tokens)* Add support for bridging treesitter tokens for editors without any basic features
- *(bin)* Run with godebug by default
- *(actions)* Prepare for code actions
- *(diagnostics)* Cache the last diagnostic results
- *(actions)* Add support for code actions
- *(diag)* Add support for variable naming diagnostics
- *(diag)* Add linting for weird function definitions
- *(diag)* Add code action for renaming a local function
- *(diag)* Add diag to prevent dollar usage
- *(diag)* Add ability to disable lints by adding a comment
- *(diag)* Update nolint query to only disable the lint for the next line
- *(inlay)* Replace : with = in parameters to have a valid jsonnet syntax
- *(diag)* Add diagnostics for recursion in default function arguments
- *(diag)* Add a simple separate binary to lint code
- *(linter)* Add severity info to output
- *(linter)* Add note if there is an automatic fix available in the ls
- *(diag)* Add linter for shadowed variables
- *(diag)* Add diagnostics for duplicate detection
- *(diag)* Add more config options for duplicate string detection
- *(diag)* Add default uri and source name to the diagresults
- *(diag)* Increase default duplicate detection threshold to 5
- *(lint)* Force colors to be on
- *(command)* Add support for getting extvars, extcode, and jpaths via lsp commands
- *(node)* Add ability to get apply name with a var target
- *(signature)* Add basic signature help
- *(signature)* Add argument positional information for signature help
- *(completion)* Add docsonnet snippets
- *(output)* Better display implementation for vars
- *(logging)* Move performance logging to debug
- *(logging)* Change default loglevel to info
- *(inlay)* Don't show the inlay hint if the variable name is identical to the parameter name
- *(diagnostics)* Replace busy wait with a condvar
- *(nix)* Add GODEBUG variable to flake
- *(lint)* Add parameters to code action message for function linter
- *(rust2go)* Add a new crate for rust2go restart workaround
- *(build)* Add support for cross compiling to windows
- *(linter)* Add support for code climate output
- *(complete)* Add newobject snippet for docsonnet
- *(diagnostics)* Add diagnostics for wrong docsonnet defaults
- *(command)* Properly format an eval error during a jsonnet.evalFile call

### 🐛 Bug Fixes

- *(completion)* Fix completion inside nodes without a valid position
- *(log)* Replace println with trace log
- *(completion)* Fix completion inside object locals
- *(completion)* Fix completion inside an object's assert
- *(completion)* Fix completion with a broken binary
- Use correct token result to fix compilation
- *(test)* Add missing test file
- *(completion)* Fix completion for the first node in an array after an assert
- *(diag)* Fix locations of shadow diagnostics
- *(inlay)* Fix inlay hints for apply with a var target
- *(snippets)* Only show docsonnet snippets in an object
- *(completion)* Add missing std.is functions
- *(config)* Prioritize explicitly set extcode over extcode from files
- *(lint)* Also consider functions without a parameter for function lints
- *(definition)* Fix definition location for nodes without a location
- *(completion)* Fix global completion of object function parameters
- *(definition)* Fix goto definition of arguments that are in an object field
- *(definition)* Fit goto definition in function args
- *(references)* Check the url as well for the self reference

### 💼 Other

- *(diag)* Create linter dir and remove unused imports
- Fix clippy warnings
- Add first code for arguments documentation

### 🚜 Refactor

- *(location)* Move jsonnet location code to their own crate
- *(cst)* Move cst code to a new crate
- Move filter logic to its own file
- *(diag)* Remove duplicate code for diagnostics
- *(lint)* Remove macro to have better completion
- *(node)* Move the nodes out of the lib and into their own crate
- *(config)* Move configuration to its own crate
- *(bridge)* Move evaluate error to bridge crate
- *(config)* Move FormatOptions to config
- *(signature)* Add common function to get apply and function node
- Use a wrapper for apply function combo

### 📚 Documentation

- Add mdbook with some example recordings
- Add a few more example recordings
- Compile all recordings at the same time
- Add basic editor instructions
- Add notifier to prevent possible endless loop

### 🧪 Testing

- *(completion)* Add test for completing inside std function parameters
- Add missing changes for the loop test
- Add failing test
- Add dollar import test
- Enable a deactivated test
- Fix diagnostic tests
- *(language-server)* Add tests for multiple diagnostic files
- *(lint)* Add test cases for snake case linter
- *(lint)* Add test cases for unused variables
- Fix test compilation
- *(diag)* Add support for ignoring some fields
- *(diag)* Add tests for shadow diagnostics
- *(diag)* Add test for dollar diagnostics
- *(diag)* Add tests for recursive arguments
- *(inlay)* Add tests for apply inlay hints
- Add simple docsonnet test
- *(function)* Move some function tests to their correct file
- *(function)* Add a few (failing) tests
- *(completion)* Add test for std completions
- *(definition)* Add failing test for function arguments
- *(references)* Ensure references stick to the correct file

### ⚙️ Miscellaneous Tasks

- *(fmt)* Format some code
- Remove a bunch of unused files
- *(git)* Update gitignore
- *(docker)* Add dockerfile to repo
- Add git cliff config
- Remove old file
- *(recording)* Add script for generating demo recordings
- Add conform config with pre commit hooks
- Fix clippy warnings and add clippy to pre-push hooks
