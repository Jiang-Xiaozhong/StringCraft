//! 20 个内置字符串转换纯函数（需求 4.3）。
//!
//! 统一约定：
//! - “单词”边界为空白、下划线、中横线、小数点、换行；
//! - 连续分隔符不做合并；
//! - 转换后无变化时原样返回；
//! - 全部规则对 Unicode 文本有效。

/// 是否为“空格字符”（空白但排除换行，包含半角/全角空格与制表符）。
fn is_space_char(c: char) -> bool {
    c.is_whitespace() && c != '\n' && c != '\r'
}

/// 是否为单词分隔符：空白、下划线、中横线、小数点、换行。
fn is_word_separator(c: char) -> bool {
    c.is_whitespace() || c == '_' || c == '-' || c == '.'
}

/// 每个单词首字母大写，其余字母小写，分隔符保留。
fn capitalize_words(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word_start = true;
    for c in text.chars() {
        if is_word_separator(c) {
            out.push(c);
            word_start = true;
        } else if word_start {
            out.extend(c.to_uppercase());
            word_start = false;
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

/// 每个单词首字母小写，其余字母大写，分隔符保留。
fn uncapitalize_words(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word_start = true;
    for c in text.chars() {
        if is_word_separator(c) {
            out.push(c);
            word_start = true;
        } else if word_start {
            out.extend(c.to_lowercase());
            word_start = false;
        } else {
            out.extend(c.to_uppercase());
        }
    }
    out
}

/// 每个句子首字母大写，其余字母小写；以 `.` `!` `?` 为句子边界。
fn sentence_case(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut sentence_start = true;
    for c in text.chars() {
        if sentence_start && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            sentence_start = false;
        } else {
            out.extend(c.to_lowercase());
            if c == '.' || c == '!' || c == '?' {
                sentence_start = true;
            } else if !c.is_whitespace() {
                sentence_start = false;
            }
        }
    }
    out
}

/// 按下划线/空格分词：首词转小写，后续词首字母大写、其余小写，删除分隔符；
/// 换行保留，每行单独按驼峰规则处理。
fn to_camel(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word_start = true;
    let mut first_word = true;
    for c in text.chars() {
        if c == '_' || is_space_char(c) {
            word_start = true;
        } else if c == '\n' || c == '\r' {
            out.push(c);
            word_start = true;
            first_word = true;
        } else if word_start {
            if first_word {
                out.extend(c.to_lowercase());
            } else {
                out.extend(c.to_uppercase());
            }
            first_word = false;
            word_start = false;
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

/// 大小写交界处分词，全部转小写，用指定分隔符连接。
fn camel_to_separated(text: &str, separator: char) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 8);
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev = i.checked_sub(1).and_then(|j| chars.get(j)).copied();
            let next = chars.get(i + 1).copied();
            let boundary = match prev {
                Some(p) if p.is_lowercase() || p.is_numeric() => true,
                Some(p) if p.is_uppercase() && next.is_some_and(|n| n.is_lowercase()) => true,
                _ => false,
            };
            if boundary {
                out.push(separator);
            }
        }
        out.extend(c.to_lowercase());
    }
    out
}

fn camel_to_underscore(text: &str) -> String {
    camel_to_separated(text, '_')
}

fn camel_to_space(text: &str) -> String {
    camel_to_separated(text, ' ')
}

/// 空格字符替换为指定字符（换行保留）。
fn replace_space_chars(text: &str, replacement: char) -> String {
    text.chars()
        .map(|c| if is_space_char(c) { replacement } else { c })
        .collect()
}

fn space_to_underscore(text: &str) -> String {
    replace_space_chars(text, '_')
}

fn space_to_hyphen(text: &str) -> String {
    replace_space_chars(text, '-')
}

fn space_to_newline(text: &str) -> String {
    replace_space_chars(text, '\n')
}

fn replace_char(text: &str, from: char, to: char) -> String {
    text.chars()
        .map(|c| if c == from { to } else { c })
        .collect()
}

fn underscore_to_hyphen(text: &str) -> String {
    replace_char(text, '_', '-')
}

fn hyphen_to_underscore(text: &str) -> String {
    replace_char(text, '-', '_')
}

fn underscore_to_space(text: &str) -> String {
    replace_char(text, '_', ' ')
}

fn underscore_to_dot(text: &str) -> String {
    replace_char(text, '_', '.')
}

fn dot_to_underscore(text: &str) -> String {
    replace_char(text, '.', '_')
}

/// 换行（含 CRLF）替换为空格。
fn newline_to_space(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            out.push(' ');
        } else if c == '\n' {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

/// 删除除 Unicode 字母、数字、空白外的所有字符。
fn remove_symbols(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect()
}

/// 删除所有空格字符（半角/全角），保留换行。
fn remove_spaces(text: &str) -> String {
    text.chars().filter(|c| !is_space_char(*c)).collect()
}

/// 删除所有换行（LF/CRLF），保留空格。
fn remove_newlines(text: &str) -> String {
    text.chars().filter(|c| *c != '\n' && *c != '\r').collect()
}

/// JSON 格式化：合法 JSON 按 2 空格缩进输出，保持字段顺序；非法输入原样返回。
fn json_format(text: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|_| text.to_string()),
        Err(_) => text.to_string(),
    }
}

