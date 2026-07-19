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
  input_text?: string;
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
  theme: string;
  word_wrap: 'off' | 'on' | 'wordWrapColumn';
}

export interface AppearanceSettings {
  background_image: string;
  opacity: number;
  frosted_glass: boolean;
  blur_amount: number;
  background_opacity: number;
  scrim_auto: boolean;
  scrim_opacity: number;
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
  restore_tabs: boolean;
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

export async function listFiles(subdir?: string): Promise<FileListResponse> {
  return invoke('list_files', { subdir: subdir || null });
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

export interface SessionTab {
  filename: string;
  language: string;
}

export interface SessionData {
  tabs: SessionTab[];
  active_tab: number;
}

export async function saveSession(session: SessionData): Promise<void> {
  return invoke('save_session', { req: { session } });
}

export async function loadSession(): Promise<SessionData> {
  return invoke('load_session');
}

export interface FileOpResponse {
  success: boolean;
  message: string;
}

export async function createDir(name: string): Promise<FileOpResponse> {
  return invoke('create_dir', { req: { name } });
}

export async function deleteFile(filename: string): Promise<FileOpResponse> {
  return invoke('delete_file', { req: { filename } });
}

export async function renameFile(oldName: string, newName: string): Promise<FileOpResponse> {
  return invoke('rename_file', { req: { old_name: oldName, new_name: newName } });
}

export async function copyFile(source: string, dest: string): Promise<FileOpResponse> {
  return invoke('copy_file', { req: { source, dest } });
}

export async function getSystemFonts(): Promise<string[]> {
  return invoke('get_system_fonts');
}

// ── 多测试点 ────────────────────────────────────────

export interface TestCase {
  input: string;
  expected?: string | null;
}

export interface TestCaseResult {
  index: number;
  output: string;
  exit_code: number | null;
  time_ms: number;
  passed: boolean | null;
}

export interface TestCasesRequest {
  code: string;
  filename: string;
  compiler: 'gcc' | 'clang';
  compile_options?: CompileOptions;
  options?: string;
  std?: string | null;
  compile_only: boolean;
  testcases: TestCase[];
}

export interface TestCasesResponse {
  success: boolean;
  compile_output: string;
  results: TestCaseResult[];
}

export async function runTestcases(req: TestCasesRequest): Promise<TestCasesResponse> {
  return invoke('run_testcases', { req });
}

// ── 对拍（差分测试）─────────────────────────────────

export interface StressRequest {
  solution_code: string;
  solution_filename: string;
  reference_code: string;
  reference_filename: string;
  generator_code: string;
  generator_filename: string;
  compiler: 'gcc' | 'clang';
  compile_options?: CompileOptions;
  options?: string;
  std?: string | null;
  iterations: number;
  timeout_ms: number;
}

export interface StressResponse {
  found: boolean;
  iterations: number;
  compile_error?: string | null;
  runtime_error?: string | null;
  counterexample_input?: string | null;
  solution_output?: string | null;
  reference_output?: string | null;
  timed_out: boolean;
}

export async function stressTest(req: StressRequest): Promise<StressResponse> {
  return invoke('stress_test', { req });
}

// ── 测试点持久化 ──────────────────────────────────────

export async function saveTestcases(filename: string, cases: TestCase[]): Promise<void> {
  return invoke('save_testcases', { filename, cases });
}

export async function loadTestcases(filename: string): Promise<TestCase[]> {
  return invoke('load_testcases', { filename });
}
