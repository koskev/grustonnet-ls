from build import NeovimRecorder

with NeovimRecorder("../../crates/grustonnet-ls-lib/testdata/complete/array/index.jsonnet", "array_index").record() as recorder:
    recorder.input("Gko").type("y: myArr[0].").sleep(1).type("keyZero.").sleep(1).type("innerZero,").enter().type("z: myArr[1].").sleep(1).type("keyOne.").sleep(1).quit()
