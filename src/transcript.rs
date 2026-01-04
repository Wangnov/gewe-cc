use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Transcript 条目（顶层）
#[derive(Debug, Deserialize, Serialize)]
pub struct TranscriptEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(default)]
    pub message: Option<Message>,
    #[serde(default)]
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    #[serde(default)]
    pub content: MessageContent,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    String(String),
    Array(Vec<ContentBlock>),
}

impl Default for MessageContent {
    fn default() -> Self {
        MessageContent::Array(Vec::new())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "thinking")]
    Thinking { thinking: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        #[serde(default)]
        content: ToolResultContent,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    String(String),
    Array(Vec<ToolResultItem>),
}

impl Default for ToolResultContent {
    fn default() -> Self {
        ToolResultContent::String(String::new())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolResultItem {
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

/// 解析 transcript 文件
pub fn parse_transcript(path: &Path) -> Result<Vec<Message>> {
    let content = fs::read_to_string(path)
        .context(format!("读取 transcript 文件失败: {}", path.display()))?;

    let mut messages = Vec::new();
    for (line_no, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let entry: TranscriptEntry = serde_json::from_str(line).context(format!(
            "解析 transcript 第 {} 行失败: {}",
            line_no + 1,
            path.display()
        ))?;

        // 只保留有 message 的条目（user 和 assistant 消息）
        if let Some(message) = entry.message {
            messages.push(message);
        }
    }

    Ok(messages)
}

/// 将消息渲染成 HTML
pub fn render_to_html(messages: &[Message], session_id: &str) -> String {
    let mut html = String::new();
    let safe_session_id = html_escape(session_id);

    // HTML 头部
    html.push_str(&format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>会话记录 - {}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 20px;
        }}

        .container {{
            max-width: 1000px;
            margin: 0 auto;
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            overflow: hidden;
        }}

        header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
            position: relative;
        }}

        header h1 {{
            font-size: 2em;
            margin-bottom: 10px;
        }}

        header p {{
            opacity: 0.9;
            font-size: 0.9em;
        }}

        .scroll-btn {{
            position: fixed;
            bottom: 30px;
            right: 30px;
            background: #667eea;
            color: white;
            border: none;
            padding: 15px 25px;
            border-radius: 50px;
            cursor: pointer;
            font-size: 16px;
            box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
            transition: all 0.3s ease;
            z-index: 1000;
        }}

        .scroll-btn:hover {{
            background: #5568d3;
            transform: translateY(-2px);
            box-shadow: 0 6px 20px rgba(102, 126, 234, 0.6);
        }}

        .messages {{
            padding: 20px;
        }}

        .message {{
            margin-bottom: 20px;
            padding: 15px 20px;
            border-radius: 8px;
            border-left: 4px solid #ccc;
        }}

        .message.user {{
            background: #e3f2fd;
            border-left-color: #2196f3;
        }}

        .message.assistant {{
            background: #f3e5f5;
            border-left-color: #9c27b0;
        }}

