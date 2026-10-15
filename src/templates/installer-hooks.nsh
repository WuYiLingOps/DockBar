!macro NSIS_HOOK_PREINSTALL
  StrCpy $1 "taskkill.exe /F /T /IM dockbar.exe"
  DetailPrint 'Exec: $1'
  nsExec::Exec $1
  Pop $0

  ; Clean static folder to remove assets from previous versions
  DetailPrint 'Cleaning static folder from previous installation...'
  RMDir /r "$INSTDIR\static"

  DetailPrint 'Cleaning webview2 runtime from previous installation...'
  RMDir /r "$INSTDIR\runtime"

  File /a "${__FILEDIR__}\..\..\SHA256SUMS"
  File /a "${__FILEDIR__}\..\..\SHA256SUMS.sig"

  ; Include PDB file only for nightly builds
  ${StrLoc} $0 "${VERSION}" "nightly" ">"
  ${If} $0 != ""
    File /a "${__FILEDIR__}\..\..\dockbar.pdb"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; No background service in this build: the app runs directly from the
  ; Start Menu shortcut / autostart entry.
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  StrCpy $1 "taskkill.exe /F /T /IM dockbar.exe"
  DetailPrint 'Exec: $1'
  nsExec::Exec $1
  Pop $0
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  Delete "$INSTDIR\SHA256SUMS"
  Delete "$INSTDIR\SHA256SUMS.sig"
  Delete "$INSTDIR\dockbar.pdb"

  ; Refresh file associations icons
  !insertmacro UPDATEFILEASSOC
!macroend
