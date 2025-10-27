# Phase 2: Pine Script策略系统 实施计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现Pine Script DSL解析器，构建灵活的策略系统，支持Freqtrade策略转换

**Architecture:** Pest解析器，AST执行器，技术指标库，策略热加载，Freqtrade集成

**Tech Stack:** Rust 1.75+, Pest, Freqtrade, Python AST分析

---

## 📊 实施进度 (开始时间: 2025-01-27)

### 总体进度: 3/14 任务完成 (21%)

| Task | 状态 | 预计天数 | 测试数量 | Git Commit |
|------|------|----------|----------|------------|
| Task 1: DSL语法定义 | ✅ 已完成 | 2天 | - | 088256e |
| Task 2: 词法分析器 | ✅ 已完成 | 1.5天 | 6个 | e0de6ec |
| Task 3: AST生成器 | ✅ 已完成 | 1.5天 | 8个 | 5dc24cf |
| Task 4: 基础指标实现 | ⏳ 待开始 | 2天 | - | - |
| Task 5: 指标注册系统 | ⏳ 待开始 | 1天 | - | - |
| Task 6: 指标缓存优化 | ⏳ 待开始 | 1天 | - | - |
| Task 7: AST执行器 | ⏳ 待开始 | 2天 | - | - |
| Task 8: 策略函数实现 | ⏳ 待开始 | 1.5天 | - | - |
| Task 9: 内置函数库 | ⏳ 待开始 | 1.5天 | - | - |
| Task 10: Freqtrade环境 | ⏳ 待开始 | 0.5天 | - | - |
| Task 11: 策略转换器 | ⏳ 待开始 | 2天 | - | - |
| Task 12: 回测一致性验证 | ⏳ 待开始 | 1.5天 | - | - |
| Task 13: 策略热加载 | ⏳ 待开始 | 1天 | - | - |
| Task 14: 多策略管理 | ⏳ 待开始 | 1天 | - | - |

### 关键指标
- **目标测试数**: 100+ 单元测试 (当前: 14个)
- **DSL覆盖度**: Pine Script v5核心功能的60% (当前: 语法解析完成)
- **回测一致性**: 与Freqtrade信号一致性>95% (待实现)
- **性能目标**: 1000数据点指标计算<1ms (待测试)
- **开发方法**: TDD + 增量集成 ✅

---

## Task 1: Pine Script DSL语法定义

**Goal:** 定义支持的Pine Script语法子集，创建BNF规范

**Files:**
- Create: `trading-engine/docs/pine-script-spec.md`
- Create: `trading-engine/examples/strategies/`
- Create: `trading-engine/examples/strategies/simple_ma.pine`
- Create: `trading-engine/examples/strategies/bollinger_bands.pine`
- Create: `trading-engine/examples/strategies/rsi_strategy.pine`

### Step 1: 研究Pine Script v5核心语法

**分析内容:**
- 变量声明和赋值
- 技术指标函数
- 条件语句
- 策略函数
- 内置变量

**创建文件: `docs/pine-script-spec.md`**

```markdown
# Pine Script DSL 语法规范

## 1. 支持的语法元素

### 1.1 变量声明
```pine
// 简单赋值
length = 20
price = close

// 输入参数
length = input(20, "Period")
multiplier = input.float(2.0, "Multiplier", step=0.1)
```

### 1.2 技术指标函数
```pine
// 移动平均
sma_value = ta.sma(close, length)
ema_value = ta.ema(close, length)

// RSI
rsi_value = ta.rsi(close, 14)

// MACD
[macd_line, signal_line, histogram] = ta.macd(close, 12, 26, 9)

// 布林带
[middle, upper, lower] = ta.bb(close, 20, 2.0)
```

### 1.3 条件语句
```pine
if condition
    action()
else if other_condition
    other_action()
else
    default_action()
```

### 1.4 策略函数
```pine
strategy("My Strategy", overlay=true)
strategy.entry("Long", strategy.long)
strategy.close("Long")
strategy.exit("Stop Loss", "Long", stop=stop_price)
```

## 2. BNF语法规范

```bnf
<strategy> ::= <strategy_declaration> <statement_list>

<strategy_declaration> ::= "strategy" "(" <string> <parameters>? ")"

<statement_list> ::= <statement> | <statement> <statement_list>

<statement> ::= <assignment>
              | <if_statement>
              | <strategy_call>
              | <expression>

<assignment> ::= <identifier> "=" <expression>

<if_statement> ::= "if" <condition> <block> <else_clause>?

<else_clause> ::= "else" "if" <condition> <block> <else_clause>?
                | "else" <block>

<expression> ::= <literal>
               | <identifier>
               | <function_call>
               | <binary_operation>
               | <array_access>

<function_call> ::= <identifier> "(" <argument_list>? ")"
                  | <namespace> "." <identifier> "(" <argument_list>? ")"

<namespace> ::= "ta" | "strategy" | "input" | "math"

<binary_operation> ::= <expression> <operator> <expression>

<operator> ::= "+" | "-" | "*" | "/" | ">" | "<" | ">=" | "<=" | "==" | "!=" | "and" | "or"
```
```

### Step 2: 创建示例策略文件

**文件: `examples/strategies/simple_ma.pine`**

```pine
//@version=5
strategy("Simple Moving Average", overlay=true)

// 输入参数
fast_length = input(5, "Fast MA Period")
slow_length = input(20, "Slow MA Period")

// 计算移动平均
fast_ma = ta.sma(close, fast_length)
slow_ma = ta.sma(close, slow_length)

// 生成信号
long_condition = ta.crossover(fast_ma, slow_ma)
short_condition = ta.crossunder(fast_ma, slow_ma)

// 执行交易
if long_condition
    strategy.entry("Long", strategy.long)

if short_condition
    strategy.close("Long")
```

