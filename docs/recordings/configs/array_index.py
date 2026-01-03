from lsp_recorder import NeovimRecorder
from pathlib import Path

with NeovimRecorder("../../../crates/grustonnet-ls-lib/testdata/complete/array/index.jsonnet", relative_to_file=__file__).record() as recorder:
    recorder.input("Gko").type("y: myArr[0].").sleep(1).type("keyZero.").sleep(1).type("innerZero,").enter().type("z: myArr[1].").sleep(1).type("keyOne.").sleep(1).quit()
