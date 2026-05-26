{
  config,
  lib,
  ...
}:
let
  cfg = config.programs.crawldesktopshell;
in
{
  options.programs.crawldesktopshell = {};
  config = lib.mkIf cfg.enable {};
}