const RMB_DIGITS: [char; 10] = ['零', '壹', '贰', '叁', '肆', '伍', '陆', '柒', '捌', '玖'];
const RMB_UNITS: [char; 4] = ['\0', '拾', '佰', '仟'];
const RMB_BIG_UNITS: [&str; 3] = ["", "万", "亿"];

/// 数字转人民币大写：支持 0~999999999999.99（千亿以内），非法输入返回 None。
fn number_to_rmb_uppercase(text: &str) -> Option<String> {
    let input = text.trim();
    if input.is_empty() || input.starts_with('-') {
        return None;
    }

    let (int_part, dec_part) = match input.split_once('.') {
        Some((i, d)) => (i, Some(d)),
        None => (input, None),
    };
    if int_part.is_empty() || !int_part.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let integer = int_part.trim_start_matches('0');
    if integer.len() > 12 {
        return None;
    }
    let integer = if integer.is_empty() { "0" } else { integer };

    let (jiao, fen) = match dec_part {
        Some(d) => {
            if d.is_empty() || !d.chars().all(|c| c.is_ascii_digit()) || d.len() > 2 {
                return None;
            }
            let mut padded = d.to_string();
            while padded.len() < 2 {
                padded.push('0');
            }
            let mut chars = padded.chars();
            (
                chars.next().unwrap().to_digit(10).unwrap(),
                chars.next().unwrap().to_digit(10).unwrap(),
            )
        }
        None => (0, 0),
    };

    let mut out = format!("{}元", integer_to_rmb(integer));
    if jiao == 0 && fen == 0 {
        out.push('整');
    } else {
        if jiao > 0 {
            out.push(RMB_DIGITS[jiao as usize]);
            out.push('角');
        } else if fen > 0 {
            out.push('零');
        }
        if fen > 0 {
            out.push(RMB_DIGITS[fen as usize]);
            out.push('分');
        }
    }
    Some(out)
}

/// 整数部分转中文大写，支持最多 12 位（千亿以内）。
fn integer_to_rmb(integer: &str) -> String {
    let width = integer.len().div_ceil(4) * 4;
    let padded = format!("{integer:0>width$}");
    let sections: Vec<&str> = padded
        .as_bytes()
        .chunks(4)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();

    let mut result = String::new();
    let mut need_zero = false;
    let count = sections.len();
    for (i, section) in sections.iter().enumerate() {
        let value: u32 = section.parse().unwrap_or(0);
        let big_index = count - 1 - i;
        if value == 0 {
            need_zero = true;
            continue;
        }
        if need_zero && !result.is_empty() && !result.ends_with('零') {
            result.push('零');
        }
        // 低位段以 0 开头时补零，如 10001 → 壹万零壹
        if section.starts_with('0') && !result.is_empty() && !result.ends_with('零') {
            result.push('零');
        }
        need_zero = false;
        result.push_str(&four_digit_to_rmb(section));
        result.push_str(RMB_BIG_UNITS[big_index]);
    }
    if result.is_empty() {
        result.push('零');
    }
    result
}

