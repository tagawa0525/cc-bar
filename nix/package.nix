{ lib
, rustPlatform
, libcosmicAppHook
, makeWrapper
, jq
}:

rustPlatform.buildRustPackage {
  pname = "cc-bar";
  version = "0.1.0";

  src = lib.cleanSource ./..;

  cargoHash = "sha256-e9KwSB6PehYVEUms9aGeTCr2lcmfTF3HzwqNCZj4si4=";

  nativeBuildInputs = [
    libcosmicAppHook
    makeWrapper
  ];

  postInstall = ''
    # シェルスクリプトをインストール
    install -Dm755 scripts/cc-bar-relay.sh $out/bin/cc-bar-relay.sh
    install -Dm755 scripts/cc-bar-subagent-hook.sh $out/bin/cc-bar-subagent-hook.sh
    install -Dm755 scripts/cc-bar-session-cleanup.sh $out/bin/cc-bar-session-cleanup.sh

    # スクリプトが jq を確実に見つけられるよう PATH をラップ
    wrapProgram $out/bin/cc-bar-relay.sh --prefix PATH : ${lib.makeBinPath [ jq ]}
    wrapProgram $out/bin/cc-bar-subagent-hook.sh --prefix PATH : ${lib.makeBinPath [ jq ]}
    wrapProgram $out/bin/cc-bar-session-cleanup.sh --prefix PATH : ${lib.makeBinPath [ jq ]}

    # デスクトップファイルをインストール（COSMIC がアプレットとして認識）
    install -Dm644 data/com.github.tagawa.cc-bar.desktop $out/share/applications/com.github.tagawa.cc-bar.desktop
  '';

  meta = with lib; {
    description = "Claude Code Context Window Monitor for Cosmic DE";
    homepage = "https://github.com/tagawa0525/cc-bar";
    maintainers = [ ];
    platforms = platforms.linux;
    mainProgram = "cc-bar";
  };
}
