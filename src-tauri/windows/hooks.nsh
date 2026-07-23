; Force-quit any running instance before replacing files.
; Manual installers cannot reliably "hot-swap" a live process the way updater relaunch does.
!macro NSIS_HOOK_PREINSTALL
  ; Ignore errors when the process is not running (exit code non-zero).
  ExecWait 'cmd /c taskkill /F /IM "${MAINBINARYNAME}.exe" /T >nul 2>&1 & exit /b 0'
  Sleep 400
!macroend

; After files are installed: prepare default content dir under the user profile.
; Finish page already offers "Run Note Reader" (checked by default) to start the new build.
!macro NSIS_HOOK_POSTINSTALL
  CreateDirectory "$PROFILE\NoteReader"
  CreateDirectory "$PROFILE\NoteReader\columns"
  FileOpen $0 "$PROFILE\NoteReader\README.txt" w
  FileWrite $0 "Note Reader — local columns folder$\r$\n$\r$\n"
  FileWrite $0 "Put each column as a subfolder under columns\, e.g.:$\r$\n"
  FileWrite $0 "  columns\MySQL\01 intro.md$\r$\n$\r$\n"
  FileWrite $0 "Notes:$\r$\n"
  FileWrite $0 "  - Install path: chosen in this installer (app binary)$\r$\n"
  FileWrite $0 "  - Read path: default %USERPROFILE%\NoteReader\columns (changeable in Settings)$\r$\n"
  FileClose $0
!macroend
