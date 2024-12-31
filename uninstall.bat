@echo off
SET script_dir=%~dp0
SET app_name="tif2dds"
SET app_executable="%app_name%.exe"
SET app_version="<version>"
SET script_version="1.0"

echo =============================================
echo
echo   %app_name% Uninstaller [App Version %app_version%, Script Version %script_version%]
echo   Executing from: %script_dir%
echo
echo =============================================
echo

REM Check if the executable exists
if not exist "%script_dir%%app_executable%" (
    echo Error: %app_executable% not found in %script_dir%.
    echo Uninstallation cannot continue.
    pause
    exit /b
)

REM Change directory to the script's folder
cd /d "%script_dir%"

REM Run the uninstall command
%app_executable% uninstall

if %ERRORLEVEL% EQU 0 (
    echo Uninstallation completed successfully.
) else (
    echo Uninstallation failed with error code %ERRORLEVEL%.
    echo Please check your installation or contact support.
)

pause
exit /b
