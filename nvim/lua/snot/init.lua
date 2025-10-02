local M = {}

local config = {
  vault_path = vim.fn.getcwd(),
  snot_bin = "snot",
}

function M.setup(opts)
  opts = opts or {}
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
