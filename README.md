# rust-playground

## スクリプト

```sh
# 開発実行
cargo watch -x run

# コードチェック
cargo check

# ビルドファイルの削除
cargo clean

# ドキュメントの生成
cargo run --open

# テスト
cargo test

#　DBマイグレーション
diesel migration run

# Linter
cargo clippy

# ソースコード修正
## 未コミット
cargo fix --allow-dirty
## Staging
cargo fix --allow-staged
```
