; =============================================================================
; Ducker NSIS Installer Hooks
; "Be Simple, Be Duck."
;
; These hooks are called by the Tauri NSIS bundler at specific lifecycle
; points during installation and uninstallation.
;
; The CLI binary (ducker.exe) is bundled as a resource in the installer.
; These hooks handle:
; 1. Component selection: Prompting user whether to install the CLI component
; 2. Installing the CLI binary alongside the Desktop app (or skipping it)
; 3. Adding/removing the CLI from the user's PATH (safe, duplicate-free, reversible)
; 4. Cleaning up Ducker-owned entries on uninstall (preserving user data)
; =============================================================================

Var DuckerInstallCLI

; ---------------------------------------------------------------------------
; PREINSTALL — runs before files are copied
; ---------------------------------------------------------------------------
!macro NSIS_HOOK_PREINSTALL
  DetailPrint "Preparing Ducker installation..."

  ; Default CLI selection to YES for passive/silent installs
  StrCpy $DuckerInstallCLI "1"

  ${If} $PassiveMode <> 1
  ${AndIfNot} ${Silent}
    ; Prompt user for component selection: Desktop only vs Desktop + CLI
    ; MessageBox supports up to 2 return checks; cancel falls through to Abort
    MessageBox MB_YESNOCANCEL|MB_ICONQUESTION \
      "Deseja instalar a ferramenta de linha de comando (Ducker CLI) e adiciona-la ao PATH?$\n$\n[Sim] Instalar Ducker Desktop + Ducker CLI (com PATH)$\n[Nao] Instalar apenas o Ducker Desktop$\n[Cancelar] Cancelar a instalacao" \
      /SD IDYES \
      IDYES _ducker_choose_yes \
      IDNO _ducker_choose_no

    ; If user clicked Cancel:
    Abort "Instalacao cancelada pelo usuario."

    _ducker_choose_no:
      StrCpy $DuckerInstallCLI "0"
      DetailPrint "Componente selecionado: Apenas Ducker Desktop"
      Goto _ducker_preinstall_done

    _ducker_choose_yes:
      StrCpy $DuckerInstallCLI "1"
      DetailPrint "Componentes selecionados: Ducker Desktop + Ducker CLI"
      Goto _ducker_preinstall_done

    _ducker_preinstall_done:
  ${EndIf}
!macroend

; ---------------------------------------------------------------------------
; POSTINSTALL — runs after all files are installed, registry keys set,
;               and shortcuts created
; ---------------------------------------------------------------------------
!macro NSIS_HOOK_POSTINSTALL
  ${If} $DuckerInstallCLI == "1"
    ; --- Install CLI binary ---
    ; The CLI binary is bundled as a resource at resources/ducker.exe
    ; We copy it to the installation directory root for easy PATH access
    IfFileExists "$INSTDIR\resources\ducker.exe" 0 _ducker_skip_cli

      CopyFiles /SILENT "$INSTDIR\resources\ducker.exe" "$INSTDIR\ducker.exe"
      DetailPrint "Ducker CLI instalado em $INSTDIR\ducker.exe"

      ; --- Add to user PATH (safe and reversible) ---
      ReadRegStr $0 HKCU "Environment" "Path"

      ; Check if $INSTDIR is already in PATH to avoid duplicates
      Push $0
      Push "$INSTDIR"
      Call _DuckerStrContains
      Pop $1
      StrCmp $1 "1" _ducker_skip_path

        ; Append our directory
        StrCmp $0 "" 0 +3
          StrCpy $0 "$INSTDIR"
          Goto _ducker_write_path
        StrCpy $0 "$0;$INSTDIR"

      _ducker_write_path:
        WriteRegExpandStr HKCU "Environment" "Path" "$0"
        ; Broadcast WM_SETTINGCHANGE so running shells pick up the change
        SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=1000
        DetailPrint "Diretorio $INSTDIR adicionado ao PATH do usuario"

      _ducker_skip_path:

      ; Record that CLI was installed for the uninstaller
      WriteRegStr SHCTX "${UNINSTKEY}" "DuckerCLI" "1"

    _ducker_skip_cli:
  ${Else}
    DetailPrint "Ducker CLI ignorado conforme escolha do usuario."
    ; Clean up the resource binary so it does not occupy space when not selected
    Delete "$INSTDIR\resources\ducker.exe"
    WriteRegStr SHCTX "${UNINSTKEY}" "DuckerCLI" "0"
  ${EndIf}

  DetailPrint "Ducker instalado com sucesso! Be Simple, Be Duck."
!macroend

; ---------------------------------------------------------------------------
; Helper: check if string $R0 contains substring $R1
; Push haystack, Push needle, Call, Pop result ("1" or "0")
; ---------------------------------------------------------------------------
Function _DuckerStrContains
  Exch $R1  ; needle
  Exch
  Exch $R0  ; haystack
  Push $R2
  Push $R3
  Push $R4
  Push $R5

  StrLen $R3 $R1
  StrLen $R4 $R0
  StrCpy $R2 0

  _ducker_sc_loop:
    IntCmp $R2 $R4 _ducker_sc_notfound _ducker_sc_notfound
    StrCpy $R5 $R0 $R3 $R2
    StrCmp $R5 $R1 _ducker_sc_found
    IntOp $R2 $R2 + 1
    Goto _ducker_sc_loop

  _ducker_sc_found:
    StrCpy $R0 "1"
    Goto _ducker_sc_done

  _ducker_sc_notfound:
    StrCpy $R0 "0"

  _ducker_sc_done:
  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Exch $R0
  Exch
  Pop $R1
FunctionEnd

; ---------------------------------------------------------------------------
; PREUNINSTALL — runs before files are removed
; ---------------------------------------------------------------------------
!macro NSIS_HOOK_PREUNINSTALL
  DetailPrint "Preparando desinstalacao do Ducker..."

  ; --- Check if CLI was installed ---
  ReadRegStr $2 SHCTX "${UNINSTKEY}" "DuckerCLI"
  ${If} $2 == "1"
    ; --- Remove $INSTDIR from user PATH ---
    ReadRegStr $0 HKCU "Environment" "Path"
    StrCmp $0 "" _ducker_un_skip_path

    ; Remove patterns: ";$INSTDIR" or "$INSTDIR;" or bare "$INSTDIR"
    ; Use WordReplace from WordFunc.nsh (included by Tauri template)
    ${WordReplace} $0 ";$INSTDIR" "" "+" $0
    ${WordReplace} $0 "$INSTDIR;" "" "+" $0
    ${WordReplace} $0 "$INSTDIR" "" "+" $0

    WriteRegExpandStr HKCU "Environment" "Path" "$0"
    SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=1000
    DetailPrint "Removido $INSTDIR do PATH do usuario"

    _ducker_un_skip_path:

    ; Clean up CLI binary at install root
    Delete "$INSTDIR\ducker.exe"
  ${EndIf}
!macroend

; ---------------------------------------------------------------------------
; POSTUNINSTALL — runs after all files are removed
; ---------------------------------------------------------------------------
!macro NSIS_HOOK_POSTUNINSTALL
  ; We intentionally preserve user data:
  ; - User config (%APPDATA%\dev.oferraz.ducker\)
  ; - TLS certificates and private keys
  ; - Received files and downloads
  ; - User cosmetics, ducks and preferences
  DetailPrint "Ducker desinstalado. Seus arquivos pessoais e configuracoes foram preservados."
!macroend
