/// Z-CPP 主应用 — 多文件 IDE
///
/// 功能：文件浏览器 + 多标签编辑器 + 编译选项 + 设置

import React, { useState, useCallback, useEffect, useRef, useMemo } from 'react';
import {
  Layout, Button, Select, Space, Typography, Tag, Switch,
  App as AntApp, Spin, Modal, Input, Drawer, message, Tooltip,
  Collapse, Slider, ConfigProvider, theme,
} from 'antd';
import {
  PlayCircleOutlined, SettingOutlined, ClearOutlined, FileAddOutlined,
  FolderOpenOutlined, CloseOutlined, CodeOutlined, ReloadOutlined,
} from '@ant-design/icons';
import Editor from '@monaco-editor/react';
import * as api from './services/api';

const { Header, Sider, Content } = Layout;
const { Text, Title } = Typography;

// ── 主题色映射 ──────────────────────────────────────

type ThemeKey = 'vs-dark' | 'vs-light' | 'hc-black' | 'monokai' | 'solarized-dark' | 'dracula' | 'nord' | 'gruvbox-dark';

const THEMES: Record<ThemeKey, {
  bg: string; headerBg: string; siderBg: string; border: string;
  text: string; textSec: string; accent: string;
  tabActive: string; tabInactive: string; tabText: string; tabTextInactive: string;
  editorBg: string; outputBg: string; inputBg: string;
  success: string; error: string; info: string;
  alg: typeof theme.darkAlgorithm;
}> = {
  'vs-dark': {
    bg: '#1e1e1e', headerBg: '#2d2d2d', siderBg: '#252526', border: '#3d3d3d',
    text: '#d4d4d4', textSec: '#888', accent: '#4fc3f7',
    tabActive: '#1e1e1e', tabInactive: '#2d2d2d', tabText: '#fff', tabTextInactive: '#999',
    editorBg: '#1e1e1e', outputBg: '#1e1e1e', inputBg: '#2d2d2d',
    success: '#6a9955', error: '#f48771', info: '#569cd6',
    alg: theme.darkAlgorithm,
  },
  'vs-light': {
    bg: '#ffffff', headerBg: '#f3f3f3', siderBg: '#f8f8f8', border: '#e0e0e0',
    text: '#333333', textSec: '#666', accent: '#0066cc',
    tabActive: '#ffffff', tabInactive: '#ececec', tabText: '#333', tabTextInactive: '#888',
    editorBg: '#ffffff', outputBg: '#ffffff', inputBg: '#ffffff',
    success: '#2e7d32', error: '#c62828', info: '#1565c0',
    alg: theme.defaultAlgorithm,
  },
  'hc-black': {
    bg: '#000000', headerBg: '#0d0d0d', siderBg: '#0a0a0a', border: '#6fc3df',
    text: '#ffffff', textSec: '#6fc3df', accent: '#6fc3df',
    tabActive: '#000000', tabInactive: '#1a1a1a', tabText: '#fff', tabTextInactive: '#6fc3df',
    editorBg: '#000000', outputBg: '#000000', inputBg: '#1a1a1a',
    success: '#6fc3df', error: '#f48771', info: '#6fc3df',
    alg: theme.darkAlgorithm,
  },
  'monokai': {
    bg: '#272822', headerBg: '#3e3d32', siderBg: '#2d2e27', border: '#49483e',
    text: '#f8f8f2', textSec: '#75715e', accent: '#a6e22e',
    tabActive: '#272822', tabInactive: '#3e3d32', tabText: '#f8f8f2', tabTextInactive: '#75715e',
    editorBg: '#272822', outputBg: '#272822', inputBg: '#3e3d32',
    success: '#a6e22e', error: '#f92672', info: '#66d9ef',
    alg: theme.darkAlgorithm,
  },
  'solarized-dark': {
    bg: '#002b36', headerBg: '#073642', siderBg: '#073642', border: '#586e75',
    text: '#839496', textSec: '#657b83', accent: '#268bd2',
    tabActive: '#002b36', tabInactive: '#073642', tabText: '#93a1a1', tabTextInactive: '#586e75',
    editorBg: '#002b36', outputBg: '#002b36', inputBg: '#073642',
    success: '#859900', error: '#dc322f', info: '#2aa198',
    alg: theme.darkAlgorithm,
  },
  'dracula': {
    bg: '#282a36', headerBg: '#343746', siderBg: '#2e303e', border: '#44475a',
    text: '#f8f8f2', textSec: '#6272a4', accent: '#bd93f9',
    tabActive: '#282a36', tabInactive: '#343746', tabText: '#f8f8f2', tabTextInactive: '#6272a4',
    editorBg: '#282a36', outputBg: '#282a36', inputBg: '#343746',
    success: '#50fa7b', error: '#ff5555', info: '#8be9fd',
    alg: theme.darkAlgorithm,
  },
  'nord': {
    bg: '#2e3440', headerBg: '#3b4252', siderBg: '#3b4252', border: '#4c566a',
    text: '#d8dee9', textSec: '#81a1c1', accent: '#88c0d0',
    tabActive: '#2e3440', tabInactive: '#3b4252', tabText: '#d8dee9', tabTextInactive: '#4c566a',
    editorBg: '#2e3440', outputBg: '#2e3440', inputBg: '#3b4252',
    success: '#a3be8c', error: '#bf616a', info: '#5e81ac',
    alg: theme.darkAlgorithm,
  },
  'gruvbox-dark': {
    bg: '#282828', headerBg: '#3c3836', siderBg: '#32302f', border: '#504945',
    text: '#ebdbb2', textSec: '#a89984', accent: '#fabd2f',
    tabActive: '#282828', tabInactive: '#3c3836', tabText: '#ebdbb2', tabTextInactive: '#665c54',
    editorBg: '#282828', outputBg: '#282828', inputBg: '#3c3836',
    success: '#b8bb26', error: '#fb4934', info: '#83a598',
    alg: theme.darkAlgorithm,
  },
};

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
  const [editFontFamily, setEditFontFamily] = useState('');
  const [editFontSize, setEditFontSize] = useState(14);
  const [editTabSize, setEditTabSize] = useState(4);
  const [editTheme, setEditTheme] = useState<ThemeKey>('vs-dark');
  const [editWordWrap, setEditWordWrap] = useState<'off' | 'on' | 'wordWrapColumn'>('off');
  const [editAutoSave, setEditAutoSave] = useState(false);
  const [editBackgroundImage, setEditBackgroundImage] = useState('');
  const [editOpacity, setEditOpacity] = useState(1.0);
  const [editFrostedGlass, setEditFrostedGlass] = useState(false);
  const [editBlurAmount, setEditBlurAmount] = useState(10);
  const [editBackgroundOpacity, setEditBackgroundOpacity] = useState(1.0);
  const [editDefaultCompileOnly, setEditDefaultCompileOnly] = useState(false);

  // 状态
  const [health, setHealth] = useState<api.HealthResponse | null>(null);
  const [backendReady, setBackendReady] = useState(false);
  const [showOptPanel, setShowOptPanel] = useState(false);
  const [newFileName, setNewFileName] = useState('');
  const [newFileModal, setNewFileModal] = useState(false);
  const [appMeta, setAppMeta] = useState<api.AppMeta | null>(null);
  const [systemFonts, setSystemFonts] = useState<string[]>([]);

  const autoSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [liveBackground, setLiveBackground] = useState('');
  const [siderWidth, setSiderWidth] = useState(180);
  const siderDragRef = useRef<{ startX: number; startW: number } | null>(null);
  const [inputText, setInputText] = useState('');
  const [currentSubdir, setCurrentSubdir] = useState('');

  const currentTheme = (settings?.editor?.theme as ThemeKey) || 'vs-dark';
  const t = THEMES[currentTheme] || THEMES['vs-dark'];
  const hasBg = !!liveBackground;

  // 有背景图时各面板使用极低透明度背景
  const panelBg = hasBg ? 'rgba(0,0,0,0.08)' : t.bg;
  const headerBg = hasBg ? 'rgba(0,0,0,0.15)' : t.headerBg;
  const siderBg = hasBg ? 'rgba(0,0,0,0.10)' : t.siderBg;

  const active = tabs[activeTab];

  // ── 初始化 ──────────────────────────────────────────

  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const init = async () => {
      try {
        const h = await api.checkHealth();
        if (cancelled) return;
        setHealth(h);
        setBackendReady(true);
        const s = await api.getSettings();
        if (cancelled) return;
        setSettings(s);
        setCompiler(s.default_compiler as 'gcc' | 'clang');
        setOptLevel(s.default_options.optimization || 'O2');
        setWarnings(s.default_options.warnings || 'Wall-Wextra');
        setStdVersion(s.default_options.standard || '');
        setExtraFlags(s.default_options.extra_flags || '');
        setCompileOnly(s.default_compile_only ?? false);
        setEditGccPath(s.gcc_path);
        setEditClangPath(s.clang_path);
        setEditWorkspace(s.workspace);
        setEditFontFamily(s.editor?.font_family ?? '');
        setEditFontSize(s.editor?.font_size ?? 14);
        setEditTabSize(s.editor?.tab_size ?? 4);
        setEditTheme((s.editor?.theme as 'vs-dark' | 'vs-light' | 'hc-black') ?? 'vs-dark');
        setEditWordWrap((s.editor?.word_wrap as 'off' | 'on' | 'wordWrapColumn') ?? 'off');
        setEditAutoSave(s.auto_save ?? false);
        setEditBackgroundImage(s.appearance?.background_image ?? '');
        setLiveBackground(s.appearance?.background_image ?? '');
        setEditOpacity(s.appearance?.opacity ?? 1.0);
        setEditFrostedGlass(s.appearance?.frosted_glass ?? false);
        setEditBlurAmount(s.appearance?.blur_amount ?? 10);
        setEditBackgroundOpacity(s.appearance?.background_opacity ?? 1.0);
        setEditDefaultCompileOnly(s.default_compile_only ?? false);
        refreshFiles();
        try {
          const meta = await api.getAppMeta();
          if (!cancelled) setAppMeta(meta);
        } catch { /* ignore */ }
        // 字体扫描延迟加载，不阻塞初始化
        setTimeout(async () => {
          try {
            const fonts = await api.getSystemFonts();
            if (!cancelled) setSystemFonts(fonts);
          } catch { /* ignore */ }
        }, 2000);
      } catch {
        if (cancelled) return;
        setBackendReady(false);
        message.warning('后端未连接，请启动 cargo run');
        timer = setTimeout(init, 2000);
      }
    };
    init();
    return () => { cancelled = true; clearTimeout(timer); };
  }, []);

  // ── 外观效果 ────────────────────────────────────────

  useEffect(() => {
    if (settings?.appearance?.opacity != null) {
      document.documentElement.style.opacity = String(settings.appearance.opacity);
    }
  }, [settings?.appearance?.opacity]);

  // ── 主题同步 ─────────────────────────────────────────
  useEffect(() => {
    document.documentElement.dataset.theme = currentTheme;
  }, [currentTheme]);

  // ── 侧栏拖拽 ────────────────────────────────────────

  const onSiderDragStart = useCallback((e: React.MouseEvent) => {
    siderDragRef.current = { startX: e.clientX, startW: siderWidth };
    const onMove = (ev: MouseEvent) => {
      if (!siderDragRef.current) return;
      const delta = ev.clientX - siderDragRef.current.startX;
      setSiderWidth(Math.max(120, Math.min(400, siderDragRef.current.startW + delta)));
    };
    const onUp = () => {
      siderDragRef.current = null;
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
    };
    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }, [siderWidth]);

  // ── 快捷键 ──────────────────────────────────────────

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const ctrl = e.ctrlKey || e.metaKey;
      // Ctrl+S — 保存
      if (ctrl && e.key === 's') {
        e.preventDefault();
        const tab = tabs[activeTabRef.current];
        if (tab) api.saveFile(tab.filename, tab.code).then(ok => {
          if (ok) setTabs(prev => prev.map((t, i) => i === activeTabRef.current ? { ...t, modified: false } : t));
        });
      }
      // Ctrl+N — 新建文件
      if (ctrl && e.key === 'n') { e.preventDefault(); setNewFileModal(true); }
      // Ctrl+W — 关闭标签
      if (ctrl && e.key === 'w') { e.preventDefault(); closeTab(activeTabRef.current); }
      // F5 — 编译运行
      if (e.key === 'F5') { e.preventDefault(); handleCompile(); }
      // Ctrl+Shift+B — 仅编译
      if (ctrl && e.shiftKey && e.key === 'B') {
        e.preventDefault();
        setCompileOnly(true);
        setTimeout(() => handleCompile(), 0);
      }
      // Ctrl+, — 打开设置
      if (ctrl && e.key === ',') { e.preventDefault(); setSettingsOpen(true); }
      // Ctrl+Tab — 下一个标签
      if (ctrl && e.key === 'Tab') {
        e.preventDefault();
        setTabs(prev => {
          const next = e.shiftKey
            ? (activeTabRef.current - 1 + prev.length) % prev.length
            : (activeTabRef.current + 1) % prev.length;
          setActiveTab(next);
          return prev;
        });
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  });

  const refreshFiles = async (subdir?: string) => {
    try {
      const res = await api.listFiles(subdir ?? currentSubdir);
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
    setTabs(prev => {
      setActiveTab(prev.length);
      return [...prev, { filename, code, modified: false, language: lang }];
    });
  };

  const closeTab = (idx: number) => {
    setTabs(prev => {
      if (prev.length === 0) return prev;
      const next = prev.filter((_, i) => i !== idx);
      if (next.length === 0) {
        setActiveTab(0);
        return [{ filename: 'untitled.cpp', code: '', modified: false, language: 'cpp' as const }];
      }
      setActiveTab(a => (a >= idx && a > 0) ? a - 1 : a);
      return next;
    });
  };

  const closeAllTabs = () => {
    setTabs([{ filename: 'untitled.cpp', code: '', modified: false, language: 'cpp' as const }]);
    setActiveTab(0);
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
    await refreshFiles();
    await openFile(fullName);
  };

  const activeTabRef = useRef(activeTab);
  activeTabRef.current = activeTab;
  const tabsRef = useRef(tabs);
  tabsRef.current = tabs;
  const autoSaveRef = useRef(settings?.auto_save ?? false);
  autoSaveRef.current = settings?.auto_save ?? false;

  const handleCodeChange = useCallback((val: string | undefined) => {
    const code = val || '';
    const idx = activeTabRef.current;
    setTabs(prev => prev.map((t, i) => i === idx ? { ...t, code, modified: true } : t));

    if (autoSaveRef.current) {
      if (autoSaveTimerRef.current) clearTimeout(autoSaveTimerRef.current);
      autoSaveTimerRef.current = setTimeout(async () => {
        const tab = tabsRef.current[idx];
        if (tab?.filename) {
          const ok = await api.saveFile(tab.filename, code);
          if (ok) setTabs(p => p.map((t, i) => i === idx ? { ...t, modified: false } : t));
        }
      }, 1000);
    }
  }, []);

  // ── 编译 ────────────────────────────────────────────

  const handleCompile = async () => {
    if (!active) return;
    if (!backendReady) { message.error('后端未连接'); return; }

    setCompiling(true);
    setResult(null);
    try {
      const saved = await api.saveFile(active.filename, active.code);
      if (!saved) { message.error('文件保存失败'); setCompiling(false); return; }
      setTabs(prev => prev.map((t, i) => i === activeTab ? { ...t, modified: false } : t));

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
        input_text: inputText,
      });
      setResult(res);
      if (res.success) message.success(compileOnly ? '编译成功' : '编译运行成功');
      else message.error('编译失败');
    } catch (e: unknown) {
      message.error(`请求失败: ${e instanceof Error ? e.message : String(e)}`);
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
      editor: {
        font_family: editFontFamily,
        font_size: editFontSize,
        tab_size: editTabSize,
        theme: editTheme,
        word_wrap: editWordWrap,
      },
      appearance: {
        background_image: editBackgroundImage,
        opacity: editOpacity,
        frosted_glass: editFrostedGlass,
        blur_amount: editBlurAmount,
        background_opacity: editBackgroundOpacity,
      },
      auto_save: editAutoSave,
      default_compile_only: editDefaultCompileOnly,
    };
    let ok = false;
    try {
      ok = await api.saveSettings(newSettings);
    } catch (e) {
      message.error(`保存失败: ${e instanceof Error ? e.message : String(e)}`);
      return;
    }
    if (ok) {
      setSettings(newSettings);
      setSettingsOpen(false);
      message.success('设置已保存');
      try {
        const h = await api.checkHealth();
        setHealth(h);
      } catch { /* health check failed after settings change */ }
    } else {
      message.error('保存失败');
    }
  };

  // ── 渲染 ────────────────────────────────────────────

  const editorOptions = useMemo(() => ({
    fontSize: settings?.editor?.font_size ?? 14,
    fontFamily: settings?.editor?.font_family || "'Cascadia Code', 'Fira Code', 'Consolas', monospace",
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    lineNumbers: 'on' as const,
    automaticLayout: true,
    tabSize: settings?.editor?.tab_size ?? 4,
    wordWrap: (settings?.editor?.word_wrap ?? 'off') as 'off' | 'on' | 'wordWrapColumn',
    padding: { top: 8 },
  }), [settings?.editor?.font_size, settings?.editor?.font_family, settings?.editor?.tab_size, settings?.editor?.word_wrap]);

  // Monaco 只支持 vs, vs-dark, hc-black 三个基础主题
  const monacoBaseTheme = currentTheme === 'vs-light' ? 'vs' : currentTheme === 'hc-black' ? 'hc-black' : 'vs-dark';

  const handleEditorWillMount = useCallback((monaco: unknown) => {
    const m = monaco as { editor: { defineTheme: (n: string, d: unknown) => void } };
    // 始终定义透明主题，有背景图时使用
    m.editor.defineTheme('zcpp-bg', {
      base: monacoBaseTheme,
      inherit: true,
      rules: [],
      colors: {
        'editor.background': '#00000000',
        'editorGutter.background': '#00000000',
      },
    });
  }, [monacoBaseTheme]);

  const renderEditor = () => {
    if (!active) return null;
    return (
      <Editor
        height="100%"
        language={active.language}
        theme={hasBg ? 'zcpp-bg' : monacoBaseTheme}
        value={active.code}
        onChange={handleCodeChange}
        beforeMount={handleEditorWillMount}
        options={editorOptions}
      />
    );
  };

  // 有背景图时，让 Ant Design 组件容器也透明
  const cfgBg = hasBg ? 'transparent' : t.inputBg;
  const cfgElevated = hasBg ? 'transparent' : t.siderBg;

  return (
    <ConfigProvider theme={{
      algorithm: t.alg,
      token: {
        colorPrimary: t.accent,
        colorBgContainer: cfgBg,
        colorBgElevated: cfgElevated,
        colorBgLayout: hasBg ? 'transparent' : t.bg,
        colorBorder: t.border,
        colorBorderSecondary: t.border,
        colorText: t.text,
        colorTextSecondary: t.textSec,
        colorTextPlaceholder: t.textSec,
        colorBgTextHover: `${t.accent}15`,
        colorBgTextActive: `${t.accent}25`,
      },
      components: {
        Drawer: {
          colorBgElevated: hasBg ? 'rgba(0,0,0,0.85)' : t.siderBg,
          colorIcon: t.textSec,
          colorIconHover: t.text,
          colorText: t.text,
          colorTextHeading: t.text,
        },
        Input: {
          colorBgContainer: cfgBg,
          colorBorder: t.border,
          colorText: t.text,
          colorTextPlaceholder: t.textSec,
          activeBorderColor: t.accent,
          hoverBorderColor: t.accent,
        },
        Select: {
          colorBgContainer: cfgBg,
          colorBgElevated: cfgElevated,
          colorBorder: t.border,
          colorText: t.text,
          colorTextPlaceholder: t.textSec,
          optionSelectedBg: `${t.accent}22`,
          optionActiveBg: `${t.accent}11`,
        },
        Slider: {
          colorPrimary: t.accent,
          colorPrimaryBorder: t.accent,
          trackBg: t.accent,
          trackHoverBg: t.accent,
          handleColor: t.accent,
          handleActiveColor: t.accent,
          dotActiveBorderColor: t.accent,
          railBg: t.border,
          railHoverBg: t.border,
        },
        Switch: {
          colorPrimary: t.accent,
          colorPrimaryHover: t.accent,
        },
        Collapse: {
          colorBgContainer: 'transparent',
          colorText: t.text,
          colorTextHeading: t.text,
          colorBorder: t.border,
        },
        Modal: {
          contentBg: hasBg ? 'rgba(0,0,0,0.85)' : t.siderBg,
          headerBg: hasBg ? 'rgba(0,0,0,0.85)' : t.siderBg,
          titleColor: t.text,
          colorIcon: t.textSec,
          colorIconHover: t.text,
        },
        Button: {
          defaultBg: cfgBg,
          defaultBorderColor: t.border,
          defaultColor: t.text,
          primaryColor: '#fff',
        },
        Tag: {
          colorText: t.text,
        },
        Spin: {
          colorPrimary: t.accent,
        },
      },
    }}>
    <AntApp>
      <Layout style={{
        height: '100vh',
        position: 'relative',
        background: hasBg ? 'transparent' : (settings?.appearance?.frosted_glass ? 'rgba(30,30,30,0.7)' : t.bg),
        backdropFilter: settings?.appearance?.frosted_glass ? `blur(${settings.appearance.blur_amount}px)` : undefined,
        WebkitBackdropFilter: settings?.appearance?.frosted_glass ? `blur(${settings.appearance.blur_amount}px)` : undefined,
      }}>
        {/* 背景图层 */}
        {hasBg && (
          <div style={{
            position: 'absolute', inset: 0, zIndex: 0,
            backgroundImage: `url(${liveBackground})`,
            backgroundSize: 'cover', backgroundPosition: 'center',
            opacity: settings?.appearance?.background_opacity ?? 1.0,
            pointerEvents: 'none',
          }} />
        )}
        {/* 顶部栏 */}
        <Header style={{
          background: headerBg, padding: '0 12px', display: 'flex',
          alignItems: 'center', justifyContent: 'space-between',
          borderBottom: `1px solid ${t.border}`, height: 44,
          position: 'relative', zIndex: 1,
        }}>
          <Space>
            <Title level={5} style={{ margin: 0, color: t.accent, fontSize: 15 }}>Z-CPP</Title>
            <Tag color={backendReady ? 'success' : 'error'} style={{ fontSize: 11, marginLeft: 4 }}>
              {backendReady ? '已连接' : '未连接'}
            </Tag>
          </Space>

          <Space size="small">
            <Tooltip title="新建文件">
              <Button size="small" icon={<FileAddOutlined />} onClick={() => setNewFileModal(true)}
                style={{ color: t.accent, borderColor: t.accent }} />
            </Tooltip>
            <Tooltip title="切换工作目录">
              <Button size="small" icon={<FolderOpenOutlined />} style={{ color: t.accent, borderColor: t.accent }}
                onClick={async () => {
                  try {
                    const { open } = await import('@tauri-apps/plugin-dialog');
                    const selected = await open({ directory: true, title: '选择工作目录' });
                    if (selected && typeof selected === 'string') {
                      setEditWorkspace(selected);
                      const newSettings = { ...settings!, workspace: selected };
                      const ok = await api.saveSettings(newSettings);
                      if (ok) { setSettings(newSettings); refreshFiles(); message.success('工作目录已切换'); }
                    }
                  } catch { message.warning('无法打开目录选择器'); }
                }} />
            </Tooltip>
            <Tooltip title="编译选项">
              <Button size="small" icon={<CodeOutlined />}
                type={showOptPanel ? 'primary' : 'default'}
                onClick={() => setShowOptPanel(!showOptPanel)}
                style={showOptPanel ? {} : { color: t.accent, borderColor: t.accent }} />
            </Tooltip>
            <Tooltip title="设置">
              <Button size="small" icon={<SettingOutlined />} onClick={() => setSettingsOpen(true)}
                style={{ color: t.accent, borderColor: t.accent }} />
            </Tooltip>
          </Space>
        </Header>

        <Layout style={{ height: 'calc(100vh - 44px)', position: 'relative', zIndex: 1 }}>
          {/* 左侧文件浏览器 */}
          <Sider width={siderWidth} style={{
            background: siderBg, borderRight: `1px solid ${t.border}`,
            overflow: 'auto', position: 'relative',
          }}>
            <div style={{ padding: '8px 10px', color: t.textSec, fontSize: 12, fontWeight: 500, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                {currentSubdir ? (
                  <Button type="text" size="small" icon={<FolderOpenOutlined />}
                    onClick={() => {
                      const parent = currentSubdir.replace(/[/\\][^/\\]+$/, '');
                      setCurrentSubdir(parent);
                      refreshFiles(parent);
                    }}
                    style={{ color: t.textSec, padding: 0, minWidth: 16, height: 16, fontSize: 11 }}
                  />
                ) : null}
                <span>{currentSubdir || '文件'}</span>
              </span>
              <Tooltip title="刷新">
                <Button type="text" size="small" icon={<ReloadOutlined />}
                  onClick={() => refreshFiles()} style={{ color: '#888', padding: 0, minWidth: 20, height: 20 }} />
              </Tooltip>
            </div>
            {fileList.map(f => {
              const fullPath = currentSubdir ? `${currentSubdir}/${f.name}` : f.name;
              const isActive = activeTab >= 0 && tabs[activeTab]?.filename === fullPath;
              return (
                <div key={f.name}
                  onClick={() => f.is_dir ? (() => {
                    const newSub = currentSubdir ? `${currentSubdir}/${f.name}` : f.name;
                    setCurrentSubdir(newSub);
                    refreshFiles(newSub);
                  })() : openFile(fullPath)}
                  style={{
                    padding: '4px 12px', cursor: 'pointer', fontSize: 13,
                    color: f.is_dir ? t.accent : isActive ? t.accent : t.text,
                    background: isActive ? `${t.accent}15` : 'transparent',
                    borderLeft: isActive ? `2px solid ${t.accent}` : '2px solid transparent',
                    fontWeight: f.is_dir ? 500 : 400,
                  }}
                >
                  {f.is_dir ? `📁 ${f.name}` : f.name}
                </div>
              );
            })}
            {fileList.length === 0 && (
              <div style={{ color: t.textSec, padding: '12px', fontSize: 12, textAlign: 'center' }}>
                暂无文件
              </div>
            )}
          </Sider>
          <div className="sider-resizer" onMouseDown={onSiderDragStart} />

          {/* 主编辑区 */}
          <Content style={{ display: 'flex', flexDirection: 'column', minWidth: 0 }}>
            {/* 标签栏 */}
            <div style={{
              display: 'flex', background: siderBg,
              borderBottom: `1px solid ${t.border}`, overflowX: 'auto',
            }}>
              {tabs.map((tab, i) => (
                <div key={`${tab.filename}-${i}`} onClick={() => setActiveTab(i)}
                  style={{
                    display: 'flex', alignItems: 'center', gap: 4,
                    padding: '6px 10px', cursor: 'pointer', fontSize: 13,
                    borderRight: `1px solid ${t.border}`,
                    background: i === activeTab ? t.tabActive : t.tabInactive,
                    color: i === activeTab ? t.tabText : t.tabTextInactive,
                    borderTop: i === activeTab ? `2px solid ${t.accent}` : '2px solid transparent',
                    whiteSpace: 'nowrap', userSelect: 'none', minWidth: 0,
                  }}
                >
                  {tab.filename}
                  {tab.modified && <span style={{ color: t.accent }}>●</span>}
                  <CloseOutlined style={{ fontSize: 10, color: t.textSec, marginLeft: 2 }}
                    onClick={(e: React.MouseEvent) => { e.stopPropagation(); closeTab(i); }} />
                </div>
              ))}
              {tabs.length > 1 && (
                <Tooltip title="关闭全部">
                  <CloseOutlined style={{ fontSize: 11, color: t.textSec, padding: '6px 8px', cursor: 'pointer' }}
                    onClick={closeAllTabs} />
                </Tooltip>
              )}
            </div>

            {/* 操作栏 */}
            <div style={{
              padding: '4px 12px', background: siderBg,
              display: 'flex', alignItems: 'center', gap: 8,
              borderBottom: `1px solid ${t.border}`, flexWrap: 'wrap',
            }}>
              <Text style={{ color: t.textSec, fontSize: 12, flex: 1, minWidth: 60 }}>
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
                    onChange={(e: React.ChangeEvent<HTMLInputElement>) => setExtraFlags(e.target.value)}
                    style={{ width: 120 }} />
                </Space>
              )}

              <Space size="small">
                <span style={{ color: t.textSec, fontSize: 12 }}>
                  <Switch size="small" checked={compileOnly} onChange={setCompileOnly} /> 仅编译
                </span>
                <Button type="primary" icon={<PlayCircleOutlined />}
                  onClick={handleCompile} loading={compiling} size="small"
                  style={{ background: compiling ? undefined : t.accent, borderColor: t.accent }}
                >
                  {compiling ? '编译中...' : compileOnly ? '编译' : '编译运行'}
                </Button>
              </Space>
            </div>

            {/* 编辑器 */}
            <div style={{ flex: 1, minHeight: 0, position: 'relative' }}>
              {renderEditor()}
            </div>
          </Content>

          {/* 右侧输出面板 */}
          <Sider width="35%" style={{
            background: panelBg, borderLeft: `1px solid ${t.border}`,
            display: 'flex', flexDirection: 'column',
          }}>
            <div style={{
              padding: '6px 12px', background: siderBg,
              borderBottom: `1px solid ${t.border}`,
              display: 'flex', justifyContent: 'space-between', alignItems: 'center',
            }}>
              <Text style={{ color: t.text, fontSize: 13 }}>输出</Text>
              {result && (
                <Space size="small">
                  {result.run_time_ms != null && (
                    <Tag style={{ fontSize: 11, margin: 0 }}>{result.run_time_ms} ms</Tag>
                  )}
                  <Button type="text" size="small" icon={<ClearOutlined />}
                    onClick={() => setResult(null)} style={{ color: t.textSec }} />
                </Space>
              )}
            </div>

            <div style={{
              flex: 1, overflow: 'auto', padding: '8px 12px',
              fontFamily: "'Cascadia Code','Consolas',monospace", fontSize: 13,
              whiteSpace: 'pre-wrap', color: t.text,
            }}>
              {compiling && <div style={{ textAlign:'center', padding: 40 }}><Spin tip="编译中..." /></div>}
              {!compiling && !result && <div style={{ color: t.textSec, padding:20 }}>点击「编译运行」开始</div>}

              {result && !result.success && (
                <div>
                  <div style={{ color: t.error, marginBottom:8 }}>⚠ 编译错误</div>
                  <pre style={{ margin:0, color: t.error, fontFamily:'inherit', fontSize:'inherit' }}>{result.compile_output}</pre>
                </div>
              )}

              {result && result.success && (
                <div>
                  <div style={{ color: t.success, marginBottom:8 }}>✓ 编译成功</div>
                  {result.compile_output && result.compile_output.split('\n').filter(l=>l.trim()).map((l,i) => (
                    <div key={i} style={{ color: t.textSec }}>{l}</div>
                  ))}
                  {!compileOnly && (
                    <>
                      <div style={{ color: t.info, marginTop:8, marginBottom:4, borderTop: `1px solid ${t.border}`, paddingTop:8 }}>
                        运行输出
                      </div>
                      <pre style={{ margin:0, color: t.text, fontFamily:'inherit', fontSize:'inherit' }}>
                        {result.run_output || '（无输出）'}
                      </pre>
                      {result.exit_code != null && result.exit_code !== 0 && (
                        <div style={{ color: t.error, marginTop:8 }}>进程退出，退出码: {result.exit_code}</div>
                      )}
                    </>
                  )}
                </div>
              )}
            </div>

            {/* 程序输入 */}
            <div style={{
              padding: '4px 12px', borderTop: `1px solid ${t.border}`, background: siderBg,
              display: 'flex', flexDirection: 'column', gap: 4,
            }}>
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <Text style={{ color: t.textSec, fontSize: 11 }}>输入（多行，编译前填写）</Text>
                {inputText && (
                  <Button type="text" size="small" icon={<ClearOutlined />}
                    onClick={() => setInputText('')} style={{ color: t.textSec, padding: 0 }} />
                )}
              </div>
              <Input.TextArea
                placeholder="每行对应一次 stdin 输入"
                value={inputText}
                onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setInputText(e.target.value)}
                autoSize={{ minRows: 2, maxRows: 6 }}
                style={{ fontSize: 12 }}
              />
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
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => setNewFileName(e.target.value)}
          onPressEnter={handleNewFile}
        />
        <div style={{ color: t.textSec, fontSize:12, marginTop:4 }}>
          提示：不写扩展名将自动添加 .cpp
        </div>
      </Modal>

      {/* 设置抽屉 */}
      <Drawer title="设置" open={settingsOpen} onClose={() => setSettingsOpen(false)}
        width={480}
        extra={<Button type="primary" onClick={handleSaveSettings}>保存</Button>}
      >
        <Collapse defaultActiveKey={['compiler', 'editor', 'appearance']} ghost
          items={[
            {
              key: 'compiler',
              label: <span style={{ color: t.text, fontWeight: 500 }}>编译器</span>,
              children: (
                <>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>GCC (g++) 路径</Text>
                    <Input placeholder="留空 = 使用 PATH" value={editGccPath}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setEditGccPath(e.target.value)} size="small" />
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>Clang (clang++) 路径</Text>
                    <Input placeholder="留空 = 使用 PATH" value={editClangPath}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setEditClangPath(e.target.value)} size="small" />
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>默认编译器</Text>
                    <Select size="small" value={compiler} onChange={setCompiler} style={{ width: '100%' }}
                      options={[{ value: 'gcc', label: 'GCC' }, { value: 'clang', label: 'Clang' }]} />
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>额外编译参数</Text>
                    <Input placeholder="如 -lm -pthread" value={extraFlags}
                      onChange={(e: React.ChangeEvent<HTMLInputElement>) => setExtraFlags(e.target.value)} size="small" />
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <Text style={{ color: t.textSec, fontSize:12 }}>默认仅编译</Text>
                    <Switch size="small" checked={editDefaultCompileOnly} onChange={setEditDefaultCompileOnly} />
                  </div>
                </>
              ),
            },
            {
              key: 'workspace',
              label: <span style={{ color: t.text, fontWeight: 500 }}>工作目录</span>,
              children: (
                <Input placeholder="默认: ./workspace" value={editWorkspace}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => setEditWorkspace(e.target.value)} size="small" />
              ),
            },
            {
              key: 'editor',
              label: <span style={{ color: t.text, fontWeight: 500 }}>编辑器</span>,
              children: (
                <>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>字体</Text>
                    <Select size="small"
                      value={editFontFamily || "'Cascadia Code', 'Fira Code', 'Consolas', monospace"}
                      onChange={setEditFontFamily}
                      style={{ width: '100%' }}
                      showSearch
                      filterOption={(input: string, option?: { label?: React.ReactNode; value: string }) =>
                        (option?.label as string)?.toLowerCase().includes(input.toLowerCase()) ?? false
                      }
                      options={systemFonts.length > 0
                        ? systemFonts.map(f => ({ value: `'${f}', monospace`, label: f }))
                        : [
                            { value: "'Cascadia Code', 'Fira Code', 'Consolas', monospace", label: 'Cascadia Code' },
                            { value: "'Fira Code', 'Consolas', monospace", label: 'Fira Code' },
                            { value: "'JetBrains Mono', 'Consolas', monospace", label: 'JetBrains Mono' },
                            { value: "'Source Code Pro', 'Consolas', monospace", label: 'Source Code Pro' },
                            { value: "Consolas, monospace", label: 'Consolas' },
                            { value: "'Courier New', monospace", label: 'Courier New' },
                          ]
                      } />
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>字号: {editFontSize}</Text>
                    <Slider min={8} max={32} value={editFontSize} onChange={setEditFontSize} />
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>Tab 大小: {editTabSize}</Text>
                    <Slider min={1} max={8} value={editTabSize} onChange={setEditTabSize} />
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>主题</Text>
                    <Select size="small" value={editTheme} onChange={setEditTheme} style={{ width: '100%' }}
                      options={[
                        { value: 'vs-dark', label: 'Dark' },
                        { value: 'vs-light', label: 'Light' },
                        { value: 'hc-black', label: 'High Contrast' },
                        { value: 'monokai', label: 'Monokai' },
                        { value: 'solarized-dark', label: 'Solarized Dark' },
                        { value: 'dracula', label: 'Dracula' },
                        { value: 'nord', label: 'Nord' },
                        { value: 'gruvbox-dark', label: 'Gruvbox Dark' },
                      ]} />
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>自动换行</Text>
                    <Select size="small" value={editWordWrap} onChange={setEditWordWrap} style={{ width: '100%' }}
                      options={[
                        { value: 'off', label: '关闭' },
                        { value: 'on', label: '开启' },
                        { value: 'wordWrapColumn', label: '按列换行' },
                      ]} />
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <Text style={{ color: t.textSec, fontSize:12 }}>自动保存</Text>
                    <Switch size="small" checked={editAutoSave} onChange={setEditAutoSave} />
                  </div>
                </>
              ),
            },
            {
              key: 'appearance',
              label: <span style={{ color: t.text, fontWeight: 500 }}>外观</span>,
              children: (
                <>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>背景图片</Text>
                    <Space>
                      <Button size="small" onClick={() => {
                        const input = document.createElement('input');
                        input.type = 'file';
                        input.accept = 'image/*';
                        input.onchange = () => {
                          const file = input.files?.[0];
                          if (!file) return;
                          if (file.size > 2 * 1024 * 1024) {
                            message.warning('图片大小建议不超过 2MB');
                          }
                          const reader = new FileReader();
                          reader.onload = () => {
                            const data = reader.result as string;
                            setEditBackgroundImage(data);
                            setLiveBackground(data);
                          };
                          reader.readAsDataURL(file);
                        };
                        input.click();
                      }}>选择图片</Button>
                      {editBackgroundImage && (
                        <Button size="small" danger onClick={() => { setEditBackgroundImage(''); setLiveBackground(''); }}>清除</Button>
                      )}
                    </Space>
                    {editBackgroundImage && (
                      <div style={{ marginTop: 8 }}>
                        <div style={{
                          width: '100%', height: 80, borderRadius: 4,
                          backgroundImage: `url(${editBackgroundImage})`,
                          backgroundSize: 'cover', backgroundPosition: 'center',
                          border: `1px solid ${t.border}`,
                        }} />
                      </div>
                    )}
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>
                      背景图透明度: {editBackgroundOpacity.toFixed(2)}
                    </Text>
                    <Slider min={0} max={1} step={0.05} value={editBackgroundOpacity} onChange={setEditBackgroundOpacity} />
                  </div>
                  <div style={{ marginBottom: 12 }}>
                    <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>
                      窗口不透明度: {editOpacity.toFixed(2)}
                    </Text>
                    <Slider min={0.1} max={1.0} step={0.05} value={editOpacity} onChange={setEditOpacity} />
                  </div>
                  <div style={{ marginBottom: 12, display: 'flex', alignItems: 'center', gap: 8 }}>
                    <Text style={{ color: t.textSec, fontSize:12 }}>毛玻璃效果</Text>
                    <Switch size="small" checked={editFrostedGlass} onChange={setEditFrostedGlass} />
                  </div>
                  {editFrostedGlass && (
                    <div style={{ marginBottom: 12 }}>
                      <Text style={{ color: t.textSec, fontSize:12, display:'block', marginBottom:2 }}>
                        模糊程度: {editBlurAmount}px
                      </Text>
                      <Slider min={0} max={30} value={editBlurAmount} onChange={setEditBlurAmount} />
                    </div>
                  )}
                </>
              ),
            },
            {
              key: 'about',
              label: <span style={{ color: t.text, fontWeight: 500 }}>关于</span>,
              children: (
                <>
                  <div style={{ marginBottom: 8 }}>
                    <Text style={{ color: t.text, fontSize:13 }}>
                      版本: {appMeta?.version ?? '...'}
                    </Text>
                  </div>
                  <div style={{ marginBottom: 8 }}>
                    <Text style={{ color: t.text, fontSize:13 }}>
                      许可证: {appMeta?.license ?? '...'}
                    </Text>
                  </div>
                  <div>
                    <Text style={{ color: t.textSec, fontSize:12 }}>
                      {health ? `编译器: GCC ${health.gcc_available ? '✓' : '✗'}  |  Clang ${health.clang_available ? '✓' : '✗'}` : '正在检测...'}
                    </Text>
                  </div>
                </>
              ),
            },
          ]}
        />
      </Drawer>
    </AntApp>
    </ConfigProvider>
  );
};

export default App;
