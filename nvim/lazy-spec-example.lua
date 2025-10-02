-- Example lazy.nvim plugin spec for SNOT
-- Add this to your lazy.nvim plugin configuration

return {
  dir = "~/dev/snot/nvim",  -- or wherever you cloned the repo
  name = "snot",
  opts = {
    -- Path to your notes vault (supports ~ expansion)
    vault_path = "~/notes",

    -- Path to snot binary (defaults to "snot" in PATH)
    snot_bin = "snot",  -- or full path like "/usr/local/bin/snot"

    -- Enable auto-completion (default: true)
    enable_completion = true,

    -- File picker: "auto" (detects fzf-lua/telescope), "fzf-lua", "telescope", or "select"
    picker = "auto",
  },
  -- Optional: define keymaps
  keys = {
    { "<leader>nn", "<cmd>NoteNew<cr>", desc = "New note" },
    { "<leader>nf", "<cmd>NoteFind<cr>", desc = "Find note" },
    { "<leader>ns", "<cmd>NoteSearch<cr>", desc = "Search notes" },
    { "<leader>nb", "<cmd>NoteBacklinks<cr>", desc = "Show backlinks" },
    { "<leader>ni", "<cmd>NoteIndex<cr>", desc = "Index vault" },
    { "<leader>nl", "<cmd>NoteLink<cr>", desc = "Insert link" },
  },
  -- Optional: lazy load on commands
  cmd = {
    "NoteNew",
    "NoteFind",
    "NoteSearch",
    "NoteBacklinks",
    "NoteIndex",
    "NoteInit",
    "NoteLink",
  },
  -- Optional: lazy load on markdown files
  ft = "markdown",
}