/// 4 位数字段转中文大写，如 "1234" → 壹仟贰佰叁拾肆。
fn four_digit_to_rmb(section: &str) -> String {
    let mut result = String::new();
    let mut zero = false;
    for (i, c) in section.chars().enumerate() {
        let digit = c.to_digit(10).unwrap() as usize;
        if digit == 0 {
            zero = true;
        } else {
            if zero && !result.is_empty() {
                result.push('零');
            }
            zero = false;
            result.push(RMB_DIGITS[digit]);
            if 3 - i > 0 {
                result.push(RMB_UNITS[3 - i]);
            }
        }
    }
    result
}

fn rmb_digit_value(c: char) -> Option<u32> {
    RMB_DIGITS.iter().position(|&d| d == c).map(|i| i as u32)
}

/// 人民币大写转数字：支持“元/角/分/整、零壹贰叁肆伍陆柒捌玖、拾佰仟万亿”写法，非法输入返回 None。
fn rmb_uppercase_to_number(text: &str) -> Option<String> {
    let input = text.trim();
    if input.is_empty() {
        return None;
    }

    let (yuan_part, dec_part) = match input.find('元') {
        Some(idx) => (&input[..idx], &input[idx + '元'.len_utf8()..]),
        None => ("", input),
    };
    let integer = parse_rmb_integer(yuan_part)?;

    let mut jiao = 0u32;
    let mut fen = 0u32;
    let mut chars = dec_part.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '整' || c == '零' {
            continue;
        }
        let digit = rmb_digit_value(c)?;
        match chars.next() {
            Some('角') => jiao = digit,
            Some('分') => fen = digit,
            _ => return None,
        }
    }

    let mut out = integer.to_string();
    if jiao != 0 || fen != 0 {
        out.push('.');
        out.push(char::from_digit(jiao, 10)?);
        out.push(char::from_digit(fen, 10)?);
        if fen == 0 {
            out.pop();
        }
    }
    Some(out)
}

/// 解析中文数字整数部分，返回数值。
fn parse_rmb_integer(s: &str) -> Option<u64> {
    if s.is_empty() {
        return Some(0);
    }
    let mut total: u64 = 0;
    let mut section: u64 = 0;
    let mut number: u64 = 0;
    let mut has_value = false;
    for c in s.chars() {
        if let Some(d) = rmb_digit_value(c) {
            number = d as u64;
            has_value = true;
        } else {
            let unit = match c {
                '拾' => 10u64,
                '佰' => 100,
                '仟' => 1000,
                '万' => 10_000,
                '亿' => 100_000_000,
                _ => return None,
            };
            if number == 0 && !has_value {
                number = 1; // “拾”开头表示一拾
            }
            match c {
                '拾' | '佰' | '仟' => {
                    section += number * unit;
                    number = 0;
                    has_value = true;
                }
                '万' => {
                    section += number;
                    total += section * unit;
                    section = 0;
                    number = 0;
                    has_value = true;
                }
                '亿' => {
                    section += number;
                    total += section * unit;
                    section = 0;
                    number = 0;
                    has_value = true;
                }
                _ => {}
            }
        }
    }
    Some(total + section + number)
}

/// 按行追加后缀：保留换行，CRLF 保持，行内容末尾添加后缀。
fn append_suffix(text: &str, suffix: &str) -> String {
    let parts: Vec<&str> = text.split('\n').collect();
    let ends_with_newline = text.ends_with('\n');
    let mut out = String::with_capacity(text.len() + suffix.len() * parts.len());
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if ends_with_newline && i == parts.len() - 1 {
            out.push_str(part);
            continue;
        }
        let content = part.strip_suffix('\r').unwrap_or(part);
        out.push_str(content);
        out.push_str(suffix);
        if part.ends_with('\r') {
            out.push('\r');
        }
    }
    out
}

