import { invoke } from '@tauri-apps/api/core';

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

export interface EditorSettings {
  font_family: string;
  font_size: number;
  tab_size: number;
  theme: 'vs-dark' | 'vs-light' | 'hc-black';
  word_wrap: 'off' | 'on' | 'wordWrapColumn';
}

export interface AppearanceSettings {
  background_image: string;
  opacity: number;
  frosted_glass: boolean;
  blur_amount: number;
}

export interface Settings {
  gcc_path: string;
  clang_path: string;
  default_compiler: string;
  default_options: CompileOptions;
  workspace: string;
  editor: EditorSettings;
  appearance: AppearanceSettings;
  auto_save: boolean;
  default_compile_only: boolean;
}

export interface AppMeta {
  version: string;
  license: string;
}

export async function checkHealth(): Promise<HealthResponse> {
  return invoke('check_health');
}

export async function compileCode(req: CompileRequest): Promise<CompileResponse> {
  return invoke('compile_code', { req });
}

export interface LanguageInfo {
  name: string;
  extension: string;
  compilers: CompilerInfo[];
}

export async function getLanguages(): Promise<LanguageInfo[]> {
  return invoke('get_languages');
}

export async function getCompilers(): Promise<CompilerInfo[]> {
  return invoke('get_compilers');
}

export async function listFiles(): Promise<FileListResponse> {
  return invoke('list_files');
}

export async function createFile(filename: string, content = ''): Promise<{ success: boolean; message: string }> {
  return invoke('create_file', { req: { filename, content } });
}

export async function saveFile(filename: string, content: string): Promise<boolean> {
  const res = await invoke<{ success: boolean }>('save_file', { req: { filename, content } });
  return res.success;
}

export async function loadFile(filename: string): Promise<string | null> {
  const res = await invoke<{ success: boolean; content?: string }>('load_file', { filename });
  if (res.success) return res.content ?? null;
  return null;
}

export async function getSettings(): Promise<Settings> {
  const res = await invoke<{ settings: Settings }>('get_settings');
  return res.settings;
}

export async function saveSettings(settings: Settings): Promise<boolean> {
  const res = await invoke<{ success: boolean }>('save_settings', { req: { settings } });
  return res.success;
}

export async function getAppMeta(): Promise<AppMeta> {
  return invoke('get_app_meta');
}

export async function getSystemFonts(): Promise<string[]> {
  return invoke('get_system_fonts');
}
