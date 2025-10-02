local M = {}

local config = {
  vault_path = vim.fn.getcwd(),
  snot_bin = "snot",
}

local function expand_path(path)
  if not path then
    return path
  end
  -- Expand ~ to home directory
  local expanded = vim.fn.expand(path)
  -- Convert to absolute path
  return vim.fn.fnamemodify(expanded, ":p")
end

function M.setup(opts)
  opts = opts or {}

  -- Expand paths before merging
  if opts.vault_path then
    opts.vault_path = expand_path(opts.vault_path)
  end

  config = vim.tbl_deep_extend("force", config, opts)

  -- Create user commands
  require("snot.commands").setup(config)

  -- Set up auto-completion
  if opts.enable_completion ~= false then
    require("snot.completion").setup()

    -- Try to set up nvim-cmp integration if available
    pcall(function()
      require("snot.completion").setup_cmp()
    end)
  end
end

function M.get_config()
  return config
end

return M
