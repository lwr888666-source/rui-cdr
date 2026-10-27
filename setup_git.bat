@echo off
set "GIT=C:\Program Files\Git\cmd\git.exe"

"%GIT%" init -b main
"%GIT%" config user.name "lwr888666-source"
"%GIT%" config user.email "lwr888666-source@users.noreply.github.com"
"%GIT%" add .
"%GIT%" commit -m "feat: initial commit for Rui by CoreSDK"
"%GIT%" remote add origin https://github.com/lwr888666-source/rui-cdr.git
"%GIT%" status
