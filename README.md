# Haole | HavenMC Status CLI/TUI Tool

Haole は、Minecraft サーバー「HavenMC」の状態をターミナルから素早く、そして詳細に確認するための CLI/TUI ツールです。

## インストール

[Releases](https://github.com/KoHaRxnP/haole/releases) から対応するOSのバイナリをダウンロードしてパスの通った場所に配置、もしくは記載されたコマンドを実行してください。

## コマンド一覧

### haole author

Haoleの作者情報を取得します。

### haole <players|pl>

現在サーバーに接続しているプレイヤーのMCIDを取得してリストします。

### haole pq

現在サーバーに接続しているプレイヤーの人数を取得します。

### haole pall

```haole players```と```haole pq```の操作を一度で行えます。

### haole <is-online|isonline>

サーバーが現在オンラインかどうかを取得します。

### haole <is-offline|isoffline>

サーバーが現在オフラインかどうかを取得します。

### haole version

Haoleのバージョンを取得します。

### haole <server-version|sver>

サーバーのMinecraftバージョンを取得します。

### haole ip

サーバーのIPを取得します。

### haole host

サーバーのホストを取得します。

### haole <protocol|proto>

サーバーのプロトコルを取得します。

### haole port

サーバーのポート番号を取得します。

### haole motd <-raw|-clean|-html>

サーバーのMOTDを取得します。引数にraw/clean/html以外を指定した場合もしくは何も指定しなかった場合はデフォルトでcleanなMOTDを取得します。

### haole mode <cli|tui|toggle>

Haoleのモードを切り替えます。CLIモードのときはこのREADMEにあるコマンドを受け付けて実行し、TUIモードのときは```haole```を実行するとTUIが起動します。TUIは``Q```キーで終了します。

### haole update

Haoleを最新のバージョンにアップデートします。

### haole ping

サーバーのPingを疑似的に取得します。

### haole help

Haoleのコマンドヘルプを表示します。

## コマンドオプション一覧

### -w, --watch [&lt;SECONDS&gt;]

秒数が指定されている場合その秒数ごとにこのオプションをつけたコマンドを実行します。秒数が指定されていない場合デフォルトで5秒ごとに実行します。CLIモードで継続的に任意のデータを取得し続けることができます。```Q```キーで終了します。

### -h, --help

このオプションをつけたコマンドのヘルプを表示します。

### -v, --version

バージョン情報を表示します。
