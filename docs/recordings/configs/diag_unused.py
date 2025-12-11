from build import NeovimRecorder

with NeovimRecorder("test.jsonnet", "diag_unused").record() as recorder:
    (recorder
     .input("ggO")
     .type("local unused = 5;")
     .escape()
     .type("^ee")
     .sleep(1)
     .lsp_code_action()
     .sleep(1)
     .enter()
     .sleep(1)
     .quit()
     )
