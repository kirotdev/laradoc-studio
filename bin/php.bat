@echo off
:: Laradoc Studio — Smart PHP CLI Shim (Windows)
:: Dynamically routes php command to PHP 8.2, 8.3, 8.4, or 8.5

setlocal enabledelayedexpansion

set "PHP_VERSION="

if exist ".php-version" (
    set /p PHP_VERSION=<.php-version
)

if "%PHP_VERSION%"=="" (
    if exist "composer.json" (
        for /f "tokens=2 delims=:, " %%a in ('findstr /i "\"php\"" composer.json 2^>nul') do (
            set "RAW_VER=%%~a"
            for /f "tokens=1,2 delims=." %%x in ("!RAW_VER!") do (
                set "CLEAN_MAJOR=%%x"
                set "CLEAN_MAJOR=!CLEAN_MAJOR:^=!"
                set "CLEAN_MAJOR=!CLEAN_MAJOR:~-1!"
                set "CLEAN_MINOR=%%y"
                set "CLEAN_MINOR=!CLEAN_MINOR:~0,1!"
                set "PHP_VERSION=!CLEAN_MAJOR!.!CLEAN_MINOR!"
            )
        )
    )
)

if "%PHP_VERSION%"=="" (
    if defined LARADOC_PHP_VERSION (
        set "PHP_VERSION=%LARADOC_PHP_VERSION%"
    ) else (
        set "PHP_VERSION=8.3"
    )
)

set "PHP_BIN=%~dp0php\%PHP_VERSION%\php.exe"
if not exist "%PHP_BIN%" (
    set "PHP_BIN=%LOCALAPPDATA%\laradoc-studio\bin\php\%PHP_VERSION%\php.exe"
)
if not exist "%PHP_BIN%" (
    for /f "tokens=*" %%i in ('where php 2^>nul') do (
        if not defined PHP_BIN set "PHP_BIN=%%i"
    )
)

if exist "%PHP_BIN%" (
    "%PHP_BIN%" %*
) else (
    php %*
)

endlocal
