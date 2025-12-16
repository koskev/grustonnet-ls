from build import NeovimRecorder
from pathlib import Path

with NeovimRecorder("../../crates/grustonnet-ls-lib/testdata/complete/docsonnet/func.jsonnet", Path(__file__).stem).record() as recorder:
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
