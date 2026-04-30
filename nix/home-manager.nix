{
  config,
  lib,
  pkgs,
  self,
  version,
  ...
}:

let
  cfg = config.programs.blur-autoclicker;
  settingsFormat = pkgs.formats.json { };
in
{
  options.programs.blur-autoclicker = {
    enable = lib.mkEnableOption "Blur AutoClicker";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.blur-autoclicker;
      defaultText = lib.literalExpression "blur-autoclicker";
      description = "The blur-autoclicker package to use.";
    };

    settings = {
      theme = lib.mkOption {
        type = lib.types.enum [ "dark" "light" ];
        default = "dark";
      };

      clickSpeed = lib.mkOption {
        type = lib.types.float;
        default = 25.0;
      };

      clickInterval = lib.mkOption {
        type = lib.types.enum [ "s" "m" "h" ];
        default = "s";
      };

      mouseButton = lib.mkOption {
        type = lib.types.enum [ "Left" "Right" "Middle" ];
        default = "Left";
      };

      mode = lib.mkOption {
        type = lib.types.enum [ "Toggle" "Hold" ];
        default = "Toggle";
      };

      hotkey = lib.mkOption {
        type = lib.types.str;
        default = "ctrl+y";
      };

      dutyCycleEnabled = lib.mkOption {
        type = lib.types.bool;
        default = true;
      };

      dutyCycle = lib.mkOption {
        type = lib.types.float;
        default = 45.0;
      };

      speedVariationEnabled = lib.mkOption {
        type = lib.types.bool;
        default = true;
      };

      speedVariation = lib.mkOption {
        type = lib.types.float;
        default = 35.0;
      };

      doubleClickEnabled = lib.mkOption {
        type = lib.types.bool;
        default = false;
      };

      doubleClickDelay = lib.mkOption {
        type = lib.types.ints.unsigned;
        default = 40;
      };

      clickLimitEnabled = lib.mkOption {
        type = lib.types.bool;
        default = false;
      };

      clickLimit = lib.mkOption {
        type = lib.types.int;
        default = 1000;
      };

      timeLimitEnabled = lib.mkOption {
        type = lib.types.bool;
        default = false;
      };

      timeLimit = lib.mkOption {
        type = lib.types.float;
        default = 60.0;
      };

      timeLimitUnit = lib.mkOption {
        type = lib.types.enum [ "s" "m" "h" "d" ];
        default = "s";
      };

      cornerStopEnabled = lib.mkOption {
        type = lib.types.bool;
        default = true;
      };

      cornerStopTL = lib.mkOption {
        type = lib.types.int;
        default = 50;
      };

      cornerStopTR = lib.mkOption {
        type = lib.types.int;
        default = 50;
      };

      cornerStopBL = lib.mkOption {
        type = lib.types.int;
        default = 50;
      };

      cornerStopBR = lib.mkOption {
        type = lib.types.int;
        default = 50;
      };

      edgeStopEnabled = lib.mkOption {
        type = lib.types.bool;
        default = true;
      };

      edgeStopTop = lib.mkOption {
        type = lib.types.int;
        default = 40;
      };

      edgeStopRight = lib.mkOption {
        type = lib.types.int;
        default = 40;
      };

      edgeStopBottom = lib.mkOption {
        type = lib.types.int;
        default = 40;
      };

      edgeStopLeft = lib.mkOption {
        type = lib.types.int;
        default = 40;
      };

      positionEnabled = lib.mkOption {
        type = lib.types.bool;
        default = false;
      };

      positionX = lib.mkOption {
        type = lib.types.int;
        default = 0;
      };

      positionY = lib.mkOption {
        type = lib.types.int;
        default = 0;
      };

      disableScreenshots = lib.mkOption {
        type = lib.types.bool;
        default = false;
      };

      advancedSettingsEnabled = lib.mkOption {
        type = lib.types.bool;
        default = true;
      };

      explanationMode = lib.mkOption {
        type = lib.types.enum [ "text" "off" ];
        default = "text";
      };

      showStopReason = lib.mkOption {
        type = lib.types.bool;
        default = true;
      };

      showStopOverlay = lib.mkOption {
        type = lib.types.bool;
        default = true;
      };

      strictHotkeyModifiers = lib.mkOption {
        type = lib.types.bool;
        default = false;
      };
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.dataFile."BlurAutoClicker/settings.json".source =
      settingsFormat.generate "blur-autoclicker-settings" {
        Settings = {
          inherit version (cfg.settings)
            clickSpeed
            clickInterval
            mouseButton
            mode
            hotkey
            dutyCycleEnabled
            dutyCycle
            speedVariationEnabled
            speedVariation
            doubleClickEnabled
            doubleClickDelay
            clickLimitEnabled
            clickLimit
            timeLimitEnabled
            timeLimit
            timeLimitUnit
            cornerStopEnabled
            edgeStopEnabled
            edgeStopTop
            edgeStopRight
            edgeStopBottom
            edgeStopLeft
            positionEnabled
            positionX
            positionY
            disableScreenshots
            advancedSettingsEnabled
            explanationMode
            lastPanel
            showStopReason
            showStopOverlay
            strictHotkeyModifiers;

          cornerStopTL = cfg.settings.cornerStopTL;
          cornerStopTR = cfg.settings.cornerStopTR;
          cornerStopBL = cfg.settings.cornerStopBL;
          cornerStopBR = cfg.settings.cornerStopBR;
        };
      };
  };
}