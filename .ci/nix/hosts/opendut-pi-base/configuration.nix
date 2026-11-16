{
  pkgs,
  inputs,
  vars,
  ...
}:

{
  ########## NixOS ################################################################################

  system.stateVersion = "26.11"; # Change with great care!

  nixpkgs = {
    config.allowUnfree = true;
    overlays = [
      inputs.self.overlays.opendut
    ];
  };

  nix = {
    package = pkgs.nixVersions.latest;
    channel.enable = false; # remove nix-channel related tools & configs, as flakes are used for everything
    settings = {
      warn-dirty = false;
      experimental-features = [
        "flakes"
        "nix-command"
      ];
    };
    registry.nixpkgs.flake = inputs.nixpkgs;
    optimise.automatic = true;
    gc = {
      automatic = true;
      dates = "daily";
      options = "--delete-older-than 30d";
    };
  };

  ########## networking ###########################################################################

  networking = {
    hostName = vars.hostname;
    firewall.enable = true;
    useDHCP = true;
  };

  ########## services #############################################################################

  services = {
    openssh.enable = true;
    iperf3 = {
      enable = true;
      openFirewall = true;
    };
  };

  ########## localization #########################################################################

  time.timeZone = "Europe/Berlin";
  console.keyMap = "de";

  ########## users ################################################################################

  users = {
    mutableUsers = false;
    users =
      let
        sshAuthorizedKeys = [
          "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDAcXPmWMVEyT568uv/DlOrF61OGra2ZT1Xf8ZGlPY0wsUsMuF0HyrZPVOaVOoWldlqneA36ljP7gcZwFEMWRa0tRDPC9AYr7IGY3bIW8mde3/YsY2xGTr5g2XPwnjM8GUiuSuQWfMiLLM8vuZ61LnwV/2/OJyYTY34DiGRAYcqf+/WAXowZ5mo750NmAD+h8/vW+6O4X4H9WV8bzPAddhdTmHN8MCczfIV1sxoSj2xKXkQ94PN5/1PstU6Y/ZOLvSIoaW2p51X8GPDbJ43MTfW6eGhYuFqD/TWXseZUzDiDyfds7Bf7Tsy55Wlxd6chtLQhif1QwQww4e7A3DeTRZwIjQEnlaLso8jzlDYTS4RQgMRDJs+awBrvrCc6pMII4apmbWJW2BTgjofS0IH2jV13EOgYRsmyoOOc9RCq5xtciqBehn60a3mOQh4T75UFW6IHVZhOCtyjEeEQNjhe/zEgp//TwbonKrrifTyigdTxp4TKbjC+iy+fNrIwTB/Nl8yMgQYO5+FJGESWSyBvFNMXBj8sOnXLBdP4MgQ59FRr4dYJoPfb41+4rdEJb2pDjO97oleKkZ9c6u/VNLyK3BriNrkV3y2vXuB2ugP7H/aBe4SgAmN75vXQb+zzvAK42dj9ZaXF7JtCje+D2kZ3FYAa9ywrPL5oXsOLCIwH4lYYQ=="
          "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAkJANvlpdyVivRCYMcX1RfE6DUi8WNB6OTZdVuDNuod"
        ];
      in
      {
        nixos = {
          uid = 1000;
          isNormalUser = true;
          extraGroups = [ "wheel" ];
          home = "/home/nixos";
          hashedPassword = "$6$I0EFYeO.JFbxyVlg$o0NcSTYFEwD.ehAuWd0WNef4/Z75DU3X0kFUL13CV64AcuUqcNJ.HERURJVpitBSUOXG9xJvt1yDGLyTz0X34/";
          openssh.authorizedKeys.keys = sshAuthorizedKeys;
        };
        root = {
          openssh.authorizedKeys.keys = sshAuthorizedKeys;
        };
      };
  };

  ########## environment  #########################################################################

  environment.systemPackages = with pkgs; [
    ghostty.terminfo

    helix
    tmux
    zellij

    btop
    curl
    dnsutils
    fastfetch
    file
    gitMinimal
    jq
    ripgrep
    usbutils
    wget
    yq-go
  ];
}
