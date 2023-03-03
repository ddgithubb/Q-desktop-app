Start-Process powershell {npm run start; Read-Host} -NoNewWindow

For ($i=0; $i -lt $Args[0]; $i++) {
    if ( $Args[2] -eq 1 ) {
        Start-Process powershell {npm run desktopRelease; Read-Host}
    } else {
        if ($Args[1] -eq 1) {
            Start-Process powershell {npm run desktopDev; Read-Host}
        } else {
            Start-Process powershell {npm run desktopDev; Read-Host} -NoNewWindow
        }
    }
}

# npm run build

# For ($i=0; $i -lt $Args[0]; $i++) {
#     if ( $Args[2] -eq 1 ) {
#         Start-Process powershell {cargo run --release; Read-Host}
#     } else {
#         if ($Args[1] -eq 1) {
#             Start-Process powershell {cargo run; Read-Host}
#         } else {
#             Start-Process powershell {cargo run; Read-Host} -NoNewWindow
#         }
#     }
# }