**文件: `examples/strategies/bollinger_bands.pine`**

```pine
//@version=5
strategy("Bollinger Bands Mean Reversion", overlay=true)

// 输入参数
bb_length = input(20, "BB Length")
bb_mult = input.float(2.0, "BB StdDev")
rsi_period = input(14, "RSI Period")
rsi_oversold = input(30, "RSI Oversold")
rsi_overbought = input(70, "RSI Overbought")

// 计算指标
[middle, upper, lower] = ta.bb(close, bb_length, bb_mult)
rsi = ta.rsi(close, rsi_period)

// 入场条件
long_entry = close < lower and rsi < rsi_oversold
short_entry = close > upper and rsi > rsi_overbought

// 出场条件
long_exit = close > middle
short_exit = close < middle

// 执行策略
if long_entry
    strategy.entry("BBLong", strategy.long)

if long_exit
    strategy.close("BBLong")

if short_entry
    strategy.entry("BBShort", strategy.short)

if short_exit
    strategy.close("BBShort")
```

### Step 3: 验证语法覆盖度

**检查清单:**
- [ ] 变量赋值
- [ ] 输入参数定义
- [ ] 技术指标调用
- [ ] 条件判断
- [ ] 策略入场/出场
- [ ] 数学运算
- [ ] 逻辑运算
- [ ] 比较运算

### Step 4: 提交语法规范

**命令:**
```bash
git add docs/pine-script-spec.md examples/strategies/
git commit -m "feat: define Pine Script DSL syntax specification"
```

---

## Task 2: 词法分析器实现 (Pest)

**Goal:** 使用Pest解析器生成器实现词法分析

**Files:**
- Create: `trading-engine/src/strategy/parser/mod.rs`
- Create: `trading-engine/src/strategy/parser/pine.pest`
- Create: `trading-engine/tests/parser_test.rs`
- Modify: `trading-engine/Cargo.toml`

### Step 1: 添加Pest依赖

**文件: `Cargo.toml`**

```toml
[dependencies]
# 解析器
pest = "2.7"
pest_derive = "2.7"

[dev-dependencies]
pretty_assertions = "1.4"
```

### Step 2: 编写Pest语法文件

**文件: `src/strategy/parser/pine.pest`**

```pest
// Pine Script Grammar

WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* }

// 主规则
strategy = { SOI ~ strategy_declaration ~ statement* ~ EOI }

strategy_declaration = {
    "strategy" ~ "(" ~ string ~ ("," ~ parameter)* ~ ")"
}

parameter = { identifier ~ "=" ~ value }

statement = {
    assignment
  | if_statement
  | strategy_call
  | expression
}

// 赋值语句
assignment = { identifier ~ "=" ~ expression }

// 条件语句
if_statement = {
    "if" ~ condition ~ block ~ else_clause?
}

else_clause = {
    "else" ~ "if" ~ condition ~ block ~ else_clause?
  | "else" ~ block
}

block = { statement+ | "{" ~ statement* ~ "}" }

condition = { expression }

// 策略调用
strategy_call = {
    "strategy" ~ "." ~ identifier ~ "(" ~ argument_list? ~ ")"
}

// 表达式
expression = { term ~ (binary_op ~ term)* }

term = {
    literal
  | function_call
  | identifier
  | array_destructure
  | "(" ~ expression ~ ")"
}

// 函数调用
function_call = {
    namespace ~ "." ~ identifier ~ "(" ~ argument_list? ~ ")"
  | identifier ~ "(" ~ argument_list? ~ ")"
}

namespace = { "ta" | "strategy" | "input" | "math" }

argument_list = { expression ~ ("," ~ expression)* }

// 数组解构
array_destructure = {
    "[" ~ identifier ~ ("," ~ identifier)* ~ "]"
}

// 运算符
binary_op = {
    comparison_op
  | arithmetic_op
  | logical_op
}

comparison_op = { ">=" | "<=" | ">" | "<" | "==" | "!=" }
arithmetic_op = { "+" | "-" | "*" | "/" | "%" }
logical_op = { "and" | "or" }

// 字面量
literal = {
    float_literal
  | integer_literal
  | boolean_literal
  | string
}

float_literal = @{ integer ~ "." ~ ASCII_DIGIT+ }
integer_literal = @{ "-"? ~ ASCII_DIGIT+ }
boolean_literal = { "true" | "false" }
string = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }

// 标识符
identifier = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }

// 内置变量
builtin_var = {
    "close" | "open" | "high" | "low" | "volume" | "time"
}

value = { literal | identifier }
```

### Step 3: 实现解析器模块

**文件: `src/strategy/parser/mod.rs`**

```rust
use pest::Parser;
use pest_derive::Parser;
use anyhow::{Result, Context};

#[derive(Parser)]
#[grammar = "strategy/parser/pine.pest"]
pub struct PineScriptParser;

pub fn parse_strategy(input: &str) -> Result<pest::iterators::Pairs<Rule>> {
    PineScriptParser::parse(Rule::strategy, input)
        .context("Failed to parse Pine Script strategy")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_assignment() {
        let input = "strategy(\"Test\")
length = 20";
        let result = parse_strategy(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function_call() {
        let input = "strategy(\"Test\")
sma_value = ta.sma(close, 20)";
        let result = parse_strategy(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_statement() {
        let input = "strategy(\"Test\")
if close > open
    strategy.entry(\"Long\", strategy.long)";
        let result = parse_strategy(input);
        assert!(result.is_ok());
    }
}
```

