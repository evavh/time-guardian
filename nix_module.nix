{
  config,
  lib,
  inputs,
  ...
}:

with lib;
with lib.types;
let
  cfg = config.services.time-guardian;
in
{
  options = {
    services.time-guardian = {
      enable = mkEnableOption "time-guardian";
      package = lib.mkOption {
		type = types.package;
	    default = inputs.time-guardian.packages.x86_64-linux.default;
	  };
    };
  };

  config = mkIf cfg.enable {
    systemd.services.time-guardian = {
      description = "Screen time control tool";
      after = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = ''
          ${lib.getExe cfg.package} run \
        '';
      };
    };
  };
}