/// 按行添加前缀：保留换行，CRLF 保持，行内容开头添加前缀。
fn prepend_prefix(text: &str, prefix: &str) -> String {
    let parts: Vec<&str> = text.split('\n').collect();
    let ends_with_newline = text.ends_with('\n');
    let mut out = String::with_capacity(text.len() + prefix.len() * parts.len());
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if ends_with_newline && i == parts.len() - 1 {
            out.push_str(part);
            continue;
        }
        let content = part.strip_suffix('\r').unwrap_or(part);
        out.push_str(prefix);
        out.push_str(content);
        if part.ends_with('\r') {
            out.push('\r');
        }
    }
    out
}

/// 按行同时添加前缀与后缀：保留换行，CRLF 保持。
fn prepend_and_append(text: &str, prefix: &str, suffix: &str) -> String {
    let parts: Vec<&str> = text.split('\n').collect();
    let ends_with_newline = text.ends_with('\n');
    let mut out = String::with_capacity(text.len() + (prefix.len() + suffix.len()) * parts.len());
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if ends_with_newline && i == parts.len() - 1 {
            out.push_str(part);
            continue;
        }
        let content = part.strip_suffix('\r').unwrap_or(part);
        out.push_str(prefix);
        out.push_str(content);
        out.push_str(suffix);
        if part.ends_with('\r') {
            out.push('\r');
        }
    }
    out
}

/// 全文替换（区分大小写、非正则）；被替换文本为空时原样返回。
fn replace_all(text: &str, from: &str, to: &str) -> String {
    if from.is_empty() {
        text.to_string()
    } else {
        text.replace(from, to)
    }
}

/// 去重复行：完整匹配，保留首次出现，顺序不变。
fn remove_duplicate_lines(text: &str) -> String {
    let parts: Vec<&str> = text.split('\n').collect();
    let ends_with_newline = text.ends_with('\n');
    let mut seen = std::collections::HashSet::new();
    let mut kept: Vec<String> = Vec::new();
    for (i, part) in parts.iter().enumerate() {
        if ends_with_newline && i == parts.len() - 1 {
            continue;
        }
        let content = part.strip_suffix('\r').unwrap_or(part);
        if seen.insert(content.to_string()) {
            kept.push(part.to_string());
        }
    }
    let mut out = kept.join("\n");
    if ends_with_newline {
        out.push('\n');
    }
    out
}

/// 自定义按钮转换调度；未知类型或参数缺失时原样返回。
pub fn transform_custom(
    text: &str,
    custom_type: &str,
    param1: Option<&str>,
    param2: Option<&str>,
) -> String {
    match custom_type {
        "append-suffix" => param1
            .map(|suffix| append_suffix(text, suffix))
            .unwrap_or_else(|| text.to_string()),
        "prepend-prefix" => param1
            .map(|prefix| prepend_prefix(text, prefix))
            .unwrap_or_else(|| text.to_string()),
        "prepend-append" => match (param1, param2) {
            (Some(prefix), Some(suffix)) => prepend_and_append(text, prefix, suffix),
            _ => text.to_string(),
        },
        "replace-text" => match (param1, param2) {
            (Some(from), Some(to)) => replace_all(text, from, to),
            _ => text.to_string(),
        },
        "remove-duplicate-lines" => remove_duplicate_lines(text),
        _ => text.to_string(),
    }
}

/// 按内置转换 id 执行转换；未知 id 原样返回。
pub fn transform(text: &str, transform_id: &str) -> String {
    match transform_id {
        "upper" => text.to_uppercase(),
        "lower" => text.to_lowercase(),
        "capitalize-words" => capitalize_words(text),
        "uncapitalize-words" => uncapitalize_words(text),
        "sentence-case" => sentence_case(text),
        "space-to-underscore" => space_to_underscore(text),
        "to-camel" => to_camel(text),
        "camel-to-underscore" => camel_to_underscore(text),
        "camel-to-space" => camel_to_space(text),
        "space-to-hyphen" => space_to_hyphen(text),
        "underscore-to-hyphen" => underscore_to_hyphen(text),
        "hyphen-to-underscore" => hyphen_to_underscore(text),
        "underscore-to-space" => underscore_to_space(text),
        "underscore-to-dot" => underscore_to_dot(text),
        "dot-to-underscore" => dot_to_underscore(text),
        "space-to-newline" => space_to_newline(text),
        "newline-to-space" => newline_to_space(text),
        "remove-symbols" => remove_symbols(text),
        "remove-spaces" => remove_spaces(text),
        "remove-newlines" => remove_newlines(text),
        "json-format" => json_format(text),
        "number-to-rmb" => number_to_rmb_uppercase(text).unwrap_or_else(|| text.to_string()),
        "rmb-to-number" => rmb_uppercase_to_number(text).unwrap_or_else(|| text.to_string()),
        _ => text.to_string(),
    }
}

