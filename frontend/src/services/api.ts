/// Z-CPP 前端 API 服务
/// 封装与后端的所有 HTTP 通信

import axios from 'axios';

const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
});

// ── 类型定义 ──────────────────────────────────────────

export interface CompileOptions {
  optimization: string;
  warnings: string;
  standard: string;
  extra_flags: string;
}

export interface CompileRequest {
  code: string;
  filename: string;
  compiler: 'gcc' | 'clang';
  compile_options?: CompileOptions;
  options?: string;
  std?: string | null;
  compile_only: boolean;
}

export interface CompileResponse {
  success: boolean;
  compile_output: string;
  run_output: string;
  run_time_ms: number | null;
  exit_code: number | null;
}

export interface HealthResponse {
  status: string;
  version: string;
  gcc_available: boolean;
  clang_available: boolean;
}

export interface CompilerInfo {
  name: string;
  command: string;
  available: boolean;
}

export interface FileInfo {
  name: string;
  path: string;
  size: number;
  is_dir: boolean;
}

export interface FileListResponse {
  files: FileInfo[];
  workspace: string;
}

export interface Settings {
  gcc_path: string;
  clang_path: string;
  default_compiler: string;
  default_options: CompileOptions;
  workspace: string;
}

// ── API 函数 ──────────────────────────────────────────

export async function checkHealth(): Promise<HealthResponse> {
  const res = await api.get<HealthResponse>('/health');
  return res.data;
}

export async function compileCode(req: CompileRequest): Promise<CompileResponse> {
  const res = await api.post<CompileResponse>('/compile', req);
  return res.data;
}

export async function getLanguages(): Promise<any[]> {
  const res = await api.get('/languages');
  return res.data;
}

export async function getCompilers(): Promise<CompilerInfo[]> {
  const res = await api.get<CompilerInfo[]>('/compilers');
  return res.data;
}

export async function listFiles(): Promise<FileListResponse> {
  const res = await api.get<FileListResponse>('/files');
  return res.data;
}

export async function createFile(filename: string, content = ''): Promise<{ success: boolean; message: string }> {
  const res = await api.post('/files', { filename, content });
  return res.data;
}

export async function saveFile(filename: string, content: string): Promise<boolean> {
  const res = await api.post('/save', { filename, content });
  return res.data.success;
}

export async function loadFile(filename: string): Promise<string | null> {
  const res = await api.get(`/load/${encodeURIComponent(filename)}`);
  if (res.data.success) return res.data.content;
  return null;
}

export async function getSettings(): Promise<Settings> {
  const res = await api.get('/settings');
  return res.data.settings;
}

export async function saveSettings(settings: Settings): Promise<boolean> {
  const res = await api.post('/settings', { settings });
  return res.data.success;
}
