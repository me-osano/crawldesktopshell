{
  config,
  lib,
  ...
}:
let
  cfg = config.services.crawlds-shell;
in
{
  options.services.crawlds-shell = {
    enable = lib.mkEnableOption "CrawlDS shell systemd service";

    package = lib.mkOption {
      type = lib.types.package;
      description = "The crawlds-shell package to use";
    };

    target = lib.mkOption {
      type = lib.types.str;
      default = "graphical-session.target";
      example = "hyprland-session.target";
      description = "The systemd target for the crawlds-shell service.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.crawlds-shell = {
      description = "CrawlDS Shell - Wayland desktop shell";
      documentation = [ "https://docs.crawlds.dev" ];
      after = [ cfg.target ];
      partOf = [ cfg.target ];
      wantedBy = [ cfg.target ];
      restartTriggers = [ cfg.package ];

      environment = {
        PATH = lib.mkForce null;
      };

      serviceConfig = {
        ExecStart = lib.getExe cfg.package;
        Restart = "on-failure";
      };
    };

    environment.systemPackages = [ cfg.package ];
  };
}
