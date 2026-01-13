from lsp_recorder import NeovimRecorder

with NeovimRecorder("test.jsonnet", "preview").record() as recorder:
    (recorder
     .command("JsonnetPreview")
     .input("GO")
     .type("  foo: 5,")
     .escape()
     .type("obar: 'myString',")
     .escape()
     .type("omyVar: self.bar,")
     .escape()
     .type("omyFunc(arg):: arg,")
     .escape()
     .type("omyCall: self.myFunc('from call'),")
     .escape()
     .sleep(2)
     .type("ox: error")
     .escape()
     .sleep(1)
     .command("JsonnetPreview")
     .quit()
     )
