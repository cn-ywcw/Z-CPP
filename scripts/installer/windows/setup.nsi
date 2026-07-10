; Z-CPP Windows Installer (NSIS)
; 构建命令: makensis setup.nsi

!define PRODUCT_NAME "Z-CPP"
!define PRODUCT_VERSION "0.1.0"
!define PRODUCT_PUBLISHER "Z-CPP Team"
!define PRODUCT_WEB_SITE "https://github.com/cn-ywcw/Z-CPP"

!include "MUI2.nsh"
!include "FileFunc.nsh"

; ── 安装包属性 ────────────────────────────────────────

Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "Z-CPP-Setup-${PRODUCT_VERSION}.exe"
InstallDir "$PROGRAMFILES64\${PRODUCT_NAME}"
InstallDirRegKey HKLM "Software\${PRODUCT_NAME}" ""
RequestExecutionLevel admin

; ── 界面设置 ───────────────────────────────────────────

!define MUI_ABORTWARNING
!define MUI_ICON "${NSISDIR}\Contrib\Graphics\Icons\modern-install.ico"
!define MUI_UNICON "${NSISDIR}\Contrib\Graphics\Icons\modern-uninstall.ico"
!define MUI_WELCOMEPAGE_TITLE "欢迎安装 ${PRODUCT_NAME}"
!define MUI_WELCOMEPAGE_TEXT "Z-CPP 是一个面向算法竞赛选手的轻量级 C/C++ IDE。$\r$\n$\r$\n点击「下一步」继续安装。"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "SimpChinese"

; ── 安装段 ─────────────────────────────────────────────

Section "安装程序" SEC01
  SetOutPath "$INSTDIR"

  ; 后端二进制
  File "z-cpp-backend.exe"
  ; 前端构建产物（保留 frontend/dist 结构，供后端 .\frontend\dist 读取）
  SetOutPath "$INSTDIR\frontend\dist"
  File /r "frontend\dist"
  ; 启动脚本
  File "start.bat"
  File "README.md"

  ; 创建工作目录
  CreateDirectory "$INSTDIR\workspace"

  ; 创建快捷方式
  CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
  CreateShortCut "$SMPROGRAMS\${PRODUCT_NAME}\Z-CPP.lnk" "$INSTDIR\start.bat" \
    "" "$INSTDIR\z-cpp-backend.exe" 0 SW_SHOWNORMAL \
    "" "轻量级 C/C++ IDE"
  CreateShortCut "$DESKTOP\Z-CPP.lnk" "$INSTDIR\start.bat" \
    "" "$INSTDIR\z-cpp-backend.exe" 0 SW_SHOWNORMAL

  ; 写注册表
  WriteRegStr HKLM "Software\${PRODUCT_NAME}" "" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
    "DisplayName" "${PRODUCT_NAME} ${PRODUCT_VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
    "UninstallString" "$INSTDIR\uninst.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
    "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" \
    "Publisher" "${PRODUCT_PUBLISHER}"

  ; 卸载程序
  WriteUninstaller "$INSTDIR\uninst.exe"
SectionEnd

; ── 卸载段 ─────────────────────────────────────────────

Section "Uninstall"
  ; 删除快捷方式
  Delete "$DESKTOP\Z-CPP.lnk"
  RMDir /r "$SMPROGRAMS\${PRODUCT_NAME}"

  ; 删除安装目录
  RMDir /r "$INSTDIR"

  ; 删除注册表
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
  DeleteRegKey HKLM "Software\${PRODUCT_NAME}"
SectionEnd