        .message-role {{
            font-weight: bold;
            margin-bottom: 8px;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}

        .message.user .message-role {{
            color: #1976d2;
        }}

        .message.assistant .message-role {{
            color: #7b1fa2;
        }}

        .message-content {{
            white-space: pre-wrap;
            word-wrap: break-word;
        }}

        .message-content pre {{
            background: #f5f5f5;
            padding: 15px;
            border-radius: 5px;
            overflow-x: auto;
            margin: 10px 0;
        }}

        .message-content code {{
            background: #f5f5f5;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: "Monaco", "Menlo", "Ubuntu Mono", monospace;
            font-size: 0.9em;
        }}

        .message-content pre code {{
            background: none;
            padding: 0;
        }}

        .tool-use {{
            background: #fff3e0;
            border-left-color: #ff9800;
            font-size: 0.9em;
        }}

        .tool-use .tool-name {{
            font-weight: bold;
            color: #f57c00;
            margin-bottom: 5px;
        }}

        .tool-result {{
            background: #e8f5e9;
            border-left-color: #4caf50;
            font-size: 0.9em;
        }}

        .thinking {{
            background: #fff8e1;
            border-left: 4px solid #ffc107;
            padding: 15px 20px;
            margin-bottom: 20px;
            border-radius: 8px;
            font-size: 0.9em;
        }}

        .thinking-header {{
            font-weight: bold;
            color: #f57f17;
            margin-bottom: 8px;
        }}

        @media (max-width: 768px) {{
            body {{
                padding: 10px;
            }}

            header {{
                padding: 20px;
            }}

            header h1 {{
                font-size: 1.5em;
            }}

            .scroll-btn {{
                bottom: 15px;
                right: 15px;
                padding: 12px 20px;
                font-size: 14px;
            }}
        }}
    </style>
    <script src="https://cdn.jsdelivr.net/npm/marked@12/marked.min.js"></script>
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github.min.css">
    <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
</head>
<body>
    <div class="container">
        <header>
            <h1>📝 会话记录</h1>
            <p>Session ID: {}</p>
        </header>
        <div class="messages">
"#,
        safe_session_id, safe_session_id
    ));

    // 渲染每条消息
    for message in messages {
        let role_class = match message.role.as_str() {
            "user" => "user",
            "assistant" => "assistant",
            _ => "other",
        };

        html.push_str(&format!(
            r#"            <div class="message {}">
                <div class="message-role">{}</div>
"#,
            role_class, message.role
        ));

        // 渲染消息内容
        match &message.content {
            MessageContent::String(text) => {
                html.push_str(r#"                <div class="message-content" data-markdown>"#);
                html.push_str(&html_escape(text));
                html.push_str("</div>\n");
            }
            MessageContent::Array(blocks) => {
                for block in blocks {
                    match block {
                        ContentBlock::Text { text } => {
                            html.push_str(r#"                <div class="message-content" data-markdown>"#);
                            html.push_str(&html_escape(text));
                            html.push_str("</div>\n");
                        }
                        ContentBlock::Thinking { thinking } => {
                            html.push_str(r#"                <div class="thinking">"#);
                            html.push_str(r#"<div class="thinking-header">💭 思考过程</div>"#);
                            html.push_str("<pre><code>");
                            html.push_str(&html_escape(thinking));
                            html.push_str("</code></pre>");
                            html.push_str("</div>\n");
                        }
                        ContentBlock::ToolUse { name, input, .. } => {
                            html.push_str(r#"                <div class="tool-use">"#);
                            html.push_str(&format!(r#"<div class="tool-name">🔧 Tool: {}</div>"#, name));
                            html.push_str("<pre><code>");
                            html.push_str(&html_escape(
                                &serde_json::to_string_pretty(input).unwrap_or_default(),
                            ));
                            html.push_str("</code></pre>");
                            html.push_str("</div>\n");
                        }
                        ContentBlock::ToolResult { content, .. } => {
                            html.push_str(r#"                <div class="tool-result">"#);
                            html.push_str("<pre><code>");
                            match content {
                                ToolResultContent::String(s) => {
                                    html.push_str(&html_escape(s));
                                }
                                ToolResultContent::Array(items) => {
                                    for item in items {
                                        if let Some(text) = &item.text {
                                            html.push_str(&html_escape(text));
                                        }
                                    }
                                }
                            }
                            html.push_str("</code></pre>");
                            html.push_str("</div>\n");
                        }
                        ContentBlock::Other => {}
                    }
                }
            }
        }

        html.push_str("            </div>\n");
    }

    // HTML 尾部
    html.push_str(
        r#"        </div>
    </div>
    <button class="scroll-btn" onclick="scrollToBottom()">⬇️ 跳到底部</button>
    <script>
        function scrollToBottom() {
            window.scrollTo({ top: document.body.scrollHeight, behavior: 'smooth' });
        }

        // Markdown 渲染
        document.addEventListener('DOMContentLoaded', function() {
            const renderer = new marked.Renderer();
            renderer.html = () => '';
            renderer.link = (href, title, text) => {
                const safeHref = sanitizeUrl(href);
                if (!safeHref) return text;
                const titleAttr = title ? ` title="${escapeAttr(title)}"` : '';
                return `<a href="${safeHref}"${titleAttr} rel="noopener noreferrer" target="_blank">${text}</a>`;
            };
            renderer.image = (href, title, text) => {
                const safeHref = sanitizeUrl(href);
                if (!safeHref) return text;
                const titleAttr = title ? ` title="${escapeAttr(title)}"` : '';
                const altAttr = text ? ` alt="${escapeAttr(text)}"` : ' alt=""';
                return `<img src="${safeHref}"${altAttr}${titleAttr} />`;
            };

            marked.use({ renderer, mangle: false, headerIds: false });

            document.querySelectorAll('[data-markdown]').forEach(el => {
                const markdown = el.textContent;
                el.innerHTML = marked.parse(markdown);
            });

            // 代码高亮
            hljs.highlightAll();
        });

        function sanitizeUrl(href) {
            if (!href) return null;
            if (href.startsWith('#') || href.startsWith('/')) return href;
            try {
                const url = new URL(href, window.location.origin);
                if (['http:', 'https:', 'mailto:'].includes(url.protocol)) {
                    return url.href;
                }
            } catch (e) {
                return null;
            }
            return null;
        }

        function escapeAttr(value) {
            return value
                .replace(/&/g, "&amp;")
                .replace(/"/g, "&quot;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;");
        }
    </script>
</body>
</html>
"#,
    );

    html
}

/// HTML 转义
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>alert('xss')</script>"),
                   "&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;");
    }
}
