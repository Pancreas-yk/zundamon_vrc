# ずんだもん VRC

VOICEVOX の音声合成を VRChat の仮想マイクとして使える Linux デスクトップアプリです。テキストを入力すると VOICEVOX で音声合成し、PulseAudio の仮想シンクを経由して VRChat にマイク入力として送信します。

## 必要な環境

- **Linux** (PulseAudio が動作する環境)
- **Rust** (ビルドに必要 / 1.70 以上推奨)
- **PulseAudio** (`pactl`, `paplay` コマンドが使えること)
- **VOICEVOX Engine** (音声合成サーバー)
- **Noto Sans CJK フォント** (日本語表示用 / 任意だが強く推奨)

## インストール

### 1. 依存パッケージの導入

#### Arch Linux / Manjaro

```bash
sudo pacman -S pulseaudio rust noto-fonts-cjk
```

#### Ubuntu / Debian

```bash
sudo apt install pulseaudio libpulse0 cargo fonts-noto-cjk
```

#### Fedora

```bash
sudo dnf install pulseaudio rust cargo google-noto-sans-cjk-fonts
```

### 2. VOICEVOX Engine の準備

VOICEVOX Engine をローカルで動作させる必要があります。以下のいずれかの方法で用意してください。

**方法A: 公式バイナリ**

[VOICEVOX 公式サイト](https://voicevox.hiroshiba.jp/) からダウンロードし、任意の場所に展開します。

**方法B: Docker**

```bash
docker pull voicevox/voicevox_engine:latest
docker run --rm -p 50021:50021 voicevox/voicevox_engine:latest
```

起動後、`http://127.0.0.1:50021` で API が利用可能になります。

### 3. ビルド

```bash
git clone <リポジトリURL>
cd zundamon_vrc
cargo build --release
```

ビルド成果物は `target/release/zundamon_vrc` に生成されます。

### 4. デスクトップエントリの登録 (任意)

アプリケーションランチャーから起動したい場合は、`.desktop` ファイルをコピーします。

```bash
# Exec= のパスを自分の環境に合わせて編集してください
cp zundamon_vrc.desktop ~/.local/share/applications/
```

## 起動方法

### 1. VOICEVOX Engine を起動

アプリ起動前に VOICEVOX Engine が動いている必要があります。手動で起動するか、アプリの設定で「アプリ起動時にVOICEVOXを自動起動」を有効にしてください。

```bash
# 例: Docker の場合
docker run --rm -p 50021:50021 voicevox/voicevox_engine:latest

# 例: バイナリの場合
/path/to/voicevox_engine/run
```

### 2. アプリを起動

```bash
# release ビルドを直接実行
./target/release/zundamon_vrc

# または cargo から
cargo run --release
```

### 3. 初回セットアップ

1. アプリが起動したら、ステータスバーで VOICEVOX の接続状態を確認します。「未接続」と表示されている場合は「起動」ボタンを押すか、設定タブから接続先 URL を確認してください。
2. 「設定」タブ → 「仮想デバイス」 → 「作成」をクリックして PulseAudio 仮想シンクを作成します。
3. 作成されると `ZundamonVRC.monitor` というマイクソースが利用可能になります。

### 4. VRChat 側の設定

VRChat のマイク設定で `ZundamonVRC.monitor`（または PulseAudio の設定で「Zundamon_VRC_Virtual_Mic」として表示されるモニターソース）を入力デバイスとして選択してください。

## 使い方

### テキスト入力

- テキストボックスにテキストを入力して **Enter** で送信（音声合成 → 再生）
- **Shift+Enter** で改行
- 「送信」ボタンでも送信できます

### テンプレート

よく使うフレーズをテンプレートとして登録できます。ボタンをクリックするだけで即座に音声を合成・送信します。デフォルトでは以下が登録されています:

- こんにちは！
- ありがとう！
- おつかれさまなのだ！
- 了解なのだ！

### 設定項目

| 項目 | 説明 |
|---|---|
| VOICEVOX URL | VOICEVOX Engine の接続先 (デフォルト: `http://127.0.0.1:50021`) |
| 実行パス | VOICEVOX のバイナリパスまたは起動コマンド |
| 自動起動 | アプリ起動時に VOICEVOX を自動的に起動する |
| スピーカー | 使用する音声キャラクター・スタイル (デフォルト: ずんだもん ノーマル) |
| 速度 / ピッチ / 抑揚 / 音量 | 音声合成パラメータの調整 |
| 仮想デバイス名 | PulseAudio に作成するシンクの名前 (デフォルト: `ZundamonVRC`) |

設定は `~/.config/zundamon_vrc/config.toml` に自動保存されます。

## トラブルシューティング

### VOICEVOX に接続できない

- VOICEVOX Engine が起動しているか確認してください: `curl http://127.0.0.1:50021/version`
- 設定タブで URL が正しいか確認してください

### 仮想デバイスが作成できない

- PulseAudio が動作しているか確認してください: `pactl info`
- PipeWire 環境の場合は `pipewire-pulse` がインストールされているか確認してください

### 日本語が表示されない

- Noto Sans CJK フォントがインストールされているか確認してください
- フォントは以下のパスのいずれかに配置される必要があります:
  - `/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc`
  - `/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc`
  - `/usr/share/fonts/noto-cjk-fonts/NotoSansCJK-Regular.ttc`
  - `/usr/share/fonts/google-noto-cjk/NotoSansCJK-Regular.ttc`

### VRChat でマイクとして認識されない

- アプリの設定タブで仮想デバイスを「作成」済みか確認してください
- VRChat のマイク設定で `ZundamonVRC.monitor` を選択してください
- PulseAudio のボリューム設定 (`pavucontrol`) で仮想デバイスがミュートされていないか確認してください

## ライセンス

VOICEVOX の利用規約に従ってください。各音声キャラクターにはそれぞれ利用規約があります。
