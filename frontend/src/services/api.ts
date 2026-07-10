/// Z-CPP 前端 API 服务
/// 封装与后端的所有 HTTP 通信

import axios from 'axios';

const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
});

export interface CompileRequest {
  code: string;
  filename: string;
  compiler: 'gcc' | 'clang';
  options: string;
  std: string | null;
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

export interface LanguageInfo {
  name: string;
  extension: string;
  compilers: CompilerInfo[];
}

/// 健康检查
export async function checkHealth(): Promise<HealthResponse> {
  const res = await api.get<HealthResponse>('/health');
  return res.data;
}

/// 编译并运行
export async function compileCode(req: CompileRequest): Promise<CompileResponse> {
  const res = await api.post<CompileResponse>('/compile', req);
  return res.data;
}

/// 获取支持的语言列表
export async function getLanguages(): Promise<LanguageInfo[]> {
  const res = await api.get<LanguageInfo[]>('/languages');
  return res.data;
}

/// 获取可用编译器列表
export async function getCompilers(): Promise<CompilerInfo[]> {
  const res = await api.get<CompilerInfo[]>('/compilers');
  return res.data;
}

/// 保存文件
export async function saveFile(filename: string, content: string): Promise<boolean> {
  const res = await api.post('/save', { filename, content });
  return res.data.success;
}

/// 加载文件
export async function loadFile(filename: string): Promise<string | null> {
  const res = await api.get(`/load/${filename}`);
  if (res.data.success) {
    return res.data.content;
  }
  return null;
}
