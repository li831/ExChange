#!/bin/bash
# 技术指标自动化验证脚本

set -e  # 遇到错误立即退出

echo "========================================"
echo "技术指标自动化验证"
echo "========================================"

# 获取脚本所在目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# 步骤1: 生成 Python 参考值
echo ""
echo "步骤 1/3: 生成 Python 参考值..."
echo "----------------------------------------"
python scripts/validate_indicators.py

if [ $? -ne 0 ]; then
    echo "❌ Python 参考值生成失败"
    exit 1
fi

# 步骤2: 运行 Rust 验证测试
echo ""
echo "步骤 2/3: 运行 Rust 验证测试..."
echo "----------------------------------------"
cd trading-engine
cargo test --test indicator_validation_test -- --nocapture

if [ $? -ne 0 ]; then
    echo "❌ Rust 验证测试失败"
    exit 1
fi

# 步骤3: 运行所有单元测试
echo ""
echo "步骤 3/3: 运行所有单元测试..."
echo "----------------------------------------"
cargo test

if [ $? -ne 0 ]; then
    echo "❌ 单元测试失败"
    exit 1
fi

# 总结
echo ""
echo "========================================"
echo "✅ 所有验证测试通过！"
echo "========================================"
echo ""
echo "测试总结:"
echo "  ✅ Python 参考值生成成功"
echo "  ✅ Rust vs Python 对比验证通过"
echo "  ✅ 所有单元测试通过"
echo ""
echo "下一步建议:"
echo "  - 查看测试报告: tests/fixtures/indicator_reference.json"
echo "  - 运行性能测试: cargo bench (如果配置了)"
echo "  - 查看测试覆盖率: cargo tarpaulin (需要安装)"
echo ""
