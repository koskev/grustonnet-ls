from lsp_recorder import NeovimRecorder

with NeovimRecorder("test.jsonnet", "diag_function").record() as recorder:
    (recorder
     .input("ggO")
     .type("local myFunc = function(x) 5*x;")
     .escape()
     .type("jo  x: myFunc(5),")
     .escape()
     .sleep(1)
     .input("gg")
     .sleep(2)
     .lsp_code_action()
     .sleep(1)
     .enter()
     .sleep(1)
     .quit()
     )
