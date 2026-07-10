/// Z-CPP 主应用 — 多文件 IDE
///
/// 功能：文件浏览器 + 多标签编辑器 + 编译选项 + 设置

import React, { useState, useCallback, useEffect, useRef } from 'react';
import {
  Layout, Button, Select, Space, Typography, Tag, Switch,
  App as AntApp, Spin, Modal, Input, Drawer, message, Tooltip, Dropdown,
} from 'antd';
import {
  PlayCircleOutlined, SettingOutlined, ClearOutlined, FileAddOutlined,
  FolderOpenOutlined, PlusOutlined, CloseOutlined, CodeOutlined,
} from '@ant-design/icons';
import Editor from '@monaco-editor/react';
import type { editor } from 'monaco-editor';
import * as api from './services/api';

const { Header, Sider, Content } = Layout;
const { Text, Title } = Typography;

// ── 模板代码 ──────────────────────────────────────────

const TEMPLATES: Record<string, string> = {
  'main.cpp': `#include <iostream>
using namespace std;

int main() {
    int a, b;
    cin >> a >> b;
    cout << a + b << endl;
    return 0;
}
`,
  'main.c': `#include <stdio.h>

int main() {
    int a, b;
    scanf("%d %d", &a, &b);
    printf("%d\\n", a + b);
    return 0;
}
`,
};

// ── 编辑器 Tab ────────────────────────────────────────

interface Tab {
  filename: string;
  code: string;
  modified: boolean;
  language: 'cpp' | 'c';
}

// ── 主组件 ────────────────────────────────────────────

