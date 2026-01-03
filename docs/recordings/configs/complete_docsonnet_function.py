from lsp_recorder import NeovimRecorder

with NeovimRecorder("../../../crates/grustonnet-ls-lib/testdata/complete/docsonnet/func.jsonnet", relative_to_file=__file__).record() as recorder:
    (recorder
     .input("GO")
     .type("  y: self.funcs.myFunc")
     .sleep(3)
     .quit()
     )
