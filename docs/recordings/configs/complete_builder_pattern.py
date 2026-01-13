from lsp_recorder import NeovimRecorder

with NeovimRecorder("../../../crates/grustonnet-ls-lib/testdata/complete/builder/nested.jsonnet", relative_to_file=__file__).record() as recorder:
    (recorder
     .input("GO")
     .enter()
     .type("  y:: self.new().withoutArg()\r.")
     .sleep(1)
     .type("withArg(3)\r.")
     .sleep(1)
     .type("withInner()\r.")
     .sleep(1)
     .type("withInnerFunc()\r.")
     .sleep(1)
     .type("innerVal")
     .sleep(1)
     .escape()
     .input("bcw")
     .type("endInner()\r.")
     .sleep(1)
     .type("innerVal")
     .sleep(1)
     .quit()
     )
