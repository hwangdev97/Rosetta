# XCStrings 翻译增强任务书

## 问题分析

当前的xcstrings翻译工具存在以下问题：

1. **上下文信息利用不足**：
   - 只使用原文本进行翻译
   - 忽略了`comment`字段中的重要描述信息
   - 没有利用其他语言的现有翻译作为参考

2. **Key信息没有充分利用**：
   - 某些Key（如`CFBundleDisplayName`, `NSLocationWhenInUseUsageDescription`）包含语义信息
   - 这些Key的含义可以帮助AI更好地理解上下文

3. **翻译质量可能不一致**：
   - 缺乏参考其他语言的翻译风格
   - 无法确保术语的一致性

## 改进方案

### 1. 增强翻译上下文信息

为翻译API提供更丰富的上下文信息：

- **Key语义解析**：解析Key的含义（如CFBundleDisplayName = 应用显示名称）
- **注释信息**：包含comment字段的描述
- **现有翻译参考**：提供其他语言的翻译作为参考
- **字符串用途推断**：基于Key和注释推断字符串的使用场景

### 2. 优化翻译Prompt

重新设计翻译提示词，包含：

```
翻译信息：
- Key: {key} (含义: {key_meaning})
- 注释: {comment}
- 原文: {source_text}
- 其他语言翻译:
  - 日语: {ja_translation}
  - 中文简体: {zh_hans_translation}
  - ...

上下文说明：
- 这是一个iOS应用的本地化字符串
- 用途：{usage_context}
- 保持与其他语言翻译风格的一致性
```

### 3. Key语义映射表

建立常见Key的语义映射：

```rust
pub static KEY_MEANINGS: &[(&str, &str)] = &[
    ("CFBundleDisplayName", "应用在主屏幕显示的名称"),
    ("CFBundleName", "应用包名称"),
    ("NSLocationWhenInUseUsageDescription", "使用期间位置权限说明"),
    ("NSCameraUsageDescription", "相机权限使用说明"),
    // ... 更多映射
];
```

## 实施计划

### 阶段1：扩展数据结构
- [ ] 在`XCStringsFile`中添加获取完整上下文信息的方法
- [ ] 创建`TranslationContext`结构体存储丰富的上下文信息

### 阶段2：Key语义解析
- [ ] 创建Key语义映射表
- [ ] 实现Key含义推断功能
- [ ] 支持自定义Key模式识别

### 阶段3：增强翻译逻辑
- [ ] 修改`translate_text`方法支持丰富上下文
- [ ] 重新设计翻译Prompt模板
- [ ] 添加翻译质量验证

### 阶段4：测试验证
- [ ] 使用提供的示例文件测试
- [ ] 对比翻译质量改进
- [ ] 性能影响评估

## 预期效果

1. **翻译质量提升**：通过丰富的上下文信息，AI能更准确理解待翻译内容的含义和用途
2. **术语一致性**：参考其他语言翻译，确保术语使用的一致性
3. **更好的用户体验**：生成更自然、更符合目标语言习惯的翻译

## 技术细节

### 新增数据结构

```rust
#[derive(Debug, Clone)]
pub struct TranslationContext {
    pub key: String,
    pub key_meaning: Option<String>,
    pub comment: Option<String>,
    pub source_text: String,
    pub existing_translations: HashMap<String, String>,
    pub usage_category: Option<String>,
}
```

### 修改后的翻译接口

```rust
impl Translator {
    pub async fn translate_with_context(
        &self,
        context: &TranslationContext,
        target_language: &str,
    ) -> Result<String>
}
```

这个增强方案将显著提升翻译质量，特别是对于像用户提供的那种包含大量系统Key的xcstrings文件。 