/// 判断是否为已知的内置转换 id。
pub fn is_known_transform(transform_id: &str) -> bool {
    matches!(
        transform_id,
        "upper"
            | "lower"
            | "capitalize-words"
            | "uncapitalize-words"
            | "sentence-case"
            | "space-to-underscore"
            | "to-camel"
            | "camel-to-underscore"
            | "camel-to-space"
            | "space-to-hyphen"
            | "underscore-to-hyphen"
            | "hyphen-to-underscore"
            | "underscore-to-space"
            | "underscore-to-dot"
            | "dot-to-underscore"
            | "space-to-newline"
            | "newline-to-space"
            | "remove-symbols"
            | "remove-spaces"
            | "remove-newlines"
            | "json-format"
            | "number-to-rmb"
            | "rmb-to-number"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upper_lower() {
        assert_eq!(transform("hello world", "upper"), "HELLO WORLD");
        assert_eq!(transform("Hello World", "lower"), "hello world");
        assert_eq!(transform("中文abc", "upper"), "中文ABC");
    }

    #[test]
    fn capitalize_words() {
        assert_eq!(transform("hello world", "capitalize-words"), "Hello World");
        assert_eq!(transform("hello_world", "capitalize-words"), "Hello_World");
        assert_eq!(
            transform("hello  world", "capitalize-words"),
            "Hello  World"
        );
        assert_eq!(transform("hello_WORLD", "capitalize-words"), "Hello_World");
        assert_eq!(transform("THINK", "capitalize-words"), "Think");
    }

    #[test]
    fn uncapitalize_words() {
        assert_eq!(
            transform("hello world", "uncapitalize-words"),
            "hELLO wORLD"
        );
        assert_eq!(
            transform("Hello World", "uncapitalize-words"),
            "hELLO wORLD"
        );
    }

    #[test]
    fn sentence_case() {
        assert_eq!(
            transform("hello world. how are you", "sentence-case"),
            "Hello world. How are you"
        );
        assert_eq!(
            transform("hi! what's up?", "sentence-case"),
            "Hi! What's up?"
        );
    }

    #[test]
    fn space_replacements() {
        assert_eq!(
            transform("hello world", "space-to-underscore"),
            "hello_world"
        );
        assert_eq!(transform("a  b", "space-to-underscore"), "a__b");
        assert_eq!(transform("hello world", "space-to-hyphen"), "hello-world");
        assert_eq!(transform("a b", "space-to-newline"), "a\nb");
    }

    #[test]
    fn camel_case_conversions() {
        assert_eq!(transform("hello_world", "to-camel"), "helloWorld");
        assert_eq!(transform("hello world", "to-camel"), "helloWorld");
        assert_eq!(
            transform("hello_world\nfoo_bar", "to-camel"),
            "helloWorld\nfooBar"
        );
        assert_eq!(
            transform("hello_world\r\nfoo_bar", "to-camel"),
            "helloWorld\r\nfooBar"
        );
        assert_eq!(
            transform("helloWorld", "camel-to-underscore"),
            "hello_world"
        );
        assert_eq!(
            transform("HelloWorld", "camel-to-underscore"),
            "hello_world"
        );
        assert_eq!(transform("helloWorld", "camel-to-space"), "hello world");
        assert_eq!(transform("HelloWorld", "camel-to-space"), "hello world");
        assert_eq!(
            transform("HTTPServer", "camel-to-underscore"),
            "http_server"
        );
    }

    #[test]
    fn separator_replacements() {
        assert_eq!(
            transform("hello_world", "underscore-to-hyphen"),
            "hello-world"
        );
        assert_eq!(
            transform("hello-world", "hyphen-to-underscore"),
            "hello_world"
        );
        assert_eq!(
            transform("hello_world", "underscore-to-space"),
            "hello world"
        );
        assert_eq!(transform("hello_world", "underscore-to-dot"), "hello.world");
        assert_eq!(transform("hello.world", "dot-to-underscore"), "hello_world");
    }

    #[test]
    fn newline_conversions() {
        assert_eq!(transform("a\nb", "newline-to-space"), "a b");
        assert_eq!(transform("a\r\nb", "newline-to-space"), "a b");
        assert_eq!(transform("a\nb", "remove-newlines"), "ab");
        assert_eq!(transform("a\r\nb", "remove-newlines"), "ab");
    }

    #[test]
    fn removals() {
        assert_eq!(transform("hello, world!", "remove-symbols"), "hello world");
        assert_eq!(transform("h e l l o", "remove-spaces"), "hello");
        assert_eq!(transform("a\u{3000}b", "remove-spaces"), "ab");
        assert_eq!(transform("a\nb", "remove-newlines"), "ab");
    }

    #[test]
    fn unicode_and_edge_cases() {
        assert_eq!(transform("中文 测试", "to-camel"), "中文测试");
        assert_eq!(transform("", "upper"), "");
        assert_eq!(transform("abc", "unknown-id"), "abc");
        assert_eq!(transform("hello", "upper"), "HELLO");
    }

    #[test]
    fn json_formatting() {
        let compact = r#"{"sites":{"site":[{"id":"1","name":"菜鸟教程","url":"www.runoob.com"}]}}"#;
        let pretty = transform(compact, "json-format");
        assert!(pretty.contains("\n  \"sites\""));
        assert!(pretty.contains("\"name\": \"菜鸟教程\""));
        assert_eq!(transform("not json", "json-format"), "not json");

        let ordered = transform(r#"{"b":1,"a":2}"#, "json-format");
        assert!(ordered.find("\"b\"").unwrap() < ordered.find("\"a\"").unwrap());
    }

    #[test]
    fn number_to_rmb() {
        assert_eq!(
            transform("1234.56", "number-to-rmb"),
            "壹仟贰佰叁拾肆元伍角陆分"
        );
        assert_eq!(transform("0", "number-to-rmb"), "零元整");
        assert_eq!(transform("100000000", "number-to-rmb"), "壹亿元整");
        assert_eq!(transform("0.05", "number-to-rmb"), "零元零伍分");
        assert_eq!(transform("abc", "number-to-rmb"), "abc");
    }

    #[test]
    fn rmb_to_number() {
        assert_eq!(
            transform("壹仟贰佰叁拾肆元伍角陆分", "rmb-to-number"),
            "1234.56"
        );
        assert_eq!(transform("零元整", "rmb-to-number"), "0");
        assert_eq!(transform("壹万元整", "rmb-to-number"), "10000");
        assert_eq!(transform("伍角", "rmb-to-number"), "0.5");
        assert_eq!(transform("非法输入", "rmb-to-number"), "非法输入");
    }

    #[test]
    fn custom_transforms() {
        assert_eq!(
            transform_custom("a\nb", "append-suffix", Some("!"), None),
            "a!\nb!"
        );
        assert_eq!(
            transform_custom("a\nb", "prepend-prefix", Some(">"), None),
            ">a\n>b"
        );
        assert_eq!(
            transform_custom("a\nb", "prepend-append", Some(">"), Some("!")),
            ">a!\n>b!"
        );
        assert_eq!(
            transform_custom("aab aa", "replace-text", Some("aa"), Some("bb")),
            "bbb bb"
        );
        assert_eq!(
            transform_custom("a\nb\na\nb", "remove-duplicate-lines", None, None),
            "a\nb"
        );
        assert_eq!(transform_custom("x", "unknown", None, None), "x");
        assert_eq!(
            transform_custom("a\r\nb", "append-suffix", Some("!"), None),
            "a!\r\nb!"
        );
        assert_eq!(
            transform_custom("a\r\nb", "prepend-append", Some(">"), Some("!")),
            ">a!\r\n>b!"
        );
    }
}
