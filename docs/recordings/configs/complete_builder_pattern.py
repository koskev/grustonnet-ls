from build import NeovimRecorder

with NeovimRecorder("../../crates/grustonnet-ls-lib/testdata/complete/builder/nested.jsonnet", "complete_builder_pattern").record() as recorder:
    (recorder
     .input("28G$i")
     .enter()
     .type(".withoutArg()\r.")
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
