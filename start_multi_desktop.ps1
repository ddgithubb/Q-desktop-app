Start-Process powershell {npm run start; Read-Host} -NoNewWindow

For ($i=0; $i -lt 2; $i++) {
    Start-Process powershell {npm run desktop; Read-Host} -NoNewWindow
}