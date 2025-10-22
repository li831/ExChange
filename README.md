# 币安量化交易引擎

## 快速开始

### 环境要求
- Rust 1.75+
- 币安Testnet API密钥

### 构建
```bash
cargo build --release
```

### 配置
复制 `config.toml.example` 到 `config.local.toml` 并填入你的API密钥

### 运行
```bash
cargo run
```

## 开发

### 运行测试
```bash
cargo test
```

### 查看日志
```bash
RUST_LOG=debug cargo run
```

## 文档
详细文档见 `docs/` 目录
