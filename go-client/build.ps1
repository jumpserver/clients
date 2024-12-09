set CGO_ENABLED=0
set GOOS=windows

set GOARCH=amd64
go build -trimpath -ldflags "-w -s -H windowsgui" -o build/windows/JumpServerClient.exe ./cmd/awaken/
set GOARCH=386
go build -trimpath -ldflags "-w -s -H windowsgui" -o build/windows/JumpServerClient32.exe ./cmd/awaken/


Copy-Item -Path "build/*" -Destination "../ui/bin/" -Recurse -Force
Copy-Item -Path config.json -Destination "../ui/bin/" -Force
Copy-Item -Path putty.exe -Destination "../ui/bin/windows/" -Force
Copy-Item -Path "pkg/autoit/*.dll" -Destination "../ui/bin/windows/" -Force