### Step 4: 编写解析器测试

**文件: `tests/parser_test.rs`**

```rust
use trading_engine::strategy::parser::parse_strategy;

#[test]
fn test_parse_complete_strategy() {
    let strategy = r#"
strategy("Dual MA Strategy", overlay=true)

fast_length = input(5, "Fast Period")
slow_length = input(20, "Slow Period")

fast_ma = ta.sma(close, fast_length)
slow_ma = ta.sma(close, slow_length)

if ta.crossover(fast_ma, slow_ma)
    strategy.entry("Long", strategy.long)

if ta.crossunder(fast_ma, slow_ma)
    strategy.close("Long")
"#;

    let result = parse_strategy(strategy);
    assert!(result.is_ok(), "Failed to parse complete strategy");
}

#[test]
fn test_parse_array_destructure() {
    let strategy = r#"
strategy("BB Strategy")
[middle, upper, lower] = ta.bb(close, 20, 2.0)
"#;

    let result = parse_strategy(strategy);
    assert!(result.is_ok(), "Failed to parse array destructuring");
}

#[test]
fn test_parse_nested_conditions() {
    let strategy = r#"
strategy("Complex Strategy")
if rsi < 30
    if volume > ta.sma(volume, 20)
        strategy.entry("Long", strategy.long)
    else
        strategy.close("Long")
"#;

    let result = parse_strategy(strategy);
    assert!(result.is_ok(), "Failed to parse nested conditions");
}
```

### Step 5: 运行测试验证解析器

**命令:**
```bash
cargo test test_parse
```

**期望输出:**
```
test tests::parser_test::test_parse_complete_strategy ... ok
test tests::parser_test::test_parse_array_destructure ... ok
test tests::parser_test::test_parse_nested_conditions ... ok
```

### Step 6: 提交词法分析器

**命令:**
```bash
git add src/strategy/parser/ tests/parser_test.rs Cargo.toml
git commit -m "feat: implement Pine Script lexer using Pest parser"
```

---

## Task 3: AST (抽象语法树) 生成器

**Goal:** 将Pest解析结果转换为类型化的AST

**Files:**
- Create: `trading-engine/src/strategy/ast.rs`
- Create: `trading-engine/src/strategy/ast_builder.rs`
- Create: `tests/ast_test.rs`
- Modify: `src/strategy/mod.rs`

### Step 1: 定义AST节点类型

