import subprocess
import pathlib
import multiprocessing
import pynvim
import os
import sys
import time
import tempfile
import argparse

from contextlib import contextmanager
from typing import Tuple

parser = argparse.ArgumentParser("simple_example")
parser.add_argument("output", help="Output dir", type=str, default="out", nargs="?")
args = parser.parse_args()

OUT_DIR = pathlib.Path(args.output)

class NeovimRecorder:
    def __init__(self, input_file, output_file, delay=0.1):
        self.socket_path = tempfile.mktemp()
        self.delay = delay
        self.input_file = input_file
        self.output = output_file

    @contextmanager
    def record(self):
        self.asciinema_proc = subprocess.Popen([
            'asciinema', 'rec',
            '--command', f'nvim --listen {self.socket_path} --cmd "set noswapfile" {self.input_file}',
            '--overwrite',
            OUT_DIR/self.output
            ])
        self.wait_for_nvim()
        self.nvim = pynvim.attach("socket", path=self.socket_path)
        time.sleep(0.5)

        try:
            yield self
        finally:
            self._cleanup()

    def _cleanup(self):
        self.asciinema_proc.wait()

    def wait_for_nvim(self):
        while not os.path.exists(self.socket_path):
            time.sleep(0.1)

    def type(self, text: str):
        for c in text:
            self.input(c)
            time.sleep(self.delay)
        return self

    def insert_mode(self):
        return self.type("i")


    def input(self, input: str):
        self.nvim.feedkeys(input, "n")
        return self

    def enter(self):
        return self.input("\r")

    def escape(self):
        return self.input('\x1b')

    def sleep(self, secs):
        time.sleep(secs)
        return self

    def go_definition(self):
        self.nvim.command("lua vim.lsp.buf.definition()")
        return self

    def quit(self):
        return self.escape().input(":q!").enter()

def array_index():
    with NeovimRecorder("../crates/grustonnet-ls-lib/testdata/complete/array/index.jsonnet", "array_index.out").record() as recorder:
        recorder.input("GO").type("  y: myArr[0].").sleep(1).type("keyZero.").sleep(1).quit()
def definition_import():
    with NeovimRecorder("../crates/grustonnet-ls-lib/testdata/definition/import_simple.jsonnet ", "array_index.out").record() as recorder:
        recorder.input("Gk5e").sleep(1).lsp_definition().sleep(1).quit()

RECORDS = [
    array_index,
    definition_import,
]

def build(func):
    func()

if __name__ == "__main__":
    #test_jsonnet()
    with multiprocessing.Pool() as pool:
        pool.map(build, RECORDS)

