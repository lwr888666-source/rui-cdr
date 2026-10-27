@echo off
chcp 65001 >nul
title 推送 Rui 到 GitHub
echo ========================================================
echo   正在将定制好的 Rui 项目推送到您的 GitHub 仓库...
echo   目标: https://github.com/lwr888666-source/rui-cdr.git
echo ========================================================
echo.
echo 如果首次弹出 GitHub 授权窗口，请点击 [Sign in with your browser] 确认授权即可。
echo.

"C:\Program Files\Git\cmd\git.exe" push -u origin main

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ----------------------------------------------------
    echo [提示] 如果上面提示冲突，正在尝试同步推送...
    "C:\Program Files\Git\cmd\git.exe" push -u origin main --force
)

echo.
echo ========================================================
echo   🎉 推送完成！
echo   请回到浏览器刷新 GitHub Actions 页面即可看到构建流水线！
echo ========================================================
echo.
pause