**文件: `src/strategy/ast.rs`**

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Strategy {
    pub name: String,
    pub parameters: HashMap<String, Value>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        target: AssignmentTarget,
        value: Expression,
    },
    If {
        condition: Expression,
        then_block: Vec<Statement>,
        else_block: Option<Vec<Statement>>,
    },
    StrategyCall {
        function: String,
        arguments: Vec<Expression>,
    },
    Expression(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentTarget {
    Variable(String),
    ArrayDestructure(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    FunctionCall {
        namespace: Option<String>,
        name: String,
        arguments: Vec<Expression>,
    },
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    ArrayAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // 算术
    Add, Subtract, Multiply, Divide, Modulo,
    // 比较
    Greater, Less, GreaterEqual, LessEqual, Equal, NotEqual,
    // 逻辑
    And, Or,
}

impl BinaryOperator {
    pub fn from_str(s: &str) -> Option<Self> {
        use BinaryOperator::*;
        match s {
            "+" => Some(Add),
            "-" => Some(Subtract),
            "*" => Some(Multiply),
            "/" => Some(Divide),
            "%" => Some(Modulo),
            ">" => Some(Greater),
            "<" => Some(Less),
            ">=" => Some(GreaterEqual),
            "<=" => Some(LessEqual),
            "==" => Some(Equal),
            "!=" => Some(NotEqual),
            "and" => Some(And),
            "or" => Some(Or),
            _ => None,
        }
    }
}
```

### Step 2: 实现AST构建器

**文件: `src/strategy/ast_builder.rs`**

```rust
use super::ast::*;
use super::parser::{PineScriptParser, Rule};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use anyhow::{Result, anyhow, Context};

pub struct ASTBuilder;

impl ASTBuilder {
    pub fn build(input: &str) -> Result<Strategy> {
        let pairs = PineScriptParser::parse(Rule::strategy, input)
            .context("Failed to parse Pine Script")?;

        let strategy_pair = pairs.into_iter().next()
            .ok_or_else(|| anyhow!("No strategy found"))?;

        Self::build_strategy(strategy_pair)
    }

    fn build_strategy(pair: Pair<Rule>) -> Result<Strategy> {
        let mut name = String::new();
        let mut parameters = HashMap::new();
        let mut statements = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::strategy_declaration => {
                    let (n, p) = Self::build_strategy_declaration(inner)?;
                    name = n;
                    parameters = p;
                }
                Rule::statement => {
                    statements.push(Self::build_statement(inner)?);
                }
                _ => {}
            }
        }

        Ok(Strategy { name, parameters, statements })
    }

    fn build_strategy_declaration(pair: Pair<Rule>) -> Result<(String, HashMap<String, Value>)> {
        let mut inner = pair.into_inner();
        let name = inner.next()
            .and_then(|p| Self::extract_string(p.as_str()))
            .ok_or_else(|| anyhow!("Strategy name not found"))?;

        let mut params = HashMap::new();
        for param_pair in inner {
            if param_pair.as_rule() == Rule::parameter {
                let (key, value) = Self::build_parameter(param_pair)?;
                params.insert(key, value);
            }
        }

        Ok((name, params))
    }

    fn build_statement(pair: Pair<Rule>) -> Result<Statement> {
        let inner = pair.into_inner().next()
            .ok_or_else(|| anyhow!("Empty statement"))?;

        match inner.as_rule() {
            Rule::assignment => Self::build_assignment(inner),
            Rule::if_statement => Self::build_if_statement(inner),
            Rule::strategy_call => Self::build_strategy_call(inner),
            Rule::expression => Ok(Statement::Expression(Self::build_expression(inner)?)),
            _ => Err(anyhow!("Unknown statement type: {:?}", inner.as_rule()))
        }
    }

    fn build_assignment(pair: Pair<Rule>) -> Result<Statement> {
        let mut inner = pair.into_inner();

        let target_pair = inner.next()
            .ok_or_else(|| anyhow!("Assignment target not found"))?;

        let target = if target_pair.as_rule() == Rule::array_destructure {
            let vars = Self::build_array_destructure(target_pair)?;
            AssignmentTarget::ArrayDestructure(vars)
        } else {
            AssignmentTarget::Variable(target_pair.as_str().to_string())
        };

        let value = inner.next()
            .and_then(|p| Self::build_expression(p).ok())
            .ok_or_else(|| anyhow!("Assignment value not found"))?;

        Ok(Statement::Assignment { target, value })
    }

    fn build_expression(pair: Pair<Rule>) -> Result<Expression> {
        let inner = pair.into_inner();
        let mut parts: Vec<Pair<Rule>> = inner.collect();

        if parts.is_empty() {
            return Err(anyhow!("Empty expression"));
        }

        // 处理二元运算
        if parts.len() >= 3 {
            let left = Self::build_term(parts.remove(0))?;
            let mut current = left;

            while parts.len() >= 2 {
                let op_str = parts.remove(0).as_str();
                let right = Self::build_term(parts.remove(0))?;

                if let Some(op) = BinaryOperator::from_str(op_str) {
                    current = Expression::BinaryOp {
                        left: Box::new(current),
                        op,
                        right: Box::new(right),
                    };
                }
            }

            return Ok(current);
        }

        // 单个term
        Self::build_term(parts.into_iter().next().unwrap())
    }

    fn build_term(pair: Pair<Rule>) -> Result<Expression> {
        let inner = pair.into_inner().next();

        if inner.is_none() {
            // 可能是标识符或字面量
            return match pair.as_rule() {
                Rule::identifier => Ok(Expression::Variable(pair.as_str().to_string())),
                Rule::literal => Ok(Expression::Literal(Self::build_literal(pair)?)),
                _ => Err(anyhow!("Unknown term: {:?}", pair.as_rule()))
            };
        }

        let inner = inner.unwrap();
        match inner.as_rule() {
            Rule::literal => Ok(Expression::Literal(Self::build_literal(inner)?)),
            Rule::identifier => Ok(Expression::Variable(inner.as_str().to_string())),
            Rule::function_call => Self::build_function_call(inner),
            Rule::expression => Self::build_expression(inner),
            _ => Err(anyhow!("Unknown term type: {:?}", inner.as_rule()))
        }
    }

    fn build_function_call(pair: Pair<Rule>) -> Result<Expression> {
        let mut namespace = None;
        let mut name = String::new();
        let mut arguments = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::namespace => namespace = Some(inner.as_str().to_string()),
                Rule::identifier => name = inner.as_str().to_string(),
                Rule::argument_list => {
                    for arg in inner.into_inner() {
                        arguments.push(Self::build_expression(arg)?);
                    }
                }
                _ => {}
            }
        }

        Ok(Expression::FunctionCall { namespace, name, arguments })
    }

    fn build_literal(pair: Pair<Rule>) -> Result<Value> {
        let inner = pair.into_inner().next()
            .ok_or_else(|| anyhow!("Empty literal"))?;

        match inner.as_rule() {
            Rule::integer_literal => {
                let val = inner.as_str().parse::<i64>()
                    .context("Failed to parse integer")?;
                Ok(Value::Integer(val))
            }
            Rule::float_literal => {
                let val = inner.as_str().parse::<f64>()
                    .context("Failed to parse float")?;
                Ok(Value::Float(val))
            }
            Rule::boolean_literal => {
                Ok(Value::Boolean(inner.as_str() == "true"))
            }
            Rule::string => {
                Ok(Value::String(Self::extract_string(inner.as_str())
                    .unwrap_or_default()))
            }
            _ => Err(anyhow!("Unknown literal type: {:?}", inner.as_rule()))
        }
    }

    fn extract_string(s: &str) -> Option<String> {
        if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
            Some(s[1..s.len()-1].to_string())
        } else {
            None
        }
    }

    fn build_array_destructure(pair: Pair<Rule>) -> Result<Vec<String>> {
        let mut vars = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::identifier {
                vars.push(inner.as_str().to_string());
            }
        }
        Ok(vars)
    }

    fn build_if_statement(pair: Pair<Rule>) -> Result<Statement> {
        let mut inner = pair.into_inner();

        let condition = inner.next()
            .and_then(|p| Self::build_expression(p).ok())
            .ok_or_else(|| anyhow!("If condition not found"))?;

        let then_block = inner.next()
            .map(|p| Self::build_block(p))
            .unwrap_or_else(|| Ok(Vec::new()))?;

        let else_block = inner.next()
            .and_then(|p| {
                if p.as_rule() == Rule::else_clause {
                    Some(Self::build_else_clause(p).ok()?)
                } else {
                    None
                }
            });

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn build_block(pair: Pair<Rule>) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::statement {
                statements.push(Self::build_statement(inner)?);
            }
        }
        Ok(statements)
    }

    fn build_else_clause(pair: Pair<Rule>) -> Result<Vec<Statement>> {
        for inner in pair.into_inner() {
            if inner.as_rule() == Rule::block {
                return Self::build_block(inner);
            }
        }
        Ok(Vec::new())
    }

    fn build_strategy_call(pair: Pair<Rule>) -> Result<Statement> {
        let mut function = String::new();
        let mut arguments = Vec::new();

        for inner in pair.into_inner() {
            match inner.as_rule() {
                Rule::identifier => function = inner.as_str().to_string(),
                Rule::argument_list => {
                    for arg in inner.into_inner() {
                        arguments.push(Self::build_expression(arg)?);
                    }
                }
                _ => {}
            }
        }

        Ok(Statement::StrategyCall { function, arguments })
    }

    fn build_parameter(pair: Pair<Rule>) -> Result<(String, Value)> {
        let mut inner = pair.into_inner();
        let key = inner.next()
            .map(|p| p.as_str().to_string())
            .ok_or_else(|| anyhow!("Parameter key not found"))?;
        let value = inner.next()
            .and_then(|p| Self::build_value(p).ok())
            .ok_or_else(|| anyhow!("Parameter value not found"))?;
        Ok((key, value))
    }

    fn build_value(pair: Pair<Rule>) -> Result<Value> {
        match pair.as_rule() {
            Rule::literal => Self::build_literal(pair),
            Rule::identifier => Ok(Value::String(pair.as_str().to_string())),
            _ => Err(anyhow!("Unknown value type: {:?}", pair.as_rule()))
        }
    }
}
```

### Step 3: 编写AST测试

**文件: `tests/ast_test.rs`**

```rust
use trading_engine::strategy::ast_builder::ASTBuilder;
use trading_engine::strategy::ast::*;

