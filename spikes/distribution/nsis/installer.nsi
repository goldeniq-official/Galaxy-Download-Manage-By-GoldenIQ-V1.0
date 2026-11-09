; SPIKE ONLY (G2: S1a + S9) - per-user NSIS installer, throwaway, never shipped.
; Build: makensis /DVERSION=0.1.0 /DBRIDGE=<path to gdm-bridge.exe> /DOUT=<setup.exe> installer.nsi
Unicode true
ManifestDPIAware true
RequestExecutionLevel user          ; T-01: per-user, no admin
!include "MUI2.nsh"
!define NAME "GDM spike"
!define UNKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\GoldenIQ.GalaxyDownloadManager.Spike"
Name "${NAME}"
OutFile "${OUT}"
InstallDir "$LOCALAPPDATA\Programs\GDM-Spike"
InstallDirRegKey HKCU "${UNKEY}" "InstallLocation"
VIProductVersion "${VERSION}.0"
VIAddVersionKey /LANG=0 "ProductName" "${NAME}"
VIAddVersionKey /LANG=0 "CompanyName" "GoldenIQ"
VIAddVersionKey /LANG=0 "FileVersion" "${VERSION}"

!define MUI_ABORTWARNING
!define MUI_LANGDLL_REGISTRY_ROOT HKCU      ; remember the choice so the uninstaller uses it
!define MUI_LANGDLL_REGISTRY_KEY "${UNKEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "InstallerLanguage"
BrandingText " "
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGEEX "${__FILEDIR__}" "Khmer"   ; first = default
!insertmacro MUI_LANGUAGE "English"

Function .onInit
  ; Khmer pre-selected (not just listed first). LangDLL is called directly because MUI2
  ; passes no font, and the default MS Shell Dlg clips Khmer in the combo box.
  StrCpy $LANGUAGE 1107
  IfSilent done
  LangDLL::LangDialog "Installer Language / ភាសា" "សូមជ្រើសភាសា / Please select a language." A "ខ្មែរ" 1107 "English" 1033 "" "Khmer UI" 9
  Pop $0
  StrCmp $0 "cancel" 0 +2
    Abort
  StrCpy $LANGUAGE $0
  done:
FunctionEnd

Section
  SetOutPath "$INSTDIR"
  File "/oname=gdm-bridge.exe" "${BRIDGE}"
  ; register the native messaging host for Chrome, Edge, Firefox (HKCU, owned by this install)
  nsExec::ExecToLog '"$INSTDIR\gdm-bridge.exe" register "$INSTDIR\gdm-bridge.exe"'
  Pop $0
  StrCmp $0 "0" +3
    SetErrorLevel 2
    Abort "gdm-bridge register failed ($0)"
  WriteUninstaller "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "${UNKEY}" "DisplayName" "${NAME}"
  WriteRegStr HKCU "${UNKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNKEY}" "Publisher" "GoldenIQ"
  WriteRegStr HKCU "${UNKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNKEY}" "UninstallString" '"$INSTDIR\uninstall.exe"'
  WriteRegStr HKCU "${UNKEY}" "QuietUninstallString" '"$INSTDIR\uninstall.exe" /S'
  WriteRegStr HKCU "${UNKEY}" "InstallerLanguage" $LANGUAGE
  WriteRegDWORD HKCU "${UNKEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNKEY}" "NoRepair" 1
SectionEnd

Function un.onInit
  !insertmacro MUI_UNGETLANGUAGE
FunctionEnd

Section "Uninstall"
  nsExec::ExecToLog '"$INSTDIR\gdm-bridge.exe" unregister'
  Delete "$INSTDIR\gdm-bridge.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  ; remove only what this install owns; anything else in $LOCALAPPDATA\GDM (future user data) stays
  RMDir /r "$LOCALAPPDATA\GDM\nmh"
  RMDir /r "$LOCALAPPDATA\GDM\components"
  RMDir "$LOCALAPPDATA\GDM"            ; only if now empty
  DeleteRegKey HKCU "${UNKEY}"
SectionEnd
