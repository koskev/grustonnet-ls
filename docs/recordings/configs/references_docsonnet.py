from lsp_recorder import NeovimRecorder
from pathlib import Path

with NeovimRecorder("../../../crates/grustonnet-ls-lib/testdata/complete/docsonnet/func.jsonnet", relative_to_file=__file__).record() as recorder:
    (recorder
     .type("Gk$bb")
     .sleep(1)
     .lsp_references()
     .sleep(1)
     .escape().escape()
     .type("bb")
     .sleep(1)
     .lsp_references()
     .sleep(1)
     .escape().escape()
     .quit()
     )
