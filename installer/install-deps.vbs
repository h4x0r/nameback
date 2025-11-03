' VBScript wrapper for dependency installation with MSI progress reporting
' This script has access to the Session object which can report progress to MSI UI

Option Explicit

Dim session, installer, shell, fso, namebackExe, logFile, wshShell
Dim tempFolder, stdoutFile, stderrFile

' Get MSI session object (automatically available in custom action scripts)
On Error Resume Next
Set session = Session
If Err.Number <> 0 Then
    ' Fallback if Session not available (shouldn't happen in MSI context)
    WScript.Echo "ERROR: Session object not available"
    WScript.Quit 0
End If
On Error Goto 0

' Get installer folder
Dim installFolder
installFolder = session.Property("CustomActionData")
If installFolder = "" Then
    installFolder = session.Property("INSTALLFOLDER")
End If

' Create file system object
Set fso = CreateObject("Scripting.FileSystemObject")
Set wshShell = CreateObject("WScript.Shell")

' Build path to nameback.exe
namebackExe = fso.BuildPath(installFolder, "nameback.exe")

' Check if executable exists
If Not fso.FileExists(namebackExe) Then
    session.Message &H01000000, "ERROR: nameback.exe not found at: " & namebackExe
    WScript.Quit 0
End If

' Report progress to MSI
session.Message &H0A000000, "Checking network connectivity..."
session.Message &H0A000000, "Installing Scoop package manager..."

' Run nameback.exe --install-deps
On Error Resume Next
Dim result
result = wshShell.Run("""" & namebackExe & """ --install-deps", 0, True)
On Error Goto 0

' Report completion
If result = 0 Then
    session.Message &H0A000000, "Dependencies installed successfully"
Else
    session.Message &H01000000, "Dependency installation completed with code: " & result
End If

' Always exit success (Return="ignore" in WiX)
WScript.Quit 0
