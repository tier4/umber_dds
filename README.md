# T4RustDDS
DDSをrustで実装するプロジェクト

## RSRTPS
このプロジェクトの本体
DDSのRust実装

TODO:
- RTPSのパケットを受信
- ShapesDemoの受信側を実装

## RustDDS
参考にする既存実装

## rtps-rs
参考にする既存実装

## dds-docker
rtps解析のためのdocker環境

## ShapesDemo
既存実装の解析のためのrtpsを使ったデモ

## 既存実装の解析
dds-dockerでDockerコンテナを起動して
1. ShapeDemoのPublisher側を起動
- ShapesDemoデレクトリで`source ./install/setup.bash`
- `ShapeDemo`
- Publish
2. RustDDSのサブスクライバー側を起動
RustDDSディレクトリで`./target/debug/examples/shapes_demo -S -t Square`もしくは、デバッガ上で起動
```
$ gdb ./target/debug/examples/shapes_demo
(gdb) r -S -t Square
```


