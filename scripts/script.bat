@REM net use Z: "\\vmware-host\Shared Folders" /persistent:yes
regsvr32.exe "D:\azookey-windows\target\release\azookey_windows.dll" /u /s
start D:\azookey-windows\target\release\azookey-server.exe
start D:\azookey-windows\target\release\ui.exe
regsvr32.exe "D:\azookey-windows\target\release\azookey_windows.dll" /s