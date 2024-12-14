@REM net use Z: "\\vmware-host\Shared Folders" /persistent:yes
regsvr32.exe "Z:\azookey-windows\target\release\azookey_windows.dll" /u /s
regsvr32.exe "Z:\azookey-windows\target\release\azookey_windows.dll" /s