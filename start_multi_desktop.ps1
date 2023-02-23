Start-Process powershell {npm run start; Read-Host} -NoNewWindow

For ($i=0; $i -lt $Args[0]; $i++) {
    Start-Process powershell {npm run desktopDev; Read-Host}
}