const App: React.FC = () => {
  // 文件标签
  const [tabs, setTabs] = useState<Tab[]>([
    { filename: 'main.cpp', code: TEMPLATES['main.cpp'], modified: false, language: 'cpp' },
  ]);
  const [activeTab, setActiveTab] = useState(0);
  const [fileList, setFileList] = useState<api.FileInfo[]>([]);

  // 编译
  const [compiling, setCompiling] = useState(false);
  const [compileOnly, setCompileOnly] = useState(false);
  const [result, setResult] = useState<api.CompileResponse | null>(null);

  // 编译选项
  const [optLevel, setOptLevel] = useState('O2');
  const [warnings, setWarnings] = useState('Wall-Wextra');
  const [stdVersion, setStdVersion] = useState('');
  const [extraFlags, setExtraFlags] = useState('');
  const [compiler, setCompiler] = useState<'gcc' | 'clang'>('gcc');

  // 设置
  const [settings, setSettings] = useState<api.Settings | null>(null);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [editGccPath, setEditGccPath] = useState('');
  const [editClangPath, setEditClangPath] = useState('');
  const [editWorkspace, setEditWorkspace] = useState('');

  // 状态
  const [health, setHealth] = useState<api.HealthResponse | null>(null);
  const [backendReady, setBackendReady] = useState(false);
  const [showOptPanel, setShowOptPanel] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const [newFileModal, setNewFileModal] = useState(false);

  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);

  const active = tabs[activeTab];

  // ── 初始化 ──────────────────────────────────────────

  useEffect(() => {
    const init = async () => {
      try {
        const h = await api.checkHealth();
        setHealth(h);
        setBackendReady(true);
        // 加载设置
        const s = await api.getSettings();
        setSettings(s);
        setCompiler(s.default_compiler as 'gcc' | 'clang');
        setOptLevel(s.default_options.optimization || 'O2');
        setWarnings(s.default_options.warnings || 'Wall-Wextra');
        setStdVersion(s.default_options.standard || '');
        setExtraFlags(s.default_options.extra_flags || '');
        setEditGccPath(s.gcc_path);
        setEditClangPath(s.clang_path);
        setEditWorkspace(s.workspace);
        // 加载文件列表
        refreshFiles();
      } catch {
        setBackendReady(false);
        message.warning('后端未连接，请启动 cargo run');
        setTimeout(init, 2000);
      }
    };
    init();
  }, []);

  const refreshFiles = async () => {
    try {
      const res = await api.listFiles();
      setFileList(res.files);
    } catch { /* ignore */ }
  };

  // ── 文件操作 ────────────────────────────────────────

  const openFile = async (filename: string) => {
    const idx = tabs.findIndex(t => t.filename === filename);
    if (idx >= 0) { setActiveTab(idx); return; }

    let code = TEMPLATES[filename];
    if (code === undefined) {
      const loaded = await api.loadFile(filename);
      code = loaded ?? '// 新文件\n';
    }
    const lang = filename.endsWith('.c') ? 'c' : 'cpp';
    setTabs(prev => [...prev, { filename, code, modified: false, language: lang }]);
    setActiveTab(tabs.length);
  };

  const closeTab = (idx: number) => {
    if (tabs.length <= 1) return;
    setTabs(prev => prev.filter((_, i) => i !== idx));
    if (activeTab >= idx && activeTab > 0) setActiveTab(activeTab - 1);
  };

  const handleNewFile = async () => {
    if (!newFileName.trim()) return;
    const name = newFileName.trim();
    // 判断扩展名
    const ext = name.includes('.') ? '' : '.cpp';
    const fullName = name + ext;
    await api.createFile(fullName, TEMPLATES[fullName] || '');
    setNewFileModal(false);
    setNewFileName('');
    refreshFiles();
    openFile(fullName);
  };

  const handleCodeChange = (val: string | undefined) => {
    const code = val || '';
    setTabs(prev => prev.map((t, i) => i === activeTab ? { ...t, code, modified: true } : t));
  };

  // ── 编译 ────────────────────────────────────────────

  const handleCompile = async () => {
    if (!active) return;
    if (!backendReady) { message.error('后端未连接'); return; }

    // 先保存
    await api.saveFile(active.filename, active.code);
    setTabs(prev => prev.map((t, i) => i === activeTab ? { ...t, modified: false } : t));

    setCompiling(true);
    setResult(null);
    try {
      const res = await api.compileCode({
        code: active.code,
        filename: active.filename,
        compiler,
        compile_options: {
          optimization: optLevel,
          warnings,
          standard: stdVersion || (active.language === 'c' ? 'c17' : 'c++17'),
          extra_flags: extraFlags,
        },
        compile_only: compileOnly,
      });
      setResult(res);
      if (res.success) message.success(compileOnly ? '编译成功' : '编译运行成功');
      else message.error('编译失败');
    } catch (e: any) {
      message.error(`请求失败: ${e.message}`);
    } finally {
      setCompiling(false);
    }
  };

  // ── 设置 ────────────────────────────────────────────

  const handleSaveSettings = async () => {
    const newSettings: api.Settings = {
      gcc_path: editGccPath,
      clang_path: editClangPath,
      default_compiler: compiler,
      default_options: { optimization: optLevel, warnings, standard: stdVersion, extra_flags: extraFlags },
      workspace: editWorkspace,
    };
    const ok = await api.saveSettings(newSettings);
    if (ok) {
      setSettings(newSettings);
      setSettingsOpen(false);
      message.success('设置已保存');
      // 刷新健康状态
      const h = await api.checkHealth();
      setHealth(h);
    } else {
      message.error('保存失败');
    }
  };

  // ── 渲染 ────────────────────────────────────────────

  const renderEditor = () => {
    if (!active) return null;
    return (
      <Editor
        height="100%"
        language={active.language}
        theme="vs-dark"
        value={active.code}
        onChange={handleCodeChange}
        onMount={(e: any) => { editorRef.current = e; }}
        options={{
          fontSize: 14,
          fontFamily: "'Cascadia Code', 'Fira Code', 'Consolas', monospace",
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          lineNumbers: 'on',
          automaticLayout: true,
          tabSize: 4,
          wordWrap: 'off',
          padding: { top: 8 },
        }}
      />
    );
  };

  return (
    <AntApp>
      <Layout style={{ height: '100vh', background: '#1e1e1e' }}>
        {/* 顶部栏 */}
        <Header style={{
          background: '#2d2d2d', padding: '0 12px', display: 'flex',
          alignItems: 'center', justifyContent: 'space-between',
          borderBottom: '1px solid #3d3d3d', height: 44,
        }}>
          <Space>
            <Title level={5} style={{ margin: 0, color: '#4fc3f7', fontSize: 15 }}>Z-CPP</Title>
            <Tag color={backendReady ? 'success' : 'error'} style={{ fontSize: 11, marginLeft: 4 }}>
              {backendReady ? '已连接' : '未连接'}
            </Tag>
          </Space>

          <Space size="small">
            <Tooltip title="新建文件">
              <Button size="small" icon={<FileAddOutlined />} onClick={() => setNewFileModal(true)} />
            </Tooltip>
            <Tooltip title="刷新文件列表">
              <Button size="small" icon={<FolderOpenOutlined />} onClick={refreshFiles} />
            </Tooltip>
            <Tooltip title="编译选项">
              <Button size="small" icon={<CodeOutlined />}
                type={showOptPanel ? 'primary' : 'default'}
                onClick={() => setShowOptPanel(!showOptPanel)} />
            </Tooltip>
            <Tooltip title="设置">
              <Button size="small" icon={<SettingOutlined />} onClick={() => setSettingsOpen(true)} />
            </Tooltip>
          </Space>
        </Header>

        <Layout style={{ height: 'calc(100vh - 44px)' }}>
          {/* 左侧文件浏览器 */}
          <Sider width={180} style={{
            background: '#252526', borderRight: '1px solid #3d3d3d',
            overflow: 'auto',
          }}>
            <div style={{ padding: '8px 10px', color: '#888', fontSize: 12, fontWeight: 500 }}>
              文件
            </div>
            {fileList.map(f => (
              <div key={f.name}
                onClick={() => openFile(f.name)}
                style={{
                  padding: '4px 12px', cursor: 'pointer', fontSize: 13,
                  color: activeTab >= 0 && tabs[activeTab]?.filename === f.name ? '#4fc3f7' : '#ccc',
                  background: activeTab >= 0 && tabs[activeTab]?.filename === f.name ? '#2a2d2e' : 'transparent',
                  borderLeft: activeTab >= 0 && tabs[activeTab]?.filename === f.name ? '2px solid #4fc3f7' : '2px solid transparent',
                }}
              >
                {f.name}
              </div>
            ))}
            {fileList.length === 0 && (
              <div style={{ color: '#555', padding: '12px', fontSize: 12, textAlign: 'center' }}>
                暂无文件
              </div>
            )}
          </Sider>

          {/* 主编辑区 */}
          <Content style={{ display: 'flex', flexDirection: 'column', minWidth: 0 }}>
            {/* 标签栏 */}
            <div style={{
              display: 'flex', background: '#252526',
              borderBottom: '1px solid #3d3d3d', overflowX: 'auto',
            }}>
              {tabs.map((tab, i) => (
                <div key={tab.filename} onClick={() => setActiveTab(i)}
                  style={{
                    display: 'flex', alignItems: 'center', gap: 4,
                    padding: '6px 10px', cursor: 'pointer', fontSize: 13,
                    borderRight: '1px solid #3d3d3d',
                    background: i === activeTab ? '#1e1e1e' : '#2d2d2d',
                    color: i === activeTab ? '#fff' : '#999',
                    borderTop: i === activeTab ? '2px solid #4fc3f7' : '2px solid transparent',
                    whiteSpace: 'nowrap', userSelect: 'none', minWidth: 0,
                  }}
                >
                  {tab.filename}
                  {tab.modified && <span style={{ color: '#4fc3f7' }}>●</span>}
                  <CloseOutlined style={{ fontSize: 10, color: '#666', marginLeft: 2 }}
                    onClick={(e: any) => { e.stopPropagation(); closeTab(i); }} />
                </div>
              ))}
            </div>

            {/* 操作栏 */}
            <div style={{
              padding: '4px 12px', background: '#252526',
              display: 'flex', alignItems: 'center', gap: 8,
              borderBottom: '1px solid #3d3d3d', flexWrap: 'wrap',
            }}>
              <Text style={{ color: '#888', fontSize: 12, flex: 1, minWidth: 60 }}>
                {active?.filename || ''}
              </Text>

              {/* 编译选项面板 */}
              {showOptPanel && (
                <Space size="small" wrap>
                  <Select size="small" value={optLevel} onChange={setOptLevel}
                    style={{ width: 72 }}
                    options={['O0','O1','O2','O3','Os','Ofast'].map(v => ({value:v, label: v}))}
                  />
                  <Select size="small" value={warnings} onChange={setWarnings}
                    style={{ width: 120 }}
                    options={[
                      {value:'Wall', label:'-Wall'},
                      {value:'Wall-Wextra', label:'-Wall -Wextra'},
                      {value:'Wall-Wextra-Werror', label:'-Wall -Wextra -Werror'},
                      {value:'none', label:'无'},
                    ]}
                  />
                  <Select size="small" value={stdVersion} onChange={setStdVersion}
                    style={{ width: 90 }}
                    options={[
                      {value:'', label:'默认'},
                      {value:'c++11', label:'C++11'},{value:'c++14', label:'C++14'},
                      {value:'c++17', label:'C++17'},{value:'c++20', label:'C++20'},
                      {value:'c++23', label:'C++23'},{value:'c11', label:'C11'},{value:'c17', label:'C17'},
                    ]}
                  />
                  <Input size="small" placeholder="额外参数" value={extraFlags}
                    onChange={(e: any) => setExtraFlags(e.target.value)}
                    style={{ width: 120 }} />
                </Space>
              )}

              <Space size="small">
                <span style={{ color: '#888', fontSize: 12 }}>
                  <Switch size="small" checked={compileOnly} onChange={setCompileOnly} /> 仅编译
                </span>
                <Button type="primary" icon={<PlayCircleOutlined />}
                  onClick={handleCompile} loading={compiling} size="small"
                  style={{ background: compiling ? undefined : '#4fc3f7', borderColor: '#4fc3f7' }}
                >
                  {compiling ? '编译中...' : compileOnly ? '编译' : '编译运行'}
                </Button>
              </Space>
            </div>

            {/* 编辑器 */}
            <div style={{ flex: 1, minHeight: 0 }}>
              {renderEditor()}
            </div>
          </Content>

          {/* 右侧输出面板 */}
          <Sider width="35%" style={{
            background: '#1e1e1e', borderLeft: '1px solid #3d3d3d',
            display: 'flex', flexDirection: 'column',
          }}>
            <div style={{
              padding: '6px 12px', background: '#252526',
              borderBottom: '1px solid #3d3d3d',
              display: 'flex', justifyContent: 'space-between', alignItems: 'center',
            }}>
              <Text style={{ color: '#ccc', fontSize: 13 }}>输出</Text>
              {result && (
                <Space size="small">
                  {result.run_time_ms != null && (
                    <Tag style={{ fontSize: 11, margin: 0 }}>{result.run_time_ms} ms</Tag>
                  )}
                  <Button type="text" size="small" icon={<ClearOutlined />}
                    onClick={() => setResult(null)} style={{ color: '#888' }} />
                </Space>
              )}
            </div>

            <div style={{
              flex: 1, overflow: 'auto', padding: '8px 12px',
              fontFamily: "'Cascadia Code','Consolas',monospace", fontSize: 13,
              whiteSpace: 'pre-wrap', color: '#d4d4d4',
            }}>
              {compiling && <div style={{ textAlign:'center', padding: 40 }}><Spin tip="编译中..." /></div>}
              {!compiling && !result && <div style={{ color:'#666', padding:20 }}>点击「编译运行」开始</div>}

              {result && !result.success && (
                <div>
                  <div style={{ color:'#f48771', marginBottom:8 }}>⚠ 编译错误</div>
                  <pre style={{ margin:0, color:'#f48771', fontFamily:'inherit', fontSize:'inherit' }}>{result.compile_output}</pre>
                </div>
              )}

              {result && result.success && (
                <div>
                  <div style={{ color:'#6a9955', marginBottom:8 }}>✓ 编译成功</div>
                  {result.compile_output && result.compile_output.split('\n').filter(l=>l.trim()).map((l,i) => (
                    <div key={i} style={{ color:'#888' }}>{l}</div>
                  ))}
                  {!compileOnly && (
                    <>
                      <div style={{ color:'#569cd6', marginTop:8, marginBottom:4, borderTop:'1px solid #333', paddingTop:8 }}>
                        运行输出
                      </div>
                      <pre style={{ margin:0, color:'#d4d4d4', fontFamily:'inherit', fontSize:'inherit' }}>
                        {result.run_output || '（无输出）'}
                      </pre>
                      {result.exit_code != null && result.exit_code !== 0 && (
                        <div style={{ color:'#f48771', marginTop:8 }}>进程退出，退出码: {result.exit_code}</div>
                      )}
                    </>
                  )}
                </div>
              )}
            </div>
          </Sider>
        </Layout>
      </Layout>

      {/* 新建文件对话框 */}
      <Modal title="新建文件" open={newFileModal} onOk={handleNewFile}
        onCancel={() => { setNewFileModal(false); setNewFileName(''); }}
        okText="创建" cancelText="取消"
      >
        <Input placeholder="文件名（如 test.cpp）" value={newFileName}
          onChange={(e: any) => setNewFileName(e.target.value)}
          onPressEnter={handleNewFile}
        />
        <div style={{ color:'#888', fontSize:12, marginTop:4 }}>
          提示：不写扩展名将自动添加 .cpp
        </div>
      </Modal>

      {/* 设置抽屉 */}
      <Drawer title="设置" open={settingsOpen} onClose={() => setSettingsOpen(false)}
        width={360}
        extra={<Button type="primary" onClick={handleSaveSettings}>保存</Button>}
      >
        <div style={{ marginBottom: 16 }}>
          <div style={{ color:'#ddd', marginBottom:4, fontWeight:500 }}>编译器路径</div>
          <div style={{ marginBottom:8 }}>
            <Text style={{ color:'#888', fontSize:12, display:'block', marginBottom:2 }}>GCC (g++) 路径</Text>
            <Input placeholder="留空 = 使用 PATH" value={editGccPath}
              onChange={(e: any) => setEditGccPath(e.target.value)} size="small" />
          </div>
          <div>
            <Text style={{ color:'#888', fontSize:12, display:'block', marginBottom:2 }}>Clang (clang++) 路径</Text>
            <Input placeholder="留空 = 使用 PATH" value={editClangPath}
              onChange={(e: any) => setEditClangPath(e.target.value)} size="small" />
          </div>
        </div>

        <div style={{ marginBottom: 16 }}>
          <div style={{ color:'#ddd', marginBottom:4, fontWeight:500 }}>工作目录</div>
          <Input placeholder="默认: ./workspace" value={editWorkspace}
            onChange={(e: any) => setEditWorkspace(e.target.value)} size="small" />
        </div>

        <div>
          <Text style={{ color:'#888', fontSize:12 }}>
            {health ? `GCC ${health.gcc_available ? '✓' : '✗'}  |  Clang ${health.clang_available ? '✓' : '✗'}` : '正在检测...'}
          </Text>
        </div>
      </Drawer>
    </AntApp>
  );
};

export default App;
