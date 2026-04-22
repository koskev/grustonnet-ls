# Editors

## Neovim
To get autocompletion for the config, it is best to put the config in a json file next to the actual lsp configuration.
To do that, just put a `grustonnet-ls.lua` file in your `after/lsp` directory with the following content. The config will be loaded from a `grustonnet.json` file.

```lua
local function getJson(filename)
	-- https://neovim.io/doc/user/lua.html#lua-script-location
	local current_file = debug.getinfo(1, "S").source:sub(2)
	local current_dir = vim.fn.fnamemodify(current_file, ":h")

	local file_content = vim.fn.readfile(current_dir .. "/" .. filename)
	return vim.json.decode(table.concat(file_content, "\n"))
end

local grustonnet_settings = getJson("grustonnet.json");
return {
	cmd = { "grustonnet-ls" },
	filetypes = { 'jsonnet', 'libsonnet' },
	root_markers = { 'jsonnetfile.json', '.git' },
	settings = grustonnet_settings,
}
```

To get a live preview and debugger support you can install my [jsonnet-tools plugin](https://github.com/koskev/jsonnet-tools.nvim)

## (Evil-)Helix

```toml
[language-server.grustonnet-ls]
command = "grustonnet-ls"
config = {}

[[language]]
name = "jsonnet"
language-servers = ["grustonnet-ls"]
```

To get completion for the config you can just use jsonnet and configure the language server config in `grustonnet.json`:

```jsonnet
local config = {
  'language-server': {
    'grustonnet-ls': {
      command: 'grustonnet-ls',
      config: import './grustonnet.json',
    },
  },
  language: [
    {
      name: 'jsonnet',
      'language-servers': ['grustonnet-ls'],
    },
  ],

};

std.manifestTomlEx(config, ' ')
```

Then run `jsonnet -S config.jsonnet > ~/.config/helix/languages.toml` (or a temporary file if you don't want to override your existing config)


## VSCodium
Search for `grustonnet` and install the extension (identifier `koskev.vscode-grustonnet`). I am currently not verified and I am not sure when I'll do that.

For VSCode users: Add the openvsx marketplace to your editor. I really can't be bothered right now to sign up for another closed ecosystem from Microsoft.

Alternatively just download the file from <https://github.com/koskev/vscode-grustonnet/releases>

## IntelliJ

Just don't. IntelliJ is probably nice for Java development, but for other languages it just seems rather bad and does not have native support LSP, DAP, or Treesitter.

To get basic support you can use [lsp4ij](https://github.com/redhat-developer/lsp4ij). However, you either need to separately install syntax highlighting or use the slow Treesitter bridge of the language server.
