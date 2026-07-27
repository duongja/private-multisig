{
  description = "LP-0002 Private Multisig Basecamp QML UI";

  inputs = {
    logos-module-builder.url = "github:logos-co/logos-module-builder/2afd64405439d3fdda1ffb53852a9bb0049f0f0e";
  };

  outputs = inputs@{ logos-module-builder, ... }:
    logos-module-builder.lib.mkLogosQmlModule {
      src = ./.;
      configFile = ./metadata.json;
      flakeInputs = inputs;
    };
}
