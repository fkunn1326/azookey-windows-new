@REM net use Z: "\\vmware-host\Shared Folders" /persistent:yes
%windir%\System32\regsvr32.exe "D:\azookey-windows\target\debug\azookey_windows.dll" /u /s
%windir%\SysWoW64\regsvr32.exe "D:\azookey-windows\target\i686-pc-windows-msvc\debug\azookey_windows.dll" /u /s
start D:\azookey-windows\target\debug\azookey-server.exe
start D:\azookey-windows\target\debug\ui.exe
%windir%\System32\regsvr32.exe "D:\azookey-windows\target\debug\azookey_windows.dll" /s
%windir%\SysWoW64\regsvr32.exe "D:\azookey-windows\target\i686-pc-windows-msvc\debug\azookey_windows.dll" /s