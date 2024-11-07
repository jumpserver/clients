!macro customHeader
  RequestExecutionLevel admin
!macroend

!macro customInstall
    WriteRegStr HKCR "jms" "" "URL:jms"
    WriteRegStr HKCR "jms" "URL Protocol" ""
    WriteRegStr HKCR "jms\shell" "" ""
    WriteRegStr HKCR "jms\shell\open" "" ""
    ${If} ${RunningX64}
        WriteRegStr HKCR "jms\shell\open\command" "" '"$INSTDIR\resources\bin\windows\JumpServerClient.exe" "%1"'
    ${else}
        WriteRegStr HKCR "jms\shell\open\command" "" '"$INSTDIR\resources\bin\windows\JumpServerClient32.exe" "%1"'
    ${EndIf}
    AccessControl::GrantOnFile \
        "$INSTDIR\resources\bin" "(BU)" "GenericWrite + GenericRead"
    Pop $R0
    ${If} $R0 == error
        Pop $R0
        MessageBox MB_OK `AccessControl error: $R0`
    ${EndIf}
!macroend

!macro customUnInstall
    DeleteRegKey HKCR "jms"
!macroend