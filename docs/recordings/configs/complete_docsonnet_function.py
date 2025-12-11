from build import NeovimRecorder

with NeovimRecorder("../../crates/grustonnet-ls-lib/testdata/complete/docsonnet/func.jsonnet", "complete_docsonnet_function").record() as recorder:
    (recorder
     .input("GO")
     .type("  y: self.funcs.myFunc")
     .sleep(3)
     .quit()
     )
