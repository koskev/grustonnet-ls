from lsp_recorder import NeovimRecorder
from pathlib import Path

with NeovimRecorder("../../../crates/grustonnet-ls-lib/testdata/complete/docsonnet/func.jsonnet", relative_to_file=__file__).record() as recorder:
    (recorder
     .input("Gkyyp^ry")
     .type("$bb")
     .sleep(1)
     .lsp_rename()
     .type("renamed")
     .sleep(1)
     .enter()
     .sleep(1)
     .type("bb")
     .lsp_rename()
     .type("renamed")
     .sleep(1)
     .enter()
     .sleep(1)
     .quit()
     )
