@REM net use Z: "\\vmware-host\Shared Folders" /persistent:yes
regsvr32.exe "Z:\azookey-windows\target\release\azookey_windows.dll" /u /s
start Z:\azookey-windows\target\release\azookey-server.exe
regsvr32.exe "Z:\azookey-windows\target\release\azookey_windows.dll" /s