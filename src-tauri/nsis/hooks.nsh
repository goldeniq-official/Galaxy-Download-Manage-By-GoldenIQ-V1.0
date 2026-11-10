; SPDX-License-Identifier: RPL-1.5
; B-06/B-07: register the native messaging host for every known browser (HKCU, no admin) after install;
; T-03: remove it (and the autostart entry) before uninstall. Downloaded files are never touched.
!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToLog '"$INSTDIR\gdm-bridge.exe" register'
  Pop $0
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::ExecToLog '"$INSTDIR\gdm-bridge.exe" unregister'
  Pop $0
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "Galaxy Download Manager"
!macroend
