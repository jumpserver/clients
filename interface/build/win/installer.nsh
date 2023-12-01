!macro customHeader
  RequestExecutionLevel admin
!macroend

!macro customInstall
    WriteRegStr HKCR "jms" "" "URL:jms"
    WriteRegStr HKCR "jms" "URL Protocol" ""
    WriteRegStr HKCR "jms\shell" "" ""
    WriteRegStr HKCR "jms\shell\open" "" ""
    WriteRegStr HKCR "jms\shell\open\command" "" '"$INSTDIR\resources\bin\windows\JumpServerClient.exe" "%1"'

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