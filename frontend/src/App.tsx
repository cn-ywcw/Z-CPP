/// Z-CPP 主应用组件
/// 轻量级 C/C++ IDE 主体界面

import React, { useState, useCallback, useEffect, useRef } from 'react';
import {
  Layout,
  Button,
  Select,
  Space,
  Typography,
  message,
  Tag,
  Switch,
  App as AntApp,
  Spin,
} from 'antd';
import {
  PlayCircleOutlined,
  SettingOutlined,
  ClearOutlined,
  SaveOutlined,
  FileAddOutlined,
} from '@ant-design/icons';
import Editor from '@monaco-editor/react';
import type { editor } from 'monaco-editor';
import {
  compileCode,
  checkHealth,
  CompileResponse,
  HealthResponse,
} from './services/api';

const { Header, Content, Sider } = Layout;
const { Title, Text } = Typography;

/// 默认 C++ 模板代码
const DEFAULT_CODE = `#include <iostream>
using namespace std;

int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b << endl;
    return 0;
}
`;

/// 默认 C 模板代码
const DEFAULT_C_CODE = `#include <stdio.h>

int main() {
    int a, b;
    scanf("%d %d", &a, &b);
    printf("%d\\n", a + b);
    return 0;
}
`;

const App: React.FC = () => {
  const [code, setCode] = useState(DEFAULT_CODE);
  const [filename, setFilename] = useState('main.cpp');
  const [compiler, setCompiler] = useState<'gcc' | 'clang'>('gcc');
  const [language, setLanguage] = useState<'cpp' | 'c'>('cpp');
  const [compiling, setCompiling] = useState(false);
  const [compileOnly, setCompileOnly] = useState(false);
  const [result, setResult] = useState<CompileResponse | null>(null);
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [backendReady, setBackendReady] = useState(false);
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  // 启动时检查后端健康状态
  useEffect(() => {
    const check = async () => {
      try {
        const h = await checkHealth();
        setHealth(h);
        setBackendReady(true);
        message.success('后端服务已连接');
      } catch {
        setBackendReady(false);
        message.warning('后端服务未连接，请确保已启动 cargo run');
        // 重试
        setTimeout(check, 2000);
      }
    };
    check();
  }, []);

  // 语言切换
  const handleLanguageChange = useCallback((lang: 'cpp' | 'c') => {
    setLanguage(lang);
    setFilename(lang === 'cpp' ? 'main.cpp' : 'main.c');
    setCode(lang === 'cpp' ? DEFAULT_CODE : DEFAULT_C_CODE);
  }, []);

  // 编译器切换
  const handleCompilerChange = useCallback((value: 'gcc' | 'clang') => {
    setCompiler(value);
  }, []);

  // 编辑器挂载
  const handleEditorDidMount = useCallback(
    (editorInstance: editor.IStandaloneCodeEditor) => {
      editorRef.current = editorInstance;
    },
    []
  );

  // 编译运行
  const handleCompile = useCallback(async () => {
    if (!backendReady) {
      message.error('后端服务未连接');
      return;
    }

    setCompiling(true);
    setResult(null);

    try {
      const res = await compileCode({
        code,
        filename,
        compiler,
        options: '',
        std: language === 'cpp' ? 'c++17' : 'c17',
        compile_only: compileOnly,
      });
      setResult(res);
      if (res.success) {
        message.success('编译运行成功');
      } else {
        message.error('编译失败');
      }
    } catch (e: any) {
      message.error(`请求失败: ${e.message}`);
    } finally {
      setCompiling(false);
    }
  }, [code, filename, compiler, language, compileOnly, backendReady]);

  return (
    <AntApp>
      <Layout style={{ height: '100vh', background: '#1e1e1e' }}>
        {/* 顶部标题栏 */}
        <Header
          style={{
            background: '#2d2d2d',
            padding: '0 16px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            borderBottom: '1px solid #3d3d3d',
            height: 48,
          }}
        >
          <Space>
            <Title level={5} style={{ margin: 0, color: '#4fc3f7' }}>
              Z-CPP
            </Title>
            <Text type="secondary" style={{ fontSize: 12 }}>
              轻量级 C/C++ IDE
            </Text>
            {health && (
              <Tag
                color={backendReady ? 'success' : 'error'}
                style={{ marginLeft: 8 }}
              >
                {backendReady ? '已连接' : '未连接'}
              </Tag>
            )}
          </Space>

          <Space>
            <Select
              value={language}
              onChange={handleLanguageChange}
              size="small"
              style={{ width: 80 }}
              options={[
                { value: 'cpp', label: 'C++' },
                { value: 'c', label: 'C' },
              ]}
            />
            <Select
              value={compiler}
              onChange={handleCompilerChange}
              size="small"
              style={{ width: 90 }}
              options={[
                {
                  value: 'gcc',
                  label: `GCC${health?.gcc_available ? ' ✓' : ' ✗'}`,
                  disabled: !health?.gcc_available,
                },
                {
                  value: 'clang',
                  label: `Clang${health?.clang_available ? ' ✓' : ' ✗'}`,
                  disabled: !health?.clang_available,
                },
              ]}
            />
          </Space>
        </Header>

        <Layout style={{ height: 'calc(100vh - 48px)' }}>
          {/* 编辑器区 */}
          <Content
            style={{
              display: 'flex',
              flexDirection: 'column',
              background: '#1e1e1e',
            }}
          >
            {/* 操作栏 */}
            <div
              style={{
                padding: '4px 12px',
                background: '#252526',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                borderBottom: '1px solid #3d3d3d',
              }}
            >
              <Space>
                <Text style={{ color: '#ccc', fontSize: 12 }}>
                  {filename}
                </Text>
              </Space>
              <Space size="small">
                <span style={{ color: '#888', fontSize: 12 }}>
                  <Switch
                    size="small"
                    checked={compileOnly}
                    onChange={setCompileOnly}
                  />{' '}
                  仅编译
                </span>
                <Button
                  type="primary"
                  icon={<PlayCircleOutlined />}
                  onClick={handleCompile}
                  loading={compiling}
                  size="small"
                  style={{
                    background: compiling ? undefined : '#4fc3f7',
                    borderColor: compiling ? undefined : '#4fc3f7',
                  }}
                >
                  {compiling ? '编译中...' : compileOnly ? '编译' : '编译运行'}
                </Button>
              </Space>
            </div>

            {/* Monaco 编辑器 */}
            <div style={{ flex: 1 }}>
              <Editor
                height="100%"
                language={language === 'cpp' ? 'cpp' : 'c'}
                theme="vs-dark"
                value={code}
                onChange={(val: string | undefined) => setCode(val || '')}
                onMount={handleEditorDidMount}
                options={{
                  fontSize: 14,
                  fontFamily: "'Cascadia Code', 'Fira Code', 'Consolas', monospace",
                  minimap: { enabled: false },
                  scrollBeyondLastLine: false,
                  lineNumbers: 'on',
                  renderLineHighlight: 'line',
                  automaticLayout: true,
                  tabSize: 4,
                  wordWrap: 'off',
                  padding: { top: 8 },
                }}
              />
            </div>
          </Content>

          {/* 右侧输出面板 */}
          <Sider
            width="40%"
            style={{
              background: '#1e1e1e',
              borderLeft: '1px solid #3d3d3d',
              display: 'flex',
              flexDirection: 'column',
            }}
          >
            <div
              style={{
                padding: '8px 12px',
                background: '#252526',
                borderBottom: '1px solid #3d3d3d',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              <Text style={{ color: '#ccc', fontSize: 13, fontWeight: 500 }}>
                输出
              </Text>
              {result && (
                <Space size="small">
                  {result.run_time_ms !== null && (
                    <Tag color="default" style={{ fontSize: 11 }}>
                      {result.run_time_ms} ms
                    </Tag>
                  )}
                  <Button
                    type="text"
                    size="small"
                    icon={<ClearOutlined />}
                    onClick={() => setResult(null)}
                    style={{ color: '#888' }}
                  />
                </Space>
              )}
            </div>

            <div
              style={{
                flex: 1,
                overflow: 'auto',
                padding: '8px 12px',
                fontFamily: "'Cascadia Code', 'Consolas', monospace",
                fontSize: 13,
                whiteSpace: 'pre-wrap',
                color: '#d4d4d4',
              }}
            >
              {compiling && (
                <div style={{ textAlign: 'center', padding: 40 }}>
                  <Spin tip="编译中..." />
                </div>
              )}

              {!compiling && !result && (
                <div style={{ color: '#666', padding: 20 }}>
                  点击「编译运行」按钮开始
                </div>
              )}

              {result && !result.success && (
                <div>
                  <div style={{ color: '#f48771', marginBottom: 8 }}>
                    ⚠ 编译错误
                  </div>
                  <pre
                    style={{
                      margin: 0,
                      color: '#f48771',
                      fontFamily: 'inherit',
                      fontSize: 'inherit',
                    }}
                  >
                    {result.compile_output}
                  </pre>
                </div>
              )}

              {result && result.success && (
                <div>
                  {result.compile_output && (
                    <div style={{ marginBottom: 12 }}>
                      <div style={{ color: '#6a9955', marginBottom: 4 }}>
                        ✓ 编译成功
                      </div>
                      {result.compile_output
                        .split('\n')
                        .filter((l) => l.trim())
                        .map((line, i) => (
                          <div key={i} style={{ color: '#888' }}>
                            {line}
                          </div>
                        ))}
                    </div>
                  )}

                  {!result.compile_output && (
                    <div style={{ color: '#6a9955', marginBottom: 12 }}>
                      ✓ 编译成功
                    </div>
                  )}

                  {!compileOnly && (
                    <>
                      <div
                        style={{
                          color: '#569cd6',
                          marginBottom: 4,
                          borderTop: '1px solid #333',
                          paddingTop: 8,
                        }}
                      >
                        运行输出
                      </div>
                      <pre
                        style={{
                          margin: 0,
                          color: '#d4d4d4',
                          fontFamily: 'inherit',
                          fontSize: 'inherit',
                        }}
                      >
                        {result.run_output || '（无输出）'}
                      </pre>
                      {result.exit_code !== null && result.exit_code !== 0 && (
                        <div style={{ color: '#f48771', marginTop: 8 }}>
                          进程退出，退出码: {result.exit_code}
                        </div>
                      )}
                    </>
                  )}
                </div>
              )}
            </div>
          </Sider>
        </Layout>
      </Layout>
    </AntApp>
  );
};

export default App;
