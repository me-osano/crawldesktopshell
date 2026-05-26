{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.crawlds-shell;
  jsonFormat = pkgs.formats.json { };
  tomlFormat = pkgs.formats.toml { };

  generateJson =
    name: value:
    if lib.isString value then
      pkgs.writeText "crawlds-${name}.json" value
    else if builtins.isPath value || lib.isStorePath value then
      value
    else
      jsonFormat.generate "crawlds-${name}.json" value;
in
{
  options.programs.crawlds-shell = {
    enable = lib.mkEnableOption "CrawlDS shell configuration";

    systemd.enable = lib.mkEnableOption "CrawlDS shell systemd integration";

    package = lib.mkOption {
      type = lib.types.nullOr lib.types.package;
      description = "The crawlds-shell package to use";
    };

    settings = lib.mkOption {
      type =
        with lib.types;
        oneOf [
          jsonFormat.type
          str
          path
        ];
      default = { };
      example = lib.literalExpression ''
        {
          bar = {
            position = "bottom";
            floating = true;
            backgroundOpacity = 0.95;
          };
          general = {
            animationSpeed = 1.5;
            radiusRatio = 1.2;
          };
          colorSchemes = {
            darkMode = true;
            useWallpaperColors = true;
          };
        }
      '';
      description = ''
        CrawlDS shell configuration settings as an attribute set, string
        or filepath, to be written to ~/.config/crawlds/settings.json.
      '';
    };

    colors = lib.mkOption {
      type =
        with lib.types;
        oneOf [
          jsonFormat.type
          str
          path
        ];
      default = { };
      example = lib.literalExpression ''
         {
           cError = "#dddddd";
           cOnError = "#111111";
           cOnPrimary = "#111111";
           cOnSecondary = "#111111";
           cOnSurface = "#828282";
           cOnSurfaceVariant = "#5d5d5d";
           cOnTertiary = "#111111";
           cOutline = "#3c3c3c";
           cPrimary = "#aaaaaa";
           cSecondary = "#a7a7a7";
           cShadow = "#000000";
           cSurface = "#111111";
           cSurfaceVariant = "#191919";
           cTertiary = "#cccccc";
        }
      '';
      description = ''
        CrawlDS shell color configuration as an attribute set, string
        or filepath, to be written to ~/.config/crawlds/colors.json.
      '';
    };

    user-templates = lib.mkOption {
      default = { };
      type =
        with lib.types;
        oneOf [
          tomlFormat.type
          str
          path
        ];
      example = lib.literalExpression ''
        {
          templates = {
            neovim = {
              input_path = "~/.config/crawlds/templates/template.lua";
              output_path = "~/.config/nvim/generated.lua";
              post_hook = "pkill -SIGUSR1 nvim";
            };
          };
        }
      '';
      description = ''
        Template definitions for CrawlDS, to be written to ~/.config/crawlds/user-templates.toml.

        This option accepts:
        - a Nix attrset (converted to TOML automatically)
        - a string containing raw TOML
        - a path to an existing TOML file
      '';
    };

  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.crawlds-shell = lib.mkIf cfg.systemd.enable {
      Unit = {
        Description = "CrawlDS Shell - Wayland desktop shell";
        Documentation = "https://docs.crawlds.dev";
        PartOf = [ config.wayland.systemd.target ];
        After = [ config.wayland.systemd.target ];
        X-Restart-Triggers =
          lib.optional (cfg.settings != { }) "${config.xdg.configFile."crawlds/settings.json".source}"
          ++ lib.optional (cfg.colors != { }) "${config.xdg.configFile."crawlds/colors.json".source}"
          ++ lib.optional (
            cfg.user-templates != { }
          ) "${config.xdg.configFile."crawlds/user-templates.toml".source}";
      };

      Service = {
        ExecStart = lib.getExe cfg.package;
        Restart = "on-failure";
      };

      Install.WantedBy = [ config.wayland.systemd.target ];
    };

    home.packages = lib.optional (cfg.package != null) cfg.package;

    xdg.configFile = {
      "crawlds/settings.json" = lib.mkIf (cfg.settings != { }) {
        source = generateJson "settings" cfg.settings;
      };
      "crawlds/colors.json" = lib.mkIf (cfg.colors != { }) {
        source = generateJson "colors" cfg.colors;
      };
      "crawlds/user-templates.toml" = lib.mkIf (cfg.user-templates != { }) {
        source =
          if lib.isString cfg.user-templates then
            pkgs.writeText "crawlds-user-templates.toml" cfg.user-templates
          else if builtins.isPath cfg.user-templates || lib.isStorePath cfg.user-templates then
            cfg.user-templates
          else
            tomlFormat.generate "crawlds-user-templates.toml" cfg.user-templates;
      };
    };

    assertions = [
      {
        assertion = !cfg.systemd.enable || cfg.package != null;
        message = "crawlds-shell: The package option must not be null when systemd service is enabled.";
      }
    ];
  };
}