#[test]
fn test_ast_simple_strategy() {
    let input = r#"
strategy("Test Strategy", overlay=true)
length = 20
sma_val = ta.sma(close, length)
"#;

    let ast = ASTBuilder::build(input).expect("Failed to build AST");

    assert_eq!(ast.name, "Test Strategy");
    assert_eq!(ast.parameters.get("overlay"), Some(&Value::Boolean(true)));
    assert_eq!(ast.statements.len(), 2);

    // 验证第一个赋值语句
    match &ast.statements[0] {
        Statement::Assignment { target, value } => {
            assert_eq!(target, &AssignmentTarget::Variable("length".to_string()));
            assert_eq!(value, &Expression::Literal(Value::Integer(20)));
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_ast_function_call() {
    let input = r#"
strategy("Test")
sma_val = ta.sma(close, 20)
"#;

    let ast = ASTBuilder::build(input).expect("Failed to build AST");

    match &ast.statements[0] {
        Statement::Assignment { target: _, value } => {
            match value {
                Expression::FunctionCall { namespace, name, arguments } => {
                    assert_eq!(namespace, &Some("ta".to_string()));
                    assert_eq!(name, "sma");
                    assert_eq!(arguments.len(), 2);
                }
                _ => panic!("Expected function call expression"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}

#[test]
fn test_ast_if_statement() {
    let input = r#"
strategy("Test")
if close > open
    strategy.entry("Long", strategy.long)
else
    strategy.close("Long")
"#;

    let ast = ASTBuilder::build(input).expect("Failed to build AST");

    match &ast.statements[0] {
        Statement::If { condition, then_block, else_block } => {
            // 验证条件
            match condition {
                Expression::BinaryOp { op, .. } => {
                    assert_eq!(op, &BinaryOperator::Greater);
                }
                _ => panic!("Expected binary operation in condition"),
            }

            assert_eq!(then_block.len(), 1);
            assert!(else_block.is_some());
            assert_eq!(else_block.as_ref().unwrap().len(), 1);
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_ast_array_destructure() {
    let input = r#"
strategy("Test")
[middle, upper, lower] = ta.bb(close, 20, 2.0)
"#;

    let ast = ASTBuilder::build(input).expect("Failed to build AST");

    match &ast.statements[0] {
        Statement::Assignment { target, .. } => {
            match target {
                AssignmentTarget::ArrayDestructure(vars) => {
                    assert_eq!(vars, &vec![
                        "middle".to_string(),
                        "upper".to_string(),
                        "lower".to_string()
                    ]);
                }
                _ => panic!("Expected array destructure"),
            }
        }
        _ => panic!("Expected assignment statement"),
    }
}
```

### Step 4: 运行AST测试

**命令:**
```bash
cargo test test_ast
```

**期望输出:**
```
test tests::ast_test::test_ast_simple_strategy ... ok
test tests::ast_test::test_ast_function_call ... ok
test tests::ast_test::test_ast_if_statement ... ok
test tests::ast_test::test_ast_array_destructure ... ok
```

### Step 5: 提交AST生成器

**命令:**
```bash
git add src/strategy/ast.rs src/strategy/ast_builder.rs tests/ast_test.rs
git commit -m "feat: implement AST builder for Pine Script DSL"
```

---

## Task 4: 基础技术指标实现

**Goal:** 实现Pine Script中常用的技术指标

**Files:**
- Create: `trading-engine/src/indicators/mod.rs` (扩展)
- Create: `trading-engine/src/indicators/ma.rs`
- Create: `trading-engine/src/indicators/oscillators.rs`
- Create: `trading-engine/src/indicators/volatility.rs`
- Create: `tests/indicators_extended_test.rs`

### Step 1: 实现移动平均指标

**文件: `src/indicators/ma.rs`**

```rust
/// Simple Moving Average with epsilon handling
pub fn sma(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(data.len() - period + 1);

    for i in period - 1..data.len() {
        let sum: f64 = data[i - period + 1..=i].iter().sum();
        result.push(sum / period as f64);
    }

    result
}

/// Exponential Moving Average
pub fn ema(data: &[f64], period: usize) -> Vec<f64> {
    if data.is_empty() || period == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(data.len());
    let multiplier = 2.0 / (period as f64 + 1.0);

    // 第一个EMA值使用SMA
    if data.len() >= period {
        let first_sma: f64 = data[..period].iter().sum::<f64>() / period as f64;
        result.push(first_sma);

        for i in period..data.len() {
            let ema_val = (data[i] - result.last().unwrap()) * multiplier
                        + result.last().unwrap();
            result.push(ema_val);
        }
    }

    result
}

/// Weighted Moving Average
pub fn wma(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period || period == 0 {
        return vec![];
    }

    let mut result = Vec::with_capacity(data.len() - period + 1);
    let weight_sum: f64 = (1..=period).sum::<usize>() as f64;

    for i in period - 1..data.len() {
        let mut weighted_sum = 0.0;
        for j in 0..period {
            weighted_sum += data[i - period + 1 + j] * (j + 1) as f64;
        }
        result.push(weighted_sum / weight_sum);
    }

    result
}

/// Volume Weighted Average Price
pub fn vwap(prices: &[f64], volumes: &[f64], period: usize) -> Vec<f64> {
    if prices.len() != volumes.len() || prices.len() < period {
        return vec![];
    }

    let mut result = Vec::with_capacity(prices.len() - period + 1);

    for i in period - 1..prices.len() {
        let mut price_volume_sum = 0.0;
        let mut volume_sum = 0.0;

        for j in i - period + 1..=i {
            price_volume_sum += prices[j] * volumes[j];
            volume_sum += volumes[j];
        }

        if volume_sum > 0.0 {
            result.push(price_volume_sum / volume_sum);
        } else {
            result.push(prices[i]); // fallback to current price
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = sma(&data, 3);
        assert_eq!(sma, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_ema_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ema = ema(&data, 3);
        assert!(ema.len() == 3);
        assert!((ema[0] - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_wma_calculation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let wma = wma(&data, 3);
        assert_eq!(wma.len(), 3);
        // WMA gives more weight to recent values
        assert!(wma[0] > 2.0); // Should be > simple average
    }
}
```

### Step 2: 实现震荡指标

**文件: `src/indicators/oscillators.rs`**

```rust
use super::ma::sma;

/// Relative Strength Index with Wilder's smoothing
pub fn rsi(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period + 1 {
        return vec![];
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    // 计算价格变化
    for i in 1..data.len() {
        let change = data[i] - data[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(change.abs());
        }
    }

    let mut result = Vec::new();

    // 第一个RSI使用简单平均
    let avg_gain: f64 = gains[..period].iter().sum::<f64>() / period as f64;
    let avg_loss: f64 = losses[..period].iter().sum::<f64>() / period as f64;

    let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
    result.push(100.0 - (100.0 / (1.0 + rs)));

    // 后续RSI使用Wilder's平滑
    let mut smoothed_gain = avg_gain;
    let mut smoothed_loss = avg_loss;

    for i in period..gains.len() {
        smoothed_gain = (smoothed_gain * (period - 1) as f64 + gains[i]) / period as f64;
        smoothed_loss = (smoothed_loss * (period - 1) as f64 + losses[i]) / period as f64;

        let rs = if smoothed_loss == 0.0 { 100.0 } else { smoothed_gain / smoothed_loss };
        result.push(100.0 - (100.0 / (1.0 + rs)));
    }

    result
}

/// Moving Average Convergence Divergence
pub fn macd(data: &[f64], fast: usize, slow: usize, signal: usize)
    -> (Vec<f64>, Vec<f64>, Vec<f64>) {

    if data.len() < slow {
        return (vec![], vec![], vec![]);
    }

    let ema_fast = super::ma::ema(data, fast);
    let ema_slow = super::ma::ema(data, slow);

    // Calculate MACD line
    let mut macd_line = Vec::new();
    let offset = slow - fast;
    for i in 0..ema_slow.len() {
        if i + offset < ema_fast.len() {
            macd_line.push(ema_fast[i + offset] - ema_slow[i]);
        }
    }

    // Calculate signal line
    let signal_line = super::ma::ema(&macd_line, signal);

    // Calculate histogram
    let mut histogram = Vec::new();
    let offset = signal - 1;
    for i in 0..signal_line.len() {
        if i + offset < macd_line.len() {
            histogram.push(macd_line[i + offset] - signal_line[i]);
        }
    }

    (macd_line, signal_line, histogram)
}

/// Stochastic Oscillator
pub fn stochastic(high: &[f64], low: &[f64], close: &[f64], period: usize, smooth_k: usize, smooth_d: usize)
    -> (Vec<f64>, Vec<f64>) {

    if high.len() != low.len() || high.len() != close.len() || high.len() < period {
        return (vec![], vec![]);
    }

    let mut raw_k = Vec::new();

    for i in period - 1..close.len() {
        let highest = high[i - period + 1..=i].iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let lowest = low[i - period + 1..=i].iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));

        let range = highest - lowest;
        if range > 0.0 {
            raw_k.push(100.0 * (close[i] - lowest) / range);
        } else {
            raw_k.push(50.0); // Neutral value when no range
        }
    }

    // Smooth %K
    let k = sma(&raw_k, smooth_k);

    // Calculate %D
    let d = sma(&k, smooth_d);

    (k, d)
}

/// Commodity Channel Index
pub fn cci(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    if high.len() != low.len() || high.len() != close.len() || high.len() < period {
        return vec![];
    }

    // Calculate Typical Price
    let mut typical_price = Vec::with_capacity(high.len());
    for i in 0..high.len() {
        typical_price.push((high[i] + low[i] + close[i]) / 3.0);
    }

    let sma_tp = sma(&typical_price, period);
    let mut result = Vec::new();

    for i in 0..sma_tp.len() {
        let start = i + period - 1 - (period - 1);
        let end = i + period - 1 + 1;

        // Calculate Mean Deviation
        let mut sum_deviation = 0.0;
        for j in start..end.min(typical_price.len()) {
            sum_deviation += (typical_price[j] - sma_tp[i]).abs();
        }
        let mean_deviation = sum_deviation / period as f64;

        if mean_deviation > 0.0 {
            let cci_value = (typical_price[i + period - 1] - sma_tp[i]) / (0.015 * mean_deviation);
            result.push(cci_value);
        } else {
            result.push(0.0);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_calculation() {
        let data = vec![
            44.0, 44.25, 44.38, 44.5, 44.0,
            43.75, 44.25, 44.5, 44.75, 45.0,
            45.25, 45.5, 45.75, 46.0, 46.25
        ];

        let rsi_values = rsi(&data, 14);
        assert!(rsi_values.len() > 0);

        // RSI should be between 0 and 100
        for val in &rsi_values {
            assert!(*val >= 0.0 && *val <= 100.0);
        }
    }

    #[test]
    fn test_macd_calculation() {
        let data = vec![
            100.0, 101.0, 102.0, 101.5, 100.5,
            99.5, 100.0, 101.0, 102.5, 103.0,
            103.5, 104.0, 103.5, 103.0, 102.5,
            102.0, 102.5, 103.0, 103.5, 104.0,
            104.5, 105.0, 105.5, 106.0, 106.5
        ];

        let (macd_line, signal_line, histogram) = macd(&data, 12, 26, 9);

        assert!(macd_line.len() > 0);
        assert!(signal_line.len() > 0);
        assert!(histogram.len() > 0);
    }

    #[test]
    fn test_stochastic_calculation() {
        let high = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 14.0, 13.0, 12.0, 11.0];
        let low = vec![8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 12.0, 11.0, 10.0, 9.0];
        let close = vec![9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 13.0, 12.0, 11.0, 10.0];

        let (k, d) = stochastic(&high, &low, &close, 5, 3, 3);

        assert!(k.len() > 0);
        assert!(d.len() > 0);

        // Stochastic values should be between 0 and 100
        for val in &k {
            assert!(*val >= 0.0 && *val <= 100.0);
        }
    }
}
```

### Step 3: 实现波动率指标

**文件: `src/indicators/volatility.rs`**

```rust
use super::ma::sma;

/// Bollinger Bands
pub fn bollinger_bands(data: &[f64], period: usize, std_dev_multiplier: f64)
    -> (Vec<f64>, Vec<f64>, Vec<f64>) {

    if data.len() < period {
        return (vec![], vec![], vec![]);
    }

    let middle = sma(data, period);
    let mut upper = Vec::with_capacity(middle.len());
    let mut lower = Vec::with_capacity(middle.len());

    for i in 0..middle.len() {
        // Calculate standard deviation
        let start = i + period - 1 - (period - 1);
        let end = i + period - 1 + 1;

        let slice = &data[start..end.min(data.len())];
        let mean = middle[i];

        let variance: f64 = slice.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / period as f64;

        let std_dev = variance.sqrt();

        upper.push(mean + std_dev_multiplier * std_dev);
        lower.push(mean - std_dev_multiplier * std_dev);
    }

    (middle, upper, lower)
}

/// Average True Range
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<f64> {
    if high.len() != low.len() || high.len() != close.len() || high.len() < period + 1 {
        return vec![];
    }

    let mut true_ranges = Vec::with_capacity(high.len() - 1);

    // Calculate True Range
    for i in 1..high.len() {
        let high_low = high[i] - low[i];
        let high_close = (high[i] - close[i - 1]).abs();
        let low_close = (low[i] - close[i - 1]).abs();

        true_ranges.push(high_low.max(high_close).max(low_close));
    }

    // Calculate ATR using Wilder's smoothing
    let mut atr_values = Vec::new();

    // First ATR is simple average
    let first_atr: f64 = true_ranges[..period].iter().sum::<f64>() / period as f64;
    atr_values.push(first_atr);

    // Subsequent ATR values use smoothing
    let mut current_atr = first_atr;
    for i in period..true_ranges.len() {
        current_atr = ((current_atr * (period - 1) as f64) + true_ranges[i]) / period as f64;
        atr_values.push(current_atr);
    }

    atr_values
}

/// Keltner Channels
pub fn keltner_channels(high: &[f64], low: &[f64], close: &[f64],
                        ema_period: usize, atr_period: usize, multiplier: f64)
    -> (Vec<f64>, Vec<f64>, Vec<f64>) {

    let middle = super::ma::ema(close, ema_period);
    let atr_values = atr(high, low, close, atr_period);

    if middle.is_empty() || atr_values.is_empty() {
        return (vec![], vec![], vec![]);
    }

    let min_len = middle.len().min(atr_values.len());
    let mut upper = Vec::with_capacity(min_len);
    let mut lower = Vec::with_capacity(min_len);

    for i in 0..min_len {
        upper.push(middle[i] + multiplier * atr_values[i]);
        lower.push(middle[i] - multiplier * atr_values[i]);
    }

    (middle[..min_len].to_vec(), upper, lower)
}

/// Standard Deviation
pub fn std_dev(data: &[f64], period: usize) -> Vec<f64> {
    if data.len() < period {
        return vec![];
    }

    let sma_values = sma(data, period);
    let mut result = Vec::with_capacity(sma_values.len());

    for i in 0..sma_values.len() {
        let start = i;
        let end = i + period;

        let slice = &data[start..end.min(data.len())];
        let mean = sma_values[i];

        let variance: f64 = slice.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / period as f64;

        result.push(variance.sqrt());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands() {
        let data = vec![
            100.0, 101.0, 102.0, 101.5, 100.5,
            99.5, 100.0, 101.0, 102.5, 103.0
        ];

        let (middle, upper, lower) = bollinger_bands(&data, 5, 2.0);

        assert_eq!(middle.len(), upper.len());
        assert_eq!(middle.len(), lower.len());

        // Upper band should be above middle, lower below
        for i in 0..middle.len() {
            assert!(upper[i] > middle[i]);
            assert!(lower[i] < middle[i]);
        }
    }

    #[test]
    fn test_atr_calculation() {
        let high = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0];
        let low = vec![8.0, 9.0, 10.0, 11.0, 12.0, 13.0];
        let close = vec![9.0, 10.0, 11.0, 12.0, 13.0, 14.0];

        let atr_values = atr(&high, &low, &close, 3);

        assert!(atr_values.len() > 0);

        // ATR should always be positive
        for val in &atr_values {
            assert!(*val > 0.0);
        }
    }

    #[test]
    fn test_keltner_channels() {
        let high = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 14.0, 13.0];
        let low = vec![8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 12.0, 11.0];
        let close = vec![9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 13.0, 12.0];

        let (middle, upper, lower) = keltner_channels(&high, &low, &close, 3, 3, 2.0);

        assert!(middle.len() > 0);
        assert_eq!(middle.len(), upper.len());
        assert_eq!(middle.len(), lower.len());

        // Channels should be properly ordered
        for i in 0..middle.len() {
            assert!(upper[i] > middle[i]);
            assert!(lower[i] < middle[i]);
        }
    }
}
```

### Step 4: 更新indicators模块

**文件: `src/indicators/mod.rs`** (扩展现有文件)

```rust
pub mod ma;
pub mod oscillators;
pub mod volatility;

// Re-export commonly used functions
pub use ma::{sma, ema, wma, vwap};
pub use oscillators::{rsi, macd, stochastic, cci};
pub use volatility::{bollinger_bands, atr, keltner_channels, std_dev};

// Keep existing epsilon and crossover functions
pub const EPSILON: f64 = 1e-8;

/// 检测向上交叉
pub fn is_crossover(fast_prev: &f64, fast_curr: &f64, slow_prev: &f64, slow_curr: &f64) -> bool {
    fast_prev <= slow_prev && (fast_curr - slow_curr) > EPSILON
}

/// 检测向下交叉
pub fn is_crossunder(fast_prev: &f64, fast_curr: &f64, slow_prev: &f64, slow_curr: &f64) -> bool {
    fast_prev >= slow_prev && (slow_curr - fast_curr) > EPSILON
}

/// Crossover detection for vectors
pub fn crossover(series1: &[f64], series2: &[f64]) -> Vec<bool> {
    if series1.len() != series2.len() || series1.len() < 2 {
        return vec![];
    }

    let mut result = vec![false]; // First element has no previous

    for i in 1..series1.len() {
        result.push(is_crossover(
            &series1[i-1], &series1[i],
            &series2[i-1], &series2[i]
        ));
    }

    result
}

/// Crossunder detection for vectors
pub fn crossunder(series1: &[f64], series2: &[f64]) -> Vec<bool> {
    if series1.len() != series2.len() || series1.len() < 2 {
        return vec![];
    }

    let mut result = vec![false]; // First element has no previous

    for i in 1..series1.len() {
        result.push(is_crossunder(
            &series1[i-1], &series1[i],
            &series2[i-1], &series2[i]
        ));
    }

    result
}
```

### Step 5: 运行所有指标测试

**命令:**
```bash
cargo test indicators
```

**期望输出:**
```
test indicators::ma::tests::test_sma_calculation ... ok
test indicators::ma::tests::test_ema_calculation ... ok
test indicators::ma::tests::test_wma_calculation ... ok
test indicators::oscillators::tests::test_rsi_calculation ... ok
test indicators::oscillators::tests::test_macd_calculation ... ok
test indicators::oscillators::tests::test_stochastic_calculation ... ok
test indicators::volatility::tests::test_bollinger_bands ... ok
test indicators::volatility::tests::test_atr_calculation ... ok
test indicators::volatility::tests::test_keltner_channels ... ok
```

### Step 6: 提交技术指标实现

**命令:**
```bash
git add src/indicators/ tests/indicators_extended_test.rs
git commit -m "feat: implement comprehensive technical indicators library"
```

---

## 验收标准

Phase 2完成标准：

- [ ] Pine Script DSL解析器能解析示例策略
- [ ] 15+技术指标实现并测试通过
- [ ] AST能正确表示所有支持的语法结构
- [ ] 策略能从文件加载并解析
- [ ] 指标计算性能达到 <1ms/1000数据点
- [ ] Freqtrade策略转换器基础框架完成
- [ ] 单元测试覆盖率 >90%
- [ ] 文档完整（语法规范、使用示例）

---

## 下一步

完成Phase 2后，继续：
- **Phase 3**: 监控系统（Redis/Timestream集成、Web Dashboard）
- **Phase 4**: 生产环境部署和优化

---

**计划创建完成！** 🎉

请使用 `superpowers:executing-plans` 来逐步实施此计划。