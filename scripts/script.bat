@REM net use Z: "\\vmware-host\Shared Folders" /persistent:yes
regsvr32.exe "D:\azookey-windows\target\debug\azookey_windows.dll" /u /s
start D:\azookey-windows\target\debug\azookey-server.exe
start D:\azookey-windows\target\debug\ui.exe
regsvr32.exe "D:\azookey-windows\target\debug\azookey_windows.